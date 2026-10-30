// The game's renderer: per-fragment basic / Lambert / Phong surface shading,
// points, sprites and lines, exponential-squared fog, plus the game's shader
// patches (local light map, world-space wall UVs, creature skin detail, the
// Smiler's eyes, the Watcher's sheet, the House shadows, the Stairwell face),
// surface wear on props, floor occlusion and a highlight shoulder.
// Variants are function constants.
#include <metal_stdlib>
using namespace metal;

// ------------------------------------------------------------ variants
constant int  KIND        [[function_constant(0)]]; // 0 basic 1 lambert 2 phong 3 points 4 sprite 5 line
constant bool HAS_MAP     [[function_constant(1)]];
constant int  VCOLOR      [[function_constant(2)]]; // 0 none, 3 rgb, 4 rgba
constant bool INSTANCED   [[function_constant(3)]];
constant bool INST_COLOR  [[function_constant(4)]];
constant bool SKINNED     [[function_constant(5)]];
constant int  MORPHS      [[function_constant(6)]]; // morph target count (0..8)
constant bool USE_FOG     [[function_constant(7)]];
constant bool ALPHA_TEST  [[function_constant(8)]];
constant bool DOUBLE_SIDE [[function_constant(9)]];
constant bool FLIP_SIDED  [[function_constant(10)]];
constant bool FLAT        [[function_constant(11)]];
constant bool LL          [[function_constant(12)]]; // local light-map patch
constant int  PATCH       [[function_constant(13)]]; // Patch: 0 none 1 ArchUV 2 SkinDetail 3 SmilerEye 4 WatcherSheet 5 Shadow body 6 Shadow edge 7 Stairwell 8 WindowView 9 Glass
constant bool HAS_NORMAL  [[function_constant(14)]];
constant bool HAS_UV      [[function_constant(15)]];
constant bool OPAQUE      [[function_constant(16)]];
constant bool HAS_BUMP    [[function_constant(17)]];
constant bool MORPH_NRM   [[function_constant(18)]];
constant bool MORPH_REL   [[function_constant(19)]];
constant bool SIZE_ATTEN  [[function_constant(20)]];
constant bool HAS_VC_DATA [[function_constant(21)]]; // geometry actually has colours
constant bool WEAR        [[function_constant(22)]]; // procedural wear (lit prop surfaces)
constant bool PORTAL      [[function_constant(23)]]; // seen through a window (Material::portal)

// ------------------------------------------------------------ uniforms
struct Frame {
    float4x4 view;
    float4x4 proj;
    float4 ambient; // rgb: sum of ambient lights
    float4 fog; // rgb colour (linear, uploaded as-is), a: density
    float4 llGrid;
    float4 llAmbD; // rgb
    float4 llSkyD;
    float4 llGndD;
    float4 llFogLit;
    float4 llParams; // x on, y scale, z fog density factor where lit, w time
    float4 viewport; // x w, y h (pixels), z point scale (h/2), w unused
    uint4  counts; // x hemi, y point, z spot
    float4 look; // x floor occlusion amount, y corner occlusion amount
};
struct Hemi  { float4 sky; float4 ground; float4 dir; };
struct Point { float4 pos; float4 color; }; // pos.w distance, color.w decay
struct Spot  { float4 pos; float4 dir; float4 color; float4 cone; }; // pos.w distance, dir.w decay, cone.x cos(angle), cone.y cos(angle × (1 − penumbra))

struct Object {
    float4x4 model;
    float4x4 modelView;
    float4x4 normalMatrix; // 3x3 in a 4x4
    float4x4 bindMatrix;
    float4x4 bindMatrixInv;
    float4   morph[2]; // up to 8 influences
    float4   spriteCenterRot; // xy center, z rotation
};
struct Material {
    float4 diffuse; // rgb colour, a opacity
    float4 emissive; // rgb (× intensity)
    float4 specular; // rgb, a shininess
    float4 mapT0; // uv transform rows: (a, b, c) ...
    float4 mapT1;
    float4 misc; // x alpha test, y point size, z bump scale, w map flip-y
    float4 arch; // ArchUV: x scale, y vscale, z wallpaper
    float4 patch[4]; // patch parameters
    float4 surf; // x wear amount, y tone-mapped
    float4 portal[4]; // PortalU: centre + half width, right + half height, normal + slab, sky + haze
};
struct LightList { uint count; uint pad[3]; ushort idx[96]; };

