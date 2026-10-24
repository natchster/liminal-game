// The game's renderer: three.js r160 MeshBasic / MeshLambert / MeshPhong /
// Points / Sprite / LineBasic shading, chunk for chunk, plus the patches the
// game installs with onBeforeCompile (local light map, world-space wall UVs,
// creature skin detail, the Smiler's eyes, the Watcher's sheet, the House
// shadows, the Stairwell face). Variants are function constants.
#include <metal_stdlib>
using namespace metal;

// ------------------------------------------------------------ variants
constant int  KIND        [[function_constant(0)]];   // 0 basic 1 lambert 2 phong 3 points 4 sprite 5 line
constant bool HAS_MAP     [[function_constant(1)]];
constant int  VCOLOR      [[function_constant(2)]];   // 0 none, 3 rgb, 4 rgba
constant bool INSTANCED   [[function_constant(3)]];
constant bool INST_COLOR  [[function_constant(4)]];
constant bool SKINNED     [[function_constant(5)]];
constant int  MORPHS      [[function_constant(6)]];   // morph target count (0..8)
constant bool USE_FOG     [[function_constant(7)]];
constant bool ALPHA_TEST  [[function_constant(8)]];
constant bool DOUBLE_SIDE [[function_constant(9)]];
constant bool FLIP_SIDED  [[function_constant(10)]];
constant bool FLAT        [[function_constant(11)]];
constant bool LL          [[function_constant(12)]];  // localLightPatch
constant int  PATCH       [[function_constant(13)]];  // 0 none 1 archUV 2 skinDetail 3 smilerEye 4 watcher 5 shadowBody 6 shadowEdge 7 stairFace
constant bool HAS_NORMAL  [[function_constant(14)]];
constant bool HAS_UV      [[function_constant(15)]];
constant bool OPAQUE      [[function_constant(16)]];
constant bool HAS_BUMP    [[function_constant(17)]];
constant bool MORPH_NRM   [[function_constant(18)]];
constant bool MORPH_REL   [[function_constant(19)]];
constant bool SIZE_ATTEN  [[function_constant(20)]];
constant bool HAS_VC_DATA [[function_constant(21)]];  // geometry actually has colours

// ------------------------------------------------------------ uniforms
struct Frame {
    float4x4 view;
    float4x4 proj;
    float4 ambient;            // rgb: sum of ambient lights
    float4 fog;                // rgb colour (linear, three uploads it as-is), a: density
    float4 llGrid;
    float4 llAmbD;             // rgb
    float4 llSkyD;
    float4 llGndD;
    float4 llFogLit;
    float4 llParams;           // x on, y scale, z fogRatio, w time
    float4 viewport;           // x w, y h (pixels), z point scale (h/2), w unused
    uint4  counts;             // x hemi, y point, z spot
};
struct Hemi  { float4 sky; float4 ground; float4 dir; };
struct Point { float4 pos; float4 color; };          // pos.w distance, color.w decay
struct Spot  { float4 pos; float4 dir; float4 color; float4 cone; }; // pos.w distance, dir.w decay, cone.x coneCos, cone.y penumbraCos

