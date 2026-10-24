// Composite pass: the resolved 3D view, then the HUD and the page's CSS layers
// (vignette, tints, flashes, grain, fade) with CSS mix-blend-mode semantics.
// Apple GPUs read the framebuffer in the fragment shader ([[color(0)]]), so
// every blend mode is computed exactly instead of approximated.
#include <metal_stdlib>
using namespace metal;

struct QOut { float4 position [[position]]; float2 uv; };
vertex QOut vs_quad(uint vid [[vertex_id]], constant float4& rect [[buffer(0)]]) {
    float2 q = float2((vid & 1) ? 1.0 : 0.0, (vid & 2) ? 1.0 : 0.0);
    QOut o;
    float2 p = rect.xy + q * rect.zw;
    o.position = float4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
    o.uv = q;
    return o;
}

struct P { float4 a, b, c, d, e, f, g, h, i, j; };

// Separable blend of a straight-alpha source colour over the destination.
static float3 blendf(int mode, float3 cb, float3 cs) {
    switch (mode) {
        case 1: return cb * cs;                                   // multiply
        case 2: return cb + cs - cb * cs;                         // screen
        case 3: return min(cb + cs, float3(1.0));                 // plus-lighter
        case 4: return select(1.0 - 2.0 * (1.0 - cb) * (1.0 - cs), 2.0 * cb * cs, cb <= 0.5); // overlay
        case 5: {                                                 // soft-light
            float3 d = select(sqrt(cb), ((16.0 * cb - 12.0) * cb + 4.0) * cb, cb <= 0.25);
            return select(cb + (2.0 * cs - 1.0) * (d - cb), cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb), cs <= 0.5);
        }
        case 6: return select(min(float3(1.0), cb / max(1.0 - cs, 1e-5)), float3(1.0), cs >= 1.0); // color-dodge
        case 7: return min(cb, cs);
        case 8: return max(cb, cs);
        default: return cs;
    }
}
static float4 over(int mode, float4 dst, float3 cs, float as) {
    float3 mixed = blendf(mode, dst.rgb, cs);
    return float4(mix(dst.rgb, mixed, as), 1.0);
}

// the 3D view, through the canvas's CSS filter chain (saturate / hue-rotate /
// contrast / brightness as affine colour matrices, each clamped)
struct SceneFilter { float4 n; float4 m[12]; };
fragment float4 fs_scene(QOut in [[stage_in]], texture2d<float> t [[texture(0)]], sampler s [[sampler(0)]], constant SceneFilter& f [[buffer(0)]]) {
    float3 c = t.sample(s, in.uv).rgb;
    int n = int(f.n.x);
    for (int k = 0; k < n; k++) {
        float4 c4 = float4(c, 1.0);
        c = clamp(float3(dot(f.m[k * 3], c4), dot(f.m[k * 3 + 1], c4), dot(f.m[k * 3 + 2], c4)), 0.0, 1.0);
    }
    return float4(c, 1.0);
}
// a premultiplied image (HUD canvas, grain)
fragment float4 fs_image(QOut in [[stage_in]], float4 dst [[color(0)]], texture2d<float> t [[texture(0)]], sampler s [[sampler(0)]], constant P& p [[buffer(0)]]) {
    float2 uv = in.uv * p.a.zw + p.b.xy;
    float4 c = t.sample(s, uv);
    float a = c.a * p.a.x;
    float3 straight = c.a > 0.0 ? c.rgb / c.a : float3(0.0);
    return over(int(p.a.y), dst, straight, a);
}
fragment float4 fs_fill(QOut in [[stage_in]], float4 dst [[color(0)]], constant P& p [[buffer(0)]]) {
    return over(int(p.b.x), dst, p.a.rgb, p.a.a);
}
fragment float4 fs_radial(QOut in [[stage_in]], float4 dst [[color(0)]], constant P& p [[buffer(0)]]) {
    float2 d = (in.uv - p.a.xy) / max(p.a.zw, float2(1e-5));
    float t = length(d);
    int n = int(p.b.y);
    float4 cs[4] = { p.d, p.e, p.f, p.g };
    float ts[4] = { p.c.x, p.c.y, p.c.z, p.c.w };
    float4 col = cs[0];
    if (t >= ts[n - 1]) col = cs[n - 1];
    else {
        for (int k = 0; k < n - 1; k++) {
            if (t >= ts[k] && t < ts[k + 1]) {
                float f = (t - ts[k]) / max(ts[k + 1] - ts[k], 1e-5);
                // CSS interpolates gradients in premultiplied space
                float4 a = float4(cs[k].rgb * cs[k].a, cs[k].a), b = float4(cs[k + 1].rgb * cs[k + 1].a, cs[k + 1].a);
                float4 m = mix(a, b, f);
                col = float4(m.a > 0.0 ? m.rgb / m.a : float3(0.0), m.a);
                break;
            }
        }
    }
    return over(int(p.b.x), dst, col.rgb, col.a * p.b.z);
}
fragment float4 fs_aberration(QOut in [[stage_in]], float4 dst [[color(0)]], texture2d<float> t [[texture(0)]], sampler s [[sampler(0)]], constant P& p [[buffer(0)]]) {
    float o = p.a.x;
    float r = t.sample(s, in.uv + float2(o, 0.0)).r;
    float b = t.sample(s, in.uv - float2(o, 0.0)).b;
    float3 shifted = float3(r, dst.g, b);
    return float4(mix(dst.rgb, shifted, p.a.y), 1.0);
}