// ------------------------------------------------------------ helpers
static inline float3 srgbOETF(float3 v) {
    return select(pow(v, float3(0.41666)) * 1.055 - float3(0.055), v * 12.92, v <= float3(0.0031308));
}
static inline float pow2(float x) { return x * x; }
static inline float pow4(float x) { float x2 = x * x; return x2 * x2; }
static inline float distanceAttenuation(float d, float cutoff, float decay) {
    float f = 1.0 / max(pow(d, decay), 0.01);
    if (cutoff > 0.0) f *= pow2(saturate(1.0 - pow4(d / cutoff)));
    return f;
}
static inline float3 BRDF_Lambert(float3 c) { return 0.3183098861837907 * c; }
static inline float3 F_Schlick(float3 f0, float f90, float dotVH) {
    float fresnel = exp2((-5.55473 * dotVH - 6.98316) * dotVH);
    return f0 * (1.0 - fresnel) + (f90 * fresnel);
}
static inline float3 BRDF_BlinnPhong(float3 L, float3 V, float3 N, float3 spec, float shin) {
    float3 H = normalize(L + V);
    float dotNH = saturate(dot(N, H));
    float dotVH = saturate(dot(V, H));
    float3 F = F_Schlick(spec, 1.0, dotVH);
    float D = 0.3183098861837907 * (shin * 0.5 + 1.0) * pow(dotNH, shin);
    return F * (0.25 * D);
}
static inline float3 inverseTransformDirection(float3 dir, float4x4 m) {
    return normalize((float4(dir, 0.0) * m).xyz);
}
// value noise in 0..1 (hashed lattice, smooth interpolation)
static inline float hash13(float3 p) {
    p = fract(p * 0.1031);
    p += dot(p, p.zyx + 31.32);
    return fract((p.x + p.y) * p.z);
}
static inline float vnoise(float3 p) {
    float3 i = floor(p), f = fract(p);
    float3 u = f * f * (3.0 - 2.0 * f);
    float a = hash13(i), b = hash13(i + float3(1, 0, 0)), c = hash13(i + float3(0, 1, 0)), d = hash13(i + float3(1, 1, 0));
    float e = hash13(i + float3(0, 0, 1)), g = hash13(i + float3(1, 0, 1)), h = hash13(i + float3(0, 1, 1)), k = hash13(i + float3(1, 1, 1));
    return mix(mix(mix(a, b, u.x), mix(c, d, u.x), u.y), mix(mix(e, g, u.x), mix(h, k, u.x), u.y), u.z);
}
// a toy-bright colour muted toward its own grey: chroma (largest minus smallest
// channel, in a gamma close to sRGB) rolls off above a knee
static inline float3 limitChroma(float3 lin) {
    float3 s = sqrt(max(lin, 0.0));
    float hi = max(s.r, max(s.g, s.b)), lo = min(s.r, min(s.g, s.b));
    float c = hi - lo;
    const float knee = 0.3, top = 0.52;
    if (c <= knee) return lin;
    float want = knee + (top - knee) * (1.0 - exp(-(c - knee) / (top - knee)));
    float l = dot(s, float3(0.2126, 0.7152, 0.0722));
    s = l + (s - l) * (want / c);
    return s * s;
}
// highlight shoulder: linear up to the knee, then an exponential roll-off toward 1
static inline float3 filmShoulder(float3 c) {
    const float knee = 0.68;
    float3 over = max(c - knee, 0.0);
    return min(c, knee) + (1.0 - knee) * (1.0 - exp(-over / (1.0 - knee)));
}

// ------------------------------------------------------------ vertex
struct VOut {
    float4 position [[position]];
    float  pointSize [[point_size]];
    float3 viewPos; // view-space position, negated (= -mvPosition)
    float3 normal;
    float2 uv;
    float4 color;
    float  fogDepth;
    float3 llPos;
    float3 detPos;
    float3 detNrm;
    float4 extra; // shadow: x wisp, y rim, z floorY
    float3 smokeP;
    float  portal; // 1 beyond the window's plane (PORTAL)
    float  portalDist; // its true distance from the eye
};

