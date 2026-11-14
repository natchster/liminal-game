//! `place_chestnuts`
use crate::shared::clear_of;
use crate::shared::uv_part;
use game::prelude::*;
use placers_kit::geometry::frame;
use placers_kit::geometry::part;
use placers_kit::noise::fbm;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::paint::mul;
use placers_kit::paint::paint_px;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;
use placers_kit::place::salt_of;

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made)
macro_rules! cached_kind_tex {
    ($n:expr_2021, $k:expr_2021, $make:expr_2021) => {{
        thread_local! { static T: std::cell::RefCell<Vec<Option<Tex>>> = std::cell::RefCell::new(vec![None; $n]); }
        let k = ($k).min($n - 1);
        T.with(|t| {
            if let Some(x) = &t.borrow()[k] {
                return x.clone();
            }
            let tex = $make(k);
            t.borrow_mut()[k] = Some(tex.clone());
            tex
        })
    }};
}

// Chestnut cluster: 5% chance per level. Horse chestnuts (Blender batch
// `t17`): conkers fallen bare, and others still sitting in their husks split
// open into three valves, the husks dried brown; fresh and glossy, dried
// dull or old and wrinkled.
pub(crate) fn place_chestnuts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let cluster = group();
    let count = 2 + (rnd() * 3.0).floor() as i32;
    let out_mat = lambert!({ map: &husk_out_tex() });
    let in_mat = lambert!({ map: &husk_in_tex() });
    let mut down: Vec<(f64, f64, f64)> = vec![];
    let mut placed: Vec<(Obj, f64, f64, bool)> = vec![];
    for _i in 0..count {
        // the old chestnut's draws: its husk (60%: now a husk split open with
        // the conker in it), where it lies, its turn
        let husked = rnd() < 0.60;
        let cx = (rnd() - 0.5) * 0.13;
        let cz = (rnd() - 0.5) * 0.13;
        let yaw = rnd() * PI * 2.0;
        let chestnut = group();
        chestnut.rotation().y = yaw;
        placed.push((chestnut, cx, cz, husked));
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    // spread as they fell (twice the old spacing: a husk lying split open
    // is 8 cm across)
    for (chestnut, cx, cz, husked) in placed {
        let r = if husked { 0.043 } else { 0.017 };
        let (cx, cz) = clear_of((cx * 2.0, cz * 2.0), r, &down);
        down.push((cx, cz, r));
        let salt = salt_of(px + cx, pz + cz);
        let age = (salt % 3) as usize;
        let seed_mat = if age == 0 {
            phong!({ map: &conker_tex(age), specular: 0x262019, shininess: 26 })
        } else {
            lambert!({ map: &conker_tex(age) })
        };
        if husked {
            for (name, mat) in [("Real__conkerHuskOut", &out_mat), ("Real__conkerHuskIn", &in_mat)] {
                if let Some(geo) = part(name) {
                    chestnut.add(&mesh(geo, mat));
                }
            }
            if let Some(geo) = conker_geo("Real__conkerSeated") {
                chestnut.add(&mesh(geo, &seed_mat));
            }
        } else if let Some(geo) = conker_geo("Real__conker") {
            chestnut.add(&mesh(geo, &seed_mat));
        }
        chestnut.position().set(cx, 0, cz);
        cluster.add(&chestnut);
    }
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// ================================================================ conkers
/// The seed's half sizes and its scar's middle (`nature/a_garden_and_a_wood_indoors.py` CK, CK_HILUM)
const CK: [f64; 3] = [0.0155, 0.0120, 0.0140];

const CK_HILUM: [f64; 3] = [0.30, 0.50, 0.81];

/// The scar's middle, a unit direction
fn ck_hilum() -> [f64; 3] {
    let l = (CK_HILUM[0] * CK_HILUM[0] + CK_HILUM[1] * CK_HILUM[1] + CK_HILUM[2] * CK_HILUM[2]).sqrt();
    [CK_HILUM[0] / l, CK_HILUM[1] / l, CK_HILUM[2] / l]
}

/// (u, v) on a conker: round it (u) and up it (v) about its middle, by the
/// direction from its middle scaled to a sphere (as the model's warp)
fn conker_geo(name: &'static str) -> Option<Geo> {
    let (lo, hi) = frame(name);
    // the ellipsoid's middle: its foot flattened, so a little under the box's
    let c = [(lo[0] + hi[0]) / 2.0, lo[1] + CK[1] * (0.45 + 0.55 * 0.45), (lo[2] + hi[2]) / 2.0];
    uv_part(name, true, move |p, _| {
        let d = [(p[0] - c[0]) / CK[0], (p[1] - c[1]) / CK[1], (p[2] - c[2]) / CK[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-9);
        let u = (d[2].datan2(d[0]) / (2.0 * PI) + 1.0) % 1.0;
        let v = (d[1] / l).clamp(-1.0, 1.0).dacos() / PI;
        (u, 1.0 - v)
    })
}

/// A conker (its map by direction from its middle): glossy mahogany with
/// paler wavy lines round the scar like a grain, the scar (hilum) pale buff,
/// matt and finely pitted inside a dark edge; `k` its age (0 fresh, 1 dried
/// dull, 2 old, dark and wrinkled)
fn conker_tex(k: usize) -> Tex {
    cached_kind_tex!(3, k, |k: usize| {
        let base = [[0.40, 0.18, 0.08], [0.34, 0.17, 0.09], [0.25, 0.13, 0.08]][k];
        let seed = 0xc04 + k as u32 * 7;
        canvas_tex_opt(256, 128, true, move |ctx, w, h| {
            paint_px(ctx, w, h, |px, py| {
                let (u, v) = (px / w, 1.0 - py / h);
                let (phi, th) = (u * 2.0 * PI, (1.0 - v) * PI);
                let d = [th.dsin() * phi.dcos(), th.dcos(), th.dsin() * phi.dsin()];
                let hl = ck_hilum();
                let c0 = d[0] * hl[0] + d[1] * hl[1] + d[2] * hl[2];
                let ang = c0.clamp(-1.0, 1.0).dacos();
                // the grain: wavy bands round the scar
                let wob = fbm(seed, d[0] * 3.0 + 7.0, d[2] * 3.0 + d[1] * 2.0, 3);
                let band = ((ang * 26.0 + wob * 9.0).dsin() * 0.5 + 0.5).dpowf(3.0);
                let mut c = mix_unclamped(base, mul(base, 1.45), 0.45 * band);
                c = mix_unclamped(c, mul(base, 0.75), sstep(1.8, 2.8, ang) * 0.5);
                if k == 2 {
                    // dried: fine wrinkles darker
                    let wr = fbm(seed ^ 5, d[0] * 14.0 + 3.0, d[2] * 14.0 + d[1] * 9.0, 3);
                    c = mix_unclamped(c, mul(c, 0.6), sstep(0.55, 0.62, wr) * (1.0 - sstep(0.62, 0.70, wr)));
                }
                // the scar
                let edge = 0.72f64.dacos();
                let inside = 1.0 - sstep(edge - 0.03, edge, ang);
                let pit = fbm(seed ^ 0x77, d[0] * 30.0 + 1.0, d[2] * 30.0 + d[1] * 20.0, 3);
                let scar = mix_unclamped([0.68, 0.60, 0.46], [0.55, 0.47, 0.35], pit);
                let scar = mix_unclamped(
                    scar,
                    [0.42, 0.33, 0.24],
                    0.4 * sstep(0.62, 0.72, fbm(seed ^ 0x13, d[0] * 6.0, d[2] * 6.0 + d[1] * 4.0, 3)),
                );
                c = mix_unclamped(c, scar, inside);
                c = mix_unclamped(c, mul(base, 0.45), 0.7 * (1.0 - ((ang - edge) / 0.035).abs()).max(0.0));
                // dust on what faces up
                c = mix_unclamped(c, [0.52, 0.48, 0.43], 0.12 * sstep(0.3, 1.0, d[1]));
                [c[0], c[1], c[2], 1.0]
            });
        })
    })
}

/// The husk's outside (box-mapped): dried brown leather with olive left in
/// patches, warty, darker toward the prickles' feet
fn husk_out_tex() -> Tex {
    cached_kind_tex!(1, 0usize, |_k: usize| {
        canvas_tex_opt(256, 256, true, move |ctx, w, h| {
            paint_px(ctx, w, h, |px, py| {
                let (u, v) = (px / w, py / h);
                let n = fbm(0x4a5, u * 9.0, v * 9.0, 4);
                let mut c = mix_unclamped([0.38, 0.29, 0.18], [0.29, 0.22, 0.14], n);
                c = mix_unclamped(c, [0.38, 0.35, 0.21], 0.55 * sstep(0.6, 0.75, fbm(0x4a6, u * 4.0, v * 4.0, 3)));
                let wart = fbm(0x4a7, u * 70.0, v * 70.0, 2);
                c = mix_unclamped(c, mul(c, 0.6), sstep(0.68, 0.76, wart));
                c = mix_unclamped(c, [0.52, 0.46, 0.36], 0.25 * sstep(0.72, 0.8, fbm(0x4a8, u * 30.0, v * 30.0, 2)));
                [c[0], c[1], c[2], 1.0]
            });
        })
    })
}

/// The husk's inside: once white and soft, dried a pale tan, browned in
/// patches, fibrous
fn husk_in_tex() -> Tex {
    cached_kind_tex!(1, 0usize, |_k: usize| {
        canvas_tex_opt(256, 256, true, move |ctx, w, h| {
            paint_px(ctx, w, h, |px, py| {
                let (u, v) = (px / w, py / h);
                let n = fbm(0x1a5, u * 8.0, v * 8.0, 4);
                let mut c = mix_unclamped([0.60, 0.50, 0.36], [0.48, 0.38, 0.26], n);
                c = mix_unclamped(c, [0.42, 0.30, 0.18], 0.6 * sstep(0.58, 0.72, fbm(0x1a6, u * 5.0, v * 5.0, 3)));
                let fib = fbm(0x1a7, u * 90.0, v * 12.0, 2);
                c = mul(c, 0.9 + 0.15 * fib);
                [c[0], c[1], c[2], 1.0]
            });
        })
    })
}