struct Object {
    float4x4 model;
    float4x4 modelView;
    float4x4 normalMatrix;     // 3x3 in a 4x4
    float4x4 bindMatrix;
    float4x4 bindMatrixInv;
    float4   morph[2];         // up to 8 influences
    float4   spriteCenterRot;  // xy center, z rotation
};
struct Material {
    float4 diffuse;            // rgb colour, a opacity
    float4 emissive;           // rgb (× intensity)
    float4 specular;           // rgb, a shininess
    float4 mapT0;              // uv transform rows: (a, b, c) ...
    float4 mapT1;
    float4 misc;               // x alphaTest, y size, z bumpScale, w mapFlipY
    float4 arch;               // archUV: x scale, y vscale, z wallpaper
    float4 patch[4];           // patch parameters
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

// ------------------------------------------------------------ vertex
struct VOut {
    float4 position [[position]];
    float  pointSize [[point_size]];
    float3 viewPos;            // vViewPosition (= -mvPosition)
    float3 normal;
    float2 uv;
    float4 color;
    float  fogDepth;
    float3 llPos;
    float3 detPos;
    float3 detNrm;
    float4 extra;              // shadow: x wisp, y rim, z floorY
    float3 smokeP;
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
    // morph targets (three: relative adds, absolute blends with the base)
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
    // vertex colours (color_vertex)
    float4 vColor = float4(1.0);
    if (VCOLOR == 3) vColor = HAS_VC_DATA ? float4(colors[vid * 3], colors[vid * 3 + 1], colors[vid * 3 + 2], 1.0) : float4(0, 0, 0, 1);
    if (VCOLOR == 4) vColor = HAS_VC_DATA ? float4(colors[vid * 4], colors[vid * 4 + 1], colors[vid * 4 + 2], colors[vid * 4 + 3]) : float4(0, 0, 0, 1);
    if (INST_COLOR) vColor.xyz *= float3(instCols[iid]);
    // skinning (skinbase/skinnormal/skinning_vertex)
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
    o.position = F.proj * mvPosition;
    o.viewPos = -mvPosition.xyz;
    o.fogDepth = -mvPosition.z;
    o.pointSize = 1.0;
    if (KIND == 3) {
        o.pointSize = M.misc.y;
        if (SIZE_ATTEN) o.pointSize *= F.viewport.z / -mvPosition.z;
    }
    // normals (defaultnormal_vertex)
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
    // uv (uv_vertex), or the world-space architecture UVs
    float3 mt0 = M.mapT0.xyz, mt1 = M.mapT1.xyz;
    o.uv = float2(dot(mt0, float3(uv, 1.0)), dot(mt1, float3(uv, 1.0)));
    if (PATCH == 1 && HAS_MAP && INSTANCED) {
        float4x4 imx = instMats[iid];
        float4 awp = O.model * imx * float4(restPos, 1.0);
        float3 an = normalize(float3x3(imx[0].xyz, imx[1].xyz, imx[2].xyz) * restNrm);
        float topY = imx[3].y + 0.5 * length(imx[1].xyz);
        float vsc = M.arch.z > 0.5 ? max(M.arch.y, topY) : M.arch.y;
        float2 wuv;
        if (abs(an.y) > 0.5) wuv = awp.xz / M.arch.x;
        else if (abs(an.x) > 0.5) wuv = float2((an.x > 0.0 ? -awp.z : awp.z) / M.arch.x, awp.y / vsc);
        else wuv = float2((an.z > 0.0 ? awp.x : -awp.x) / M.arch.x, awp.y / vsc);
        o.uv = wuv;
    }
    o.color = vColor;
    // world position for the light map: three's mvPosition with the view's translation removed, rotated back
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
};

static inline float llLit(float3 p, constant Frame& F, texture2d<float> llMap, sampler llS) {
    return llMap.sample(llS, p.xz * F.llGrid.xy + F.llGrid.zw).r * F.llParams.y;
}
static inline float detH(float3 p, float3 detNrm, float scale, texture2d<float> bump, sampler bs) {
    float3 w = pow(abs(normalize(detNrm)), float3(4.0));
    w /= (w.x + w.y + w.z);
    p *= scale;
    return bump.sample(bs, p.yz).x * w.x + bump.sample(bs, p.xz).x * w.y + bump.sample(bs, p.xy).x * w.z;
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
    float4 diffuseColor = M.diffuse;
    // map_fragment / map_particle_fragment
    if (HAS_MAP) {
        float2 uv = in.uv;
        if (KIND == 3) {
            float2 p = float2(pc.x, 1.0 - pc.y);
            uv = float2(dot(M.mapT0.xyz, float3(p, 1.0)), dot(M.mapT1.xyz, float3(p, 1.0)));
        }
        if (M.misc.w > 0.5) uv.y = 1.0 - uv.y;   // flipY textures
        diffuseColor *= map.sample(mapS, uv);
    }
    // color_fragment
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

    float3 outgoing;
    if (KIND == 1 || KIND == 2) {
        float3 totalEmissive = M.emissive.rgb;
        if (PATCH == 7) totalEmissive *= in.color.rgb;          // Stairwell face: emissive × painted colour
        // normal_fragment_begin
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
        if (PATCH == 3) {
            float smEyeF = clamp(dot(normal, normalize(in.viewPos)), 0.0, 1.0);
            totalEmissive *= mix(float3(0.62, 0.64, 0.7), float3(1.0), pow(smEyeF, 1.2));
        }
        float3 geometryPosition = -in.viewPos;
        float3 geometryNormal = normal;
        float3 geometryViewDir = normalize(in.viewPos);
        float3 directDiffuse = float3(0), directSpecular = float3(0);
        float3 diff = diffuseColor.rgb;
        float3 spec = M.specular.rgb; float shin = M.specular.a;
        for (uint li = 0; li < LLst.count; li++) {
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
        if (LL && F.llParams.x > 0.5) {
            float3 llN = inverseTransformDirection(geometryNormal, F.view);
            float llK = llLit(in.llPos + llN * 0.8, F, llMap, llS);
            irradiance += (1.0 - llK) * (F.llAmbD.rgb + mix(F.llGndD.rgb, F.llSkyD.rgb, 0.5 * llN.y + 0.5));
        }
        float3 indirectDiffuse = irradiance * BRDF_Lambert(diff);
        outgoing = directDiffuse + indirectDiffuse + directSpecular + totalEmissive;
        if (PATCH == 4) {
            float3 over = max(outgoing - 0.5, 0.0);
            outgoing = min(outgoing, 0.5) + 0.143 * log(1.0 + over / 0.143);
        }
    } else {
        outgoing = diffuseColor.rgb;
    }
    if (OPAQUE) diffuseColor.a = 1.0;
    float4 col = float4(srgbOETF(outgoing), diffuseColor.a);
    // fog_fragment (after the colour-space conversion, as three r160 does), with the local light patch
    if (USE_FOG) {
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
    float2 p = rect.xy + q * rect.zw;              // in 0..1 screen space, y down
    o.position = float4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
    o.uv = q;
    return o;
}
fragment float4 fs_blit(QOut in [[stage_in]], texture2d<float> t [[texture(0)]], sampler s [[sampler(0)]], constant float4& tint [[buffer(0)]]) {
    return t.sample(s, in.uv) * tint;
}