vertex VOut vs_main(uint vid [[vertex_id]], uint iid [[instance_id]],
                    constant Frame& F [[buffer(0)]],
                    constant Object& O [[buffer(1)]],
                    constant Material& M [[buffer(2)]],
                    const device packed_float3* positions [[buffer(3)]],
                    const device packed_float3* normals [[buffer(4)]],
                    const device float2* uvs [[buffer(5)]],
                    const device float* colors [[buffer(6)]],
                    const device float4x4* instMats [[buffer(7)]],
                    const device packed_float3* instCols [[buffer(8)]],
                    const device float4* skinIdx [[buffer(9)]],
                    const device float4* skinW [[buffer(10)]],
                    const device float4x4* bones [[buffer(11)]],
                    const device packed_float3* morphPos [[buffer(12)]],
                    const device packed_float3* morphNrm [[buffer(13)]],
                    constant uint& vcount [[buffer(14)]])
{
    VOut o;
    float3 transformed;
    float3 objectNormal = float3(0, 1, 0);
    float2 uv = float2(0);
    if (KIND == 4) {
        // sprite quad: 4 vertices from vid
        float2 q = float2((vid & 1) ? 0.5 : -0.5, (vid & 2) ? 0.5 : -0.5);
        transformed = float3(q, 0.0);
        uv = q + 0.5;
    } else {
        transformed = float3(positions[vid]);
        if (HAS_NORMAL) objectNormal = float3(normals[vid]);
        if (HAS_UV) uv = uvs[vid];
    }
    float3 restPos = transformed;
    float3 restNrm = objectNormal;
    // morph targets (relative: added to the base; absolute: blended with it)
    if (MORPHS > 0) {
        float inf[8] = { O.morph[0].x, O.morph[0].y, O.morph[0].z, O.morph[0].w, O.morph[1].x, O.morph[1].y, O.morph[1].z, O.morph[1].w };
        float sum = 0.0;
        for (int k = 0; k < MORPHS; k++) sum += inf[k];
        float base = MORPH_REL ? 1.0 : 1.0 - sum;
        float3 p = transformed * base;
        float3 n = objectNormal * base;
        for (int k = 0; k < MORPHS; k++) {
            p += float3(morphPos[k * vcount + vid]) * inf[k];
            if (MORPH_NRM) n += float3(morphNrm[k * vcount + vid]) * inf[k];
        }
        transformed = p;
        if (MORPH_NRM) objectNormal = n;
    }
    // vertex colours
    float4 vColor = float4(1.0);
    if (VCOLOR == 3) vColor = HAS_VC_DATA ? float4(colors[vid * 3], colors[vid * 3 + 1], colors[vid * 3 + 2], 1.0) : float4(0, 0, 0, 1);
    if (VCOLOR == 4) vColor = HAS_VC_DATA ? float4(colors[vid * 4], colors[vid * 4 + 1], colors[vid * 4 + 2], colors[vid * 4 + 3]) : float4(0, 0, 0, 1);
    if (INST_COLOR) vColor.xyz *= float3(instCols[iid]);
    // skinning (positions and normals)
    if (SKINNED) {
        float4 si = skinIdx[vid]; float4 sw = skinW[vid];
        float4x4 bx = bones[int(si.x)], by = bones[int(si.y)], bz = bones[int(si.z)], bw = bones[int(si.w)];
        float4x4 skinMatrix = sw.x * bx + sw.y * by + sw.z * bz + sw.w * bw;
        skinMatrix = O.bindMatrixInv * skinMatrix * O.bindMatrix;
        objectNormal = (skinMatrix * float4(objectNormal, 0.0)).xyz;
        float4 skinVertex = O.bindMatrix * float4(transformed, 1.0);
        float4 skinned = bx * skinVertex * sw.x + by * skinVertex * sw.y + bz * skinVertex * sw.z + bw * skinVertex * sw.w;
        transformed = (O.bindMatrixInv * skinned).xyz;
    }
    // House shadows: the body boils out along its normals in the dark
    o.extra = float4(0);
    o.smokeP = restPos;
    if (PATCH == 5 || PATCH == 6) {
        float uTime = M.patch[0].x, uSmoke = M.patch[0].y;
        float3 smN = normalize(objectNormal);
        float wisp = vColor.x;
        if (VCOLOR == 0) wisp = HAS_VC_DATA ? colors[vid * 3] : 0.0;
        float3 smQ = restPos * 11.0 + float3(0.0, -uTime * 2.2, 0.0);
        float smB = sin(smQ.x + 1.7 * sin(smQ.y * 0.7 + uTime * 1.9)) * sin(smQ.y * 0.8 + 1.3 * sin(smQ.z * 1.3 - uTime * 1.4)) * sin(smQ.z * 0.9 + smQ.x * 0.4 + uTime * 1.1);
        transformed += smN * (uSmoke * (0.004 + 0.02 * wisp) * (0.6 + 0.6 * smB) + (PATCH == 6 ? 0.011 * (1.0 - wisp) : 0.0));
        transformed.xz += uSmoke * wisp * wisp * 0.03 * float2(sin(uTime * 2.3 + restPos.y * 9.0), cos(uTime * 1.9 + restPos.y * 7.0));
        float4 smV = O.modelView * float4(transformed, 1.0);
        float3x3 nm = float3x3(O.normalMatrix[0].xyz, O.normalMatrix[1].xyz, O.normalMatrix[2].xyz);
        o.extra.y = abs(dot(normalize(nm * smN), normalize(-smV.xyz)));
        o.extra.z = (O.model * float4(transformed, 1.0)).y;
        o.extra.x = wisp;
    }
    // project
    float4 mvPosition;
    if (KIND == 4) {
        mvPosition = O.modelView * float4(0.0, 0.0, 0.0, 1.0);
        float2 scale = float2(length(O.model[0].xyz), length(O.model[1].xyz));
        if (!SIZE_ATTEN) scale *= -mvPosition.z;
        float2 aligned = (transformed.xy - (O.spriteCenterRot.xy - float2(0.5))) * scale;
        float r = O.spriteCenterRot.z;
        mvPosition.xy += float2(cos(r) * aligned.x - sin(r) * aligned.y, sin(r) * aligned.x + cos(r) * aligned.y);
    } else if (INSTANCED) {
        mvPosition = O.modelView * (instMats[iid] * float4(transformed, 1.0));
    } else {
        mvPosition = O.modelView * float4(transformed, 1.0);
    }
    // beyond a window: slid along the eye ray onto a thin slab in front of
    // the window's plane (nearer things a little nearer), same place on screen
    o.portal = 0.0;
    o.portalDist = length(mvPosition.xyz);
    o.fogDepth = -mvPosition.z;
    if (PORTAL) {
        float3 pc = (F.view * float4(M.portal[0].xyz, 1.0)).xyz;
        float3 pn = (F.view * float4(M.portal[2].xyz, 0.0)).xyz;
        float sP = dot(mvPosition.xyz - pc, pn);
        float sE = dot(-pc, pn);
        if (sP < 0.0 && sE > 0.0) {
            float t = dot(pc, pn) / dot(mvPosition.xyz, pn);
            // the slab: `portal[2].w` metres in front of the plane at most,
            // nearer things nearer (by the log of the distance, 0.3-300 m)
            float dp = t * o.portalDist;
            float near = 1.0 - saturate(log(max(o.portalDist, 0.3) / 0.3) / 6.9077553);
            mvPosition.xyz *= t * (1.0 - M.portal[2].w * near / max(dp, 0.05));
            o.portal = 1.0;
        }
    }
    o.position = F.proj * mvPosition;
    o.viewPos = -mvPosition.xyz;
    o.pointSize = 1.0;
    if (KIND == 3) {
        o.pointSize = M.misc.y;
        if (SIZE_ATTEN) o.pointSize *= F.viewport.z / -mvPosition.z;
    }
    // normals (instance matrix, then normal matrix)
    float3 tn = objectNormal;
    if (INSTANCED) {
        float3x3 im = float3x3(instMats[iid][0].xyz, instMats[iid][1].xyz, instMats[iid][2].xyz);
        tn /= float3(dot(im[0], im[0]), dot(im[1], im[1]), dot(im[2], im[2]));
        tn = im * tn;
    }
    float3x3 nm = float3x3(O.normalMatrix[0].xyz, O.normalMatrix[1].xyz, O.normalMatrix[2].xyz);
    tn = nm * tn;
    if (FLIP_SIDED) tn = -tn;
    o.normal = normalize(tn);
    // uv (through the map's uv transform), or the world-space architecture UVs
    float3 mt0 = M.mapT0.xyz, mt1 = M.mapT1.xyz;
    o.uv = float2(dot(mt0, float3(uv, 1.0)), dot(mt1, float3(uv, 1.0)));
    if (PATCH == 1 && HAS_MAP && INSTANCED) {
        float4x4 imx = instMats[iid];
        float4 awp = O.model * imx * float4(restPos, 1.0);
        float3 an = normalize(float3x3(imx[0].xyz, imx[1].xyz, imx[2].xyz) * restNrm);
        float topY = imx[3].y + 0.5 * length(imx[1].xyz);
        float vsc = M.arch.z > 0.5 ? max(M.arch.y, topY) : M.arch.y;
        float2 wuv;
        if (abs(an.y) > 0.5) wuv = awp.xz / M.arch.x + M.arch.w;
        else if (abs(an.x) > 0.5) wuv = float2((an.x > 0.0 ? -awp.z : awp.z) / M.arch.x, awp.y / vsc);
        else wuv = float2((an.z > 0.0 ? awp.x : -awp.x) / M.arch.x, awp.y / vsc);
        o.uv = wuv;
    }
    o.color = vColor;
    // world position for the light map: mvPosition with the view's translation removed, rotated back
    o.llPos = (float4(mvPosition.xyz - F.view[3].xyz, 0.0) * F.view).xyz;
    o.detPos = restPos;
    o.detNrm = restNrm;
    return o;
}

// ------------------------------------------------------------ fragment
struct FIn {
    float4 position [[position]];
    float3 viewPos;
    float3 normal;
    float2 uv;
    float4 color;
    float  fogDepth;
    float3 llPos;
    float3 detPos;
    float3 detNrm;
    float4 extra;
    float3 smokeP;
    float  portal;
    float  portalDist;
};

static inline float llLit(float3 p, constant Frame& F, texture2d<float> llMap, sampler llS) {
    return llMap.sample(llS, p.xz * F.llGrid.xy + F.llGrid.zw).r * F.llParams.y;
}
// the light map at p, all of it: r the light, g how open the place is (the
// nearest wall that reaches its ceiling, 0.8 m = 1), b its ceiling (10 m = 1)
static inline float4 llAt(float3 p, constant Frame& F, texture2d<float> llMap, sampler llS) {
    return llMap.sample(llS, p.xz * F.llGrid.xy + F.llGrid.zw);
}
static inline float detH(float3 p, float3 detNrm, float scale, texture2d<float> bump, sampler bs) {
    float3 w = pow(abs(normalize(detNrm)), float3(4.0));
    w /= (w.x + w.y + w.z);
    p *= scale;
    return bump.sample(bs, p.yz).x * w.x + bump.sample(bs, p.xz).x * w.y + bump.sample(bs, p.xy).x * w.z;
}

// a window's view of an outside that is not there (Patch::WindowView): a
// photograph laid on a ground disc and a dome round the point it was taken
// from, just outside the glass (three.js's GroundedSkybox, traced per
// pixel): the near ground moves with your head, the far land does not. The
// map holds half the turn round the way ahead (u: -90..90 deg), from 55 deg
// above the horizon (v = 0) to 80 deg below it (v = 1). A lamp outside
// pools its light on the ground and haloes in the fog.
// patch[0] = the point (world) and its height over the ground;
// patch[1] = the way ahead (x, z, unit), the dome's radius, the exposure;
// patch[2] = the lamp's head (world) and its strength;
// patch[3] = x how far into the night it is (0 dusk .. 1 night)
static inline float3 windowView(float3 P, constant Frame& F, constant Material& M, texture2d<float> map, sampler mapS) {
    float3 eye = (float4(-F.view[3].xyz, 0.0) * F.view).xyz;
    float3 rd = normalize(P - eye);
    float3 O = M.patch[0].xyz;
    float yg = O.y - M.patch[0].w;
    float R = M.patch[1].z;
    float3 oc = eye - O;
    float b = dot(oc, rd);
    float t = -b + sqrt(max(b * b - (dot(oc, oc) - R * R), 0.0));
    float3 hit = eye + rd * t;
    bool ground = false;
    if (hit.y < yg && rd.y < 0.0) {
        hit = eye + rd * ((yg - eye.y) / rd.y);
        ground = true;
    }
    float3 d = normalize(hit - O);
    float2 fw = M.patch[1].xy;
    float az = atan2(d.x * -fw.y + d.z * fw.x, d.x * fw.x + d.z * fw.y);
    float el = asin(clamp(d.y, -1.0, 1.0));
    float u = clamp(0.5 + az * 0.31830989, 0.0, 1.0);
    float v = clamp((0.95993109 - el) / 2.35619449, 0.0, 1.0);
    float3 c = map.sample(mapS, float2(u, v)).rgb;
    float night = M.patch[3].x;
    // night: what light is left goes, and the colour with it
    float lumc = dot(c, float3(0.2126, 0.7152, 0.0722));
    c = mix(c, float3(lumc) * float3(0.85, 0.95, 1.2), 0.5 * night);
    c *= M.patch[1].w;
    // the lamp: its pool on the ground (cosine over distance squared) and
    // its halo in the fog round its head
    float3 L = M.patch[2].xyz;
    float ls = M.patch[2].w;
    float3 sodium = float3(1.0, 0.42, 0.10);
    if (ground && ls > 0.0) {
        float3 tl = L - hit;
        float dl = length(tl);
        float E = max(tl.y, 0.0) / (dl * dl * dl);
        c += map.sample(mapS, float2(u, v)).rgb * sodium * ls * E * 6.0;
    }
    if (ls > 0.0) {
        float3 tl = L - eye;
        float cosA = dot(rd, normalize(tl));
        float a = acos(clamp(cosA, -1.0, 1.0));
        float far = length(tl);
        float halo = 0.06 / (1.0 + pow(a * far / 0.9, 2.0)) + 0.012 * exp(-a * 6.0);
        c += sodium * ls * halo * (0.6 + 0.8 * night);
    }
    return c;
}

fragment float4 fs_main(FIn in [[stage_in]], bool front [[front_facing]], float2 pc [[point_coord]],
                        constant Frame& F [[buffer(0)]],
                        constant Material& M [[buffer(2)]],
                        constant Hemi* hemis [[buffer(3)]],
                        const device Point* points [[buffer(4)]],
                        const device Spot* spots [[buffer(5)]],
                        constant LightList& LLst [[buffer(6)]],
                        texture2d<float> map [[texture(0)]],
                        texture2d<float> llMap [[texture(1)]],
                        texture2d<float> bump [[texture(2)]],
                        sampler mapS [[sampler(0)]],
                        sampler llS [[sampler(1)]],
                        sampler bumpS [[sampler(2)]])
{
    if (PATCH == 8) return float4(srgbOETF(windowView(in.llPos, F, M, map, mapS)), 1.0);
    float4 diffuseColor = M.diffuse;
    // diffuse map (points sample it at the point coordinate)
    if (HAS_MAP) {
        float2 uv = in.uv;
        if (KIND == 3) {
            float2 p = float2(pc.x, 1.0 - pc.y);
            uv = float2(dot(M.mapT0.xyz, float3(p, 1.0)), dot(M.mapT1.xyz, float3(p, 1.0)));
        }
        if (M.misc.w > 0.5) uv.y = 1.0 - uv.y; // flip-y textures
        diffuseColor *= map.sample(mapS, uv);
    }
    // vertex / instance colours
    if (VCOLOR == 3 || INST_COLOR) diffuseColor.rgb *= in.color.rgb;
    if (VCOLOR == 4) diffuseColor *= in.color;
    // House shadows: smoke in the dark, a hard figure in the light
    if (PATCH == 5 || PATCH == 6) {
        float uTime = M.patch[0].x, uSmoke = M.patch[0].y;
        float wisp = in.extra.x, rim = in.extra.y, floorY = in.extra.z;
        float smFloor = mix(1.0, smoothstep(0.02, 0.4, floorY), uSmoke);
        float smWisp = mix(1.0 - smoothstep(0.0, 0.3, wisp), 1.0 - 0.85 * wisp, uSmoke);
        if (PATCH == 6) diffuseColor.a *= smWisp * smFloor;
        else {
            float3 sp = in.smokeP;
            float smDrift = 0.6 + 0.4 * sin(sp.x * 21.0 + uTime * 1.3 + 2.0 * sin(sp.y * 13.0 - uTime * 2.1)) * sin(sp.y * 17.0 + sp.z * 9.0 - uTime * 1.6);
            diffuseColor.a *= mix(1.0, smoothstep(0.04, 0.6, rim) * smDrift, uSmoke) * smFloor * smWisp;
        }
    }
    if (ALPHA_TEST && diffuseColor.a < M.misc.x) discard_fragment();
    // beyond a window: only what is seen through its opening
    bool beyond = PORTAL && in.portal > 0.5;
    if (beyond) {
        float3 pw = in.llPos - M.portal[0].xyz;
        if (abs(dot(pw, M.portal[1].xyz)) > M.portal[0].w || abs(dot(pw - dot(pw, M.portal[2].xyz) * M.portal[2].xyz, normalize(cross(M.portal[2].xyz, M.portal[1].xyz)))) > M.portal[1].w) discard_fragment();
    }

    float3 outgoing;
    if (KIND == 1 || KIND == 2) {
        float3 totalEmissive = M.emissive.rgb;
        if (PATCH == 7) totalEmissive *= in.color.rgb; // Stairwell face: emissive × painted colour
        // shading normal (flat: from screen-space derivatives)
        float faceDirection = front ? 1.0 : -1.0;
        float3 normal;
        if (FLAT) {
            float3 fdx = dfdx(in.viewPos), fdy = dfdy(in.viewPos);
            normal = normalize(cross(fdx, fdy));
        } else {
            normal = normalize(in.normal);
            if (DOUBLE_SIDE) normal *= faceDirection;
        }
        float specularStrength = 1.0;
        // glass: how much dirt is on it, from its map's grey (Patch::Glass)
        float glassDirt = 0.0;
        if (PATCH == 9) {
            glassDirt = saturate((HAS_MAP ? diffuseColor.r : 1.0) * M.patch[2].x);
            diffuseColor.rgb = M.patch[2].yzw;
        }
        // skin detail: triplanar bump at the rest-pose position
        if ((PATCH == 2 || PATCH == 4) && HAS_BUMP) {
            float sc = M.patch[0].x;
            float H = detH(in.detPos, in.detNrm, sc, bump, bumpS);
            float2 dHdxy = M.misc.z * float2(detH(in.detPos + dfdx(in.detPos), in.detNrm, sc, bump, bumpS) - H,
                                             detH(in.detPos + dfdy(in.detPos), in.detNrm, sc, bump, bumpS) - H);
            float3 surf = -in.viewPos;
            float3 sX = normalize(dfdx(surf)), sY = normalize(dfdy(surf));
            float3 R1 = cross(sY, normal), R2 = cross(normal, sX);
            float fDet = dot(sX, R1) * faceDirection;
            float3 grad = sign(fDet) * (dHdxy.x * R1 + dHdxy.y * R2);
            normal = normalize(abs(fDet) * normal - grad);
            specularStrength = mix(0.3, 1.15, smoothstep(0.42, 0.6, detH(in.detPos * 0.29, in.detNrm, sc, bump, bumpS)));
        }
        // a photographed detail over walls, floors and ceilings (ArchDetail):
        // a grey height map laid flat on the face in world space, shading the
        // colour and lit as relief; the relief fades as its texels shrink
        // under a pixel, the shading as the mipmaps grey it out
        float detK = 1.0;
        if (PATCH == 1 && HAS_BUMP) {
            float3 an = abs(inverseTransformDirection(normal, F.view));
            float3 P = in.llPos;
            float2 duv = (an.y > 0.5 ? P.xz : (an.x > an.z ? P.zy : P.xy)) * M.patch[0].x;
            float H = bump.sample(bumpS, duv).r;
            float2 dHdxy = float2(dfdx(H), dfdy(H));
            detK = 1.0 + M.patch[0].y * (H - 0.5) * 2.0;
            float rel = M.patch[0].z * (1.0 - smoothstep(0.6, 2.0, length(fwidth(duv)) * float(bump.get_width())));
            float3 surf = -in.viewPos;
            float3 sX = normalize(dfdx(surf)), sY = normalize(dfdy(surf));
            float3 R1 = cross(sY, normal), R2 = cross(normal, sX);
            float fDet = dot(sX, R1) * faceDirection;
            float3 grad = sign(fDet) * (rel * dHdxy.x * R1 + rel * dHdxy.y * R2);
            normal = normalize(abs(fDet) * normal - grad);
        }
        if (PATCH == 3) {
            float smEyeF = clamp(dot(normal, normalize(in.viewPos)), 0.0, 1.0);
            totalEmissive *= mix(float3(0.62, 0.64, 0.7), float3(1.0), pow(smEyeF, 1.2));
        }
        float3 geometryPosition = -in.viewPos;
        float3 geometryNormal = normal;
        float3 geometryViewDir = normalize(in.viewPos);
        float3 directDiffuse = float3(0), directSpecular = float3(0);
        float3 diff = diffuseColor.rgb * detK;
        float3 wN = inverseTransformDirection(geometryNormal, F.view);
        // wear: three octaves of value noise at the rest-pose position mottle
        // the colour (the finest fades out before it would alias); grime
        // settles in the dark blotches and on the parts near the floor.
        // Textures are reined in here (flat colours were muted when the
        // material was made)
        if (WEAR) {
            if (HAS_MAP) diff = limitChroma(diff);
            float3 q = in.detPos;
            float3 q3 = q * 37.0 + 3.7;
            float fine = 1.0 - saturate(length(fwidth(q3)) * 0.7);
            float n = 0.5 * vnoise(q * 3.5) + 0.3 * vnoise(q * 11.0 + 7.31) + 0.2 * mix(0.5, vnoise(q3), fine);
            float m = clamp((n - 0.5) * 3.2, -1.0, 1.0);
            float w = M.surf.x;
            diff *= 1.0 + w * 0.14 * m;
            float low = (1.0 - smoothstep(0.02, 0.55, in.llPos.y)) * smoothstep(-0.3, 0.0, in.llPos.y); // not down in a pool
            float grime = w * saturate(0.3 * saturate(-m) + low * (0.45 + 0.3 * saturate(-m)));
            diff *= mix(float3(1.0), float3(0.74, 0.70, 0.62), grime);
        }
        float3 spec = M.specular.rgb; float shin = M.specular.a;
        for (uint li = 0; li < (beyond ? 0u : LLst.count); li++) {
            uint k = LLst.idx[li];
            float3 Ldir, Lcol;
            if (k < 32768u) {
                Point p = points[k];
                float3 lv = p.pos.xyz - geometryPosition;
                Ldir = normalize(lv);
                Lcol = p.color.rgb * distanceAttenuation(length(lv), p.pos.w, p.color.w);
            } else {
                Spot s = spots[k - 32768u];
                float3 lv = s.pos.xyz - geometryPosition;
                Ldir = normalize(lv);
                float angleCos = dot(Ldir, s.dir.xyz);
                float att = smoothstep(s.cone.x, s.cone.y, angleCos);
                if (att <= 0.0) continue;
                Lcol = s.color.rgb * att * distanceAttenuation(length(lv), s.pos.w, s.dir.w);
            }
            float dotNL = saturate(dot(geometryNormal, Ldir));
            float3 irr = dotNL * Lcol;
            directDiffuse += irr * BRDF_Lambert(diff);
            if (KIND == 2) directSpecular += irr * BRDF_BlinnPhong(Ldir, geometryViewDir, geometryNormal, spec, shin) * specularStrength;
        }
        // indirect: ambient + hemisphere (+ the light map's dark end)
        float3 irradiance = F.ambient.rgb;
        for (uint h = 0; h < F.counts.x; h++) {
            float dotNL = dot(geometryNormal, hemis[h].dir.xyz);
            irradiance += mix(hemis[h].ground.rgb, hemis[h].sky.rgb, 0.5 * dotNL + 0.5);
        }
        // corners: the place 0.8 m out from this surface, where its indirect
        // light comes from, has another wall nearer than that in a corner,
        // beside a wall for a floor or a ceiling, and the ceiling over a wall
        // just above it; a plain wall's own face is that far off
        float aoCorner = 1.0;
        if (beyond) {
            // the view's own light: its sky over, a darker ground under
            irradiance = mix(M.portal[3].rgb * 0.3, M.portal[3].rgb, 0.5 * wN.y + 0.5);
        } else if (LL && F.llParams.x > 0.5) {
            float4 llO = llAt(in.llPos + wN * 0.8, F, llMap, llS);
            float llK = llO.r * F.llParams.y;
            irradiance += (1.0 - llK) * (F.llAmbD.rgb + mix(F.llGndD.rgb, F.llSkyD.rgb, 0.5 * wN.y + 0.5));
            if (PATCH <= 2 && F.look.y > 0.0) {
                float open = smoothstep(0.0, 1.0, llO.g);
                float under = 1.0 - smoothstep(0.3, 0.8, abs(wN.y));
                float toCeil = llO.b * 10.0 - in.llPos.y;
                float top = mix(1.0, smoothstep(-0.05, 0.6, toCeil), under);
                aoCorner = mix(1.0, mix(0.45, 1.0, open) * mix(0.6, 1.0, top), F.look.y);
            }
        }
        // floor occlusion: walls and the sides of things lose indirect light
        // toward the floor (faces turned up keep it)
        float ao = 1.0;
        if (PATCH <= 2) {
            ao = mix(0.55, 1.0, smoothstep(0.0, 0.45, in.llPos.y));
            ao = mix(1.0, ao, F.look.x * (1.0 - smoothstep(0.3, 0.8, wN.y)));
        }
        float3 indirectDiffuse = irradiance * BRDF_Lambert(diff) * ao * aoCorner;
        outgoing = directDiffuse + indirectDiffuse + directSpecular + totalEmissive;
        // a pane of glass (Patch::Glass): the room seen in it by Fresnel (two
        // faces, 4% each square on, all of it at a graze), the dust and smears
        // on it (the map's alpha) lit by the room and, from behind, by the
        // view; a breath's mist on its far face (patch[1]). patch[0]: x how
        // bright the room reads in it, yzw the light of the view behind (linear)
        if (PATCH == 9) {
            float cosT = saturate(abs(dot(geometryNormal, geometryViewDir)));
            float fr = M.patch[3].y + (1.0 - M.patch[3].y) * pow(1.0 - cosT, 5.0);
            float dirt = glassDirt;
            float2 dm = (in.uv - M.patch[1].xy) / max(M.patch[1].z, 1e-3);
            float mist = M.patch[1].w * (1.0 - smoothstep(0.35, 1.0, length(dm * float2(1.0, 1.3))));
            float milk = M.patch[3].x;
            float3 room = irradiance * M.patch[0].x;
            float3 back = M.patch[0].yzw;
            float3 dust = directDiffuse + indirectDiffuse + back * diff * 0.6;
            float a = saturate(dirt + milk + mist * 0.7 + fr * (1.0 - dirt));
            float3 prem = dust * dirt + (back * 0.8 + room * 0.3) * milk + back * mist * 0.55 + room * fr * (1.0 - dirt) + directSpecular;
            outgoing = prem / max(a, 1e-3);
            diffuseColor.a = a;
        }
        if (PATCH == 4) {
            float3 over = max(outgoing - 0.5, 0.0);
            outgoing = min(outgoing, 0.5) + 0.143 * log(1.0 + over / 0.143);
        } else if (PATCH <= 2 && M.surf.y > 0.5) {
            outgoing = filmShoulder(outgoing);
        }
    } else {
        outgoing = diffuseColor.rgb;
    }
    if (OPAQUE) diffuseColor.a = 1.0;
    float4 col = float4(srgbOETF(outgoing), diffuseColor.a);
    // exponential-squared fog, applied after the sRGB encode, with the local light patch
    if (beyond) {
        float hz = 1.0 - exp(-M.portal[3].w * in.portalDist);
        col.rgb = mix(col.rgb, srgbOETF(M.portal[3].rgb * 0.6), hz);
    } else if (USE_FOG) {
        float llFogK = (LL && F.llParams.x > 0.5) ? llLit(in.llPos, F, llMap, llS) : 0.0;
        float d = F.fog.a * ((LL) ? mix(1.0, F.llParams.z, llFogK) : 1.0);
        float fogFactor = 1.0 - exp(-d * d * in.fogDepth * in.fogDepth);
        float3 fogCol = (LL) ? mix(F.fog.rgb, F.llFogLit.rgb, llFogK) : F.fog.rgb;
        col.rgb = mix(col.rgb, fogCol, fogFactor);
    }
    return col;
}

// ------------------------------------------------------------ composite (scene + overlay to the drawable)
struct QOut { float4 position [[position]]; float2 uv; };
vertex QOut vs_quad(uint vid [[vertex_id]], constant float4& rect [[buffer(0)]]) {
    float2 q = float2((vid & 1) ? 1.0 : 0.0, (vid & 2) ? 1.0 : 0.0);
    QOut o;
    float2 p = rect.xy + q * rect.zw; // in 0..1 screen space, y down
    o.position = float4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
    o.uv = q;
    return o;
}
fragment float4 fs_blit(QOut in [[stage_in]], texture2d<float> t [[texture(0)]], sampler s [[sampler(0)]], constant float4& tint [[buffer(0)]]) {
    return t.sample(s, in.uv) * tint;
}
