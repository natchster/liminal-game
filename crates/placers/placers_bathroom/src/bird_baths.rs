//! `place_bird_baths`
use game::assets::props::prop_part_as_modelled;
use game::assets::props::prop_part_info;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use game::visuals::pictures::Window;
use placers_kit::garden::put;
use placers_kit::garden::put_uv;
use placers_kit::garden::top;
use placers_kit::geometry::frame;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_of;

// Bird bath: 3% chance per level.
pub(crate) fn place_bird_baths(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // A small cast concrete pedestal bird bath (Blender batch `t06`), long
    // dry: lichen and black algae on it, its basin crusted, tide lines where
    // the water stood and fell, a last puddle of dark still water, dead
    // leaves blown in (`t05`'s beech and oak) in it and at its foot
    let bath = group();
    // where it stands and how it is turned (the old draws)
    bath.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    bath.rotation().y = rnd() * PI * 2.0;
    let p = bath.pos();
    let salt = salt_of(p.x, p.z);
    put(&bath, "Real__birdBathPedestal", &lambert!({ map: &concrete_tex(salt, true) }));
    put(&bath, "Real__birdBathBowlOut", &lambert!({ map: &concrete_tex(salt ^ 0x77, false) }));
    // the basin inside: weathered concrete is matte (little sheen under a lamp)
    put_uv(
        &bath,
        "Real__birdBathBowlIn",
        &phong!({ map: &bowl_tex(salt), specular: 0x0e0e0e, shininess: 8 }),
        top(-0.205, 0.205, -0.205, 0.205),
    );
    // the water: stagnant, tea dark, its surface still (the lamp's
    // reflection in it small and sharp)
    let water = mesh(
        circle_geo!(BB_WATER_R, 40),
        phong!({ color: 0x2a2920, specular: 0x181816, shininess: 400, transparent: true, opacity: 0.92 }),
    );
    water.rotation().x = -PI / 2.0;
    water.position().y = BB_WATER_Y;
    bath.add(&water);
    // the dead leaves (which of `t05`'s, which side up, how dark: from
    // where it stands, no draw): one afloat, two dried on the dish's slope
    // (along it, lying on it), two on the floor at its foot
    let turn0 = f64::from(salt % 1000) / 1000.0 * PI * 2.0;
    let pickk = |i: u32| ((salt >> (3 * i)) % 5) as usize;
    let back = |i: u32| (salt >> (16 + i)) & 3 == 0;
    // afloat, wet and dark, near the middle
    let afloat = dead_leaf(pickk(0), back(0), 0.62);
    let (fx, fz) = (0.03 * turn0.cos(), 0.03 * turn0.sin());
    afloat.position().set(fx, BB_WATER_Y + 0.0004, fz);
    afloat.rotation().y = turn0;
    bath.add(&afloat);
    for (i, (r, a, turn)) in [(0.118, 2.2, 0.25), (0.106, 4.3, -0.35)].into_iter().enumerate() {
        let (h, slope) = bb_dish(r);
        let phi = turn0 + a;
        // the pivot's +z out along the slope, tipped up with it
        let pivot = group();
        pivot.set_rotation_order(EulerOrder::YXZ);
        pivot.position().set(r * phi.sin(), BB_DISH_Y + h + 0.0025, r * phi.cos());
        pivot.rotation().set(-slope.atan(), phi, 0.0);
        let l = dead_leaf(pickk(1 + i as u32), back(1 + i as u32), 0.82 + 0.1 * i as f64);
        l.rotation().y = turn;
        pivot.add(&l);
        bath.add(&pivot);
    }
    for (i, (r, a)) in [(0.19, 1.1), (0.235, 3.5)].into_iter().enumerate() {
        let phi = turn0 + a;
        let l = dead_leaf(pickk(3 + i as u32), back(3 + i as u32), 0.9 + 0.08 * i as f64);
        l.position().set(r * phi.sin(), 0.0, r * phi.cos());
        l.rotation().y = phi + 1.3 + i as f64 * 2.0;
        bath.add(&l);
    }
    g.world.add(&bath);
    // the old ripple's phase (its draw, kept), and its animation: the leaf
    // afloat drifts and turns a little on the still water in a draught
    let phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let mut q = afloat.position();
        q.x = fx + (t * 0.07 + phase).cos() * 0.006;
        q.z = fz + (t * 0.05 + phase).sin() * 0.005;
        drop(q);
        afloat.rotation().y = turn0 + (t * 0.04 + phase).sin() * 0.4;
    });
}

// ============================================================ bird baths
/// The bird bath's basin as modelled (`nature/garden_things.py`): its dish's lowest point
/// inside and the dish's section near its middle (r, y)
const BB_DISH_Y: f64 = 0.470;

/// The dry dish's water: a puddle 9 mm deep at its middle
const BB_WATER_Y: f64 = BB_DISH_Y + 0.009;

const BB_WATER_R: f64 = 0.093;

/// Cast concrete round and up a pedestal or the basin's outside (u round,
/// v up): its aggregate, weathered grey, lichen and black algae in
/// patches, dark streaks run down from the basin, green toward the foot
fn concrete_tex(salt: u32, streaks: bool) -> Tex {
    canvas_tex_opt(384, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xc0c0;
        fill(ctx, w, h, |u, v| {
            // u wraps: the noise blends across the seam
            let wrap = |sd: u32, fu: f64, fv: f64, o: u32| fbm(sd, u * fu, v * fv, o) * (1.0 - u) + fbm(sd, (u - 1.0) * fu, v * fv, o) * u;
            let n = wrap(seed, 24.0, 16.0, 4);
            let mut c = mix([0.38, 0.37, 0.34], [0.54, 0.53, 0.49], n);
            // the aggregate and the pits where air was trapped, at the scale
            // the map holds (2 mm a pixel round the column)
            let agg = (vn(seed ^ 2, u * 160.0, v * 110.0) - 0.7).max(0.0) * 2.0;
            c = mix(c, [0.3, 0.28, 0.25], agg);
            let pit = (vn(seed ^ 3, u * 190.0, v * 125.0) - 0.9).max(0.0) * 5.0;
            c = mix(c, [0.16, 0.15, 0.14], pit.min(1.0));
            // lichen: pale grey-green rosettes; black algae in the damp
            let li = wrap(seed ^ 4, 18.0, 12.0, 3);
            c = mix(c, [0.62, 0.64, 0.52], sstep(0.62, 0.7, li) * 0.7);
            let al = wrap(seed ^ 5, 10.0, 8.0, 3);
            c = mix(c, [0.12, 0.13, 0.1], sstep(0.6, 0.72, al) * 0.6 * (0.4 + 0.6 * (1.0 - v)));
            if streaks {
                // run-off streaks down from the top
                let st = wrap(seed ^ 6, 40.0, 1.5, 2);
                c = mix(c, c.map(|k| k * 0.62), sstep(0.55, 0.7, st) * sstep(0.2, 1.0, v));
            }
            // green toward the foot where it stood damp
            c = mix(c, [0.24, 0.27, 0.18], sstep(0.25, 0.0, v) * 0.45);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The basin from above (its 410 mm): grey concrete, the dried crust of
/// green-black algae over its floor, tide lines where the water stood and
/// fell, a dark film round the puddle still in it, dust on its rim
fn bowl_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 512, false, move |ctx, w, h| {
        let seed = salt ^ 0xb0b1;
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u - 0.5, v - 0.5);
            let r = x.hypot(y) * 0.41; // m from the middle
            let n = fbm(seed, u * 30.0, v * 30.0, 4);
            let mut c = mix([0.38, 0.37, 0.34], [0.52, 0.51, 0.47], n);
            let agg = (vn(seed ^ 2, u * 220.0, v * 220.0) - 0.7).max(0.0) * 2.0;
            c = mix(c, [0.32, 0.3, 0.27], agg);
            // the algae's crust over the dish's floor, ragged at its edge
            let crust = sstep(0.16, 0.12, r + 0.02 * (fbm(seed ^ 3, u * 14.0, v * 14.0, 3) - 0.5));
            let ac = mix([0.16, 0.18, 0.12], [0.3, 0.33, 0.22], fbm(seed ^ 4, u * 60.0, v * 60.0, 3));
            c = mix(c, ac, crust * 0.85);
            // tide lines where the water stood and dried: wavering, broken,
            // strong here and gone there, a paler crust just outside each
            let (ca, sa) = (y.atan2(x).cos(), y.atan2(x).sin());
            for (k, tr) in [0.121, 0.142, 0.163].into_iter().enumerate() {
                let sd = seed ^ (10 + k as u32);
                let wob = 0.009 * (fbm(sd, ca * 2.5 + 7.0, sa * 2.5, 3) - 0.5) + 0.003 * (fbm(sd ^ 1, u * 22.0, v * 22.0, 2) - 0.5);
                let d = r - tr - wob;
                let on = sstep(0.4, 0.62, fbm(sd ^ 2, ca * 1.8 + 3.0, sa * 1.8 + 3.0, 3));
                c = mix(c, [0.22, 0.21, 0.17], sstep(0.003, 0.0, d.abs()) * on * (0.45 - 0.1 * k as f64));
                c = mix(c, [0.6, 0.58, 0.52], sstep(0.004, 0.0, (d - 0.0035).abs()) * on * 0.2);
            }
            // the rim: dusty, droppings long dried
            c = mix(c, [0.62, 0.6, 0.55], sstep(0.18, 0.2, r) * 0.25);
            let drop = (vn(seed ^ 7, u * 120.0, v * 120.0) - 0.9).max(0.0) * 8.0 * sstep(0.175, 0.19, r);
            c = mix(c, [0.78, 0.76, 0.7], drop.min(1.0) * 0.8);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

// ============================================================ dead leaves
/// Batch `t05`'s dead leaves (`nature/bits_of_the_outside_indoors.py` LEAVES; `placers_nature`'s `leaf_litter.rs`
/// paints them so, and its painter is copied here): the part, the blade's
/// length, its envelope's half width, oak or beech
const LEAVES: [(&str, f64, f64, bool); 5] = [
    ("Real__leafBeechA", 0.072, 0.0225, false),
    ("Real__leafBeechB", 0.062, 0.0195, false),
    ("Real__leafBeechC", 0.056, 0.0180, false),
    ("Real__leafOakA", 0.086, 0.0265, true),
    ("Real__leafOakB", 0.070, 0.0220, true),
];

/// Each leaf's colour (sRGB): beech russet, beech brown, beech pale tan, oak
/// brown, oak dark (wet once)
const LEAF_RGB: [[f64; 3]; 5] = [[0.56, 0.37, 0.21], [0.48, 0.34, 0.22], [0.63, 0.49, 0.33], [0.54, 0.42, 0.28], [0.42, 0.32, 0.22]];

/// The half width of leaf `k`'s blade at a fraction `a` of its length, on
/// side `side` (±1): beech ovate with a wavy margin, oak lobed, each a little
/// ragged, and one tear (as `t05` cuts them)
fn leaf_half(k: usize, a: f64, side: f64) -> f64 {
    let (_, _, w, oak) = LEAVES[k];
    if !(0.0..=1.0).contains(&a) {
        return 0.0;
    }
    let seed = 0x1ea0 + k as u32 * 17;
    let mut hw = if oak {
        let env = (PI * a.powf(0.62)).sin().max(0.0).powf(0.75) * (1.0 - 0.15 * (1.0 - a)) + 0.18 * (-((a - 0.03) / 0.05).powi(2)).exp();
        let t = (a * 5.2 + 0.2 + 0.08 * side).fract();
        let lobe = if a > 0.1 { 0.58 + 0.42 * (1.0 - (2.0 * t - 1.0).powi(2)).max(0.0).sqrt() } else { 1.0 };
        w * env * lobe
    } else {
        let env = (PI * a.powf(1.15)).sin().max(0.0).powf(0.8);
        w * env * (1.0 - 0.04 * (0.5 + 0.5 * (2.0 * PI * a * 8.5).cos()))
    } - 0.0006;
    hw *= 1.0 - 0.07 * sstep(0.55, 0.8, fbm(seed, a * 18.0, side * 3.0, 3));
    let ta = 0.35 + 0.3 * hash(seed, 1, 0);
    if side > 0.0 && k % 2 == 0 {
        let d = ((a - ta) / 0.025).abs();
        hw *= 1.0 - 0.7 * (1.0 - d).max(0.0);
    }
    hw.max(0.0)
}

/// Leaf `k` seen from above in its part's frame (as `t05` paints it): the
/// blade's outline cut out, holes eaten through, mottled, darker at the
/// dried margin, fungal spots, the veins paler; made once a thread
fn leaf_tex(k: usize) -> Tex {
    thread_local! { static T: RefCell<Vec<Option<Tex>>> = RefCell::new(vec![None; 5]); }
    let k = k.min(4);
    if let Some(t) = T.with(|t| t.borrow()[k].clone()) {
        return t;
    }
    let (name, l, _, oak) = LEAVES[k];
    let (lo, hi) = frame(name);
    let (ex, ez) = (hi[0] - lo[0], hi[2] - lo[2]);
    let th = (256.0 * ez / ex).round().max(32.0);
    let base = LEAF_RGB[k];
    let seed = 0x1ea7 + k as u32 * 31;
    let tex = canvas_tex_opt(256, th, false, move |ctx, w, h| {
        let px_m = ex / w;
        // pixel rows top down (z from lo[2]); `fill` gives v up
        fill(ctx, w, h, |u, v| {
            let x = l - (hi[0] - (lo[0] + u * ex));
            let z = lo[2] + (1.0 - v) * ez;
            let a = x / l;
            if x < 0.0004 {
                // the petiole
                let c = mix([0.34, 0.24, 0.15], [0.26, 0.18, 0.12], vn(seed, u * w * 0.3, v * h * 0.3));
                return [c[0], c[1], c[2], 1.0];
            }
            let side = if z > 0.0 { 1.0 } else { -1.0 };
            let hw = leaf_half(k, a, side);
            let edge = (hw - z.abs()) / px_m;
            let inside = (edge + 0.5).clamp(0.0, 1.0);
            let hole_n = fbm(seed ^ 0x40, x * 260.0, z * 260.0, 3);
            let hole = sstep(0.77, 0.79, hole_n);
            let skel = sstep(0.70, 0.77, hole_n) * (1.0 - hole);
            let m = fbm(seed ^ 0x11, x * 120.0, z * 120.0, 4);
            let mut c = mix(base, mix(base, [0.30, 0.22, 0.15], 0.6), sstep(0.45, 0.8, m));
            let margin = 1.0 - sstep(0.0, 0.004, hw - z.abs());
            c = mix(c, [0.30, 0.23, 0.17], 0.5 * margin);
            let sp = fbm(seed ^ 0x77, x * 300.0, z * 300.0, 2);
            c = mix(c, [0.24, 0.18, 0.13], 0.55 * sstep(0.76, 0.79, sp));
            let mid_w = 0.0007 - 0.0005 * a;
            let midrib = 1.0 - sstep(mid_w * 0.5, mid_w, z.abs());
            let (pairs, slope) = if oak { (6.0, 0.62) } else { (8.0, 0.9) };
            let q = (x - z.abs() * slope) / l * pairs - 0.35;
            let dq = (q - q.round()).abs() * l / pairs;
            let lateral =
                (1.0 - sstep(0.00018, 0.00042, dq)) * sstep(0.0, 0.02, q / pairs) * (1.0 - sstep(0.85, 1.0, z.abs() / hw.max(1e-6)));
            let net = (1.0 - sstep(0.0, 0.08, (vn(seed ^ 0x9, x * 900.0, z * 900.0) - 0.5).abs())) * 0.25;
            let vein = midrib.max(lateral * 0.8).max(net * 0.4);
            c = mix(c, mix(c, [0.78, 0.66, 0.50], 0.45), vein);
            c = mix(c, [0.62, 0.55, 0.45], 0.5 * skel);
            [c[0], c[1], c[2], inside * (1.0 - hole)]
        });
    });
    // where it is clear its colour is the leaf's, so its smaller mips keep
    // the leaf's colour at the cut edge instead of black
    {
        let img = tex.image();
        let mut im = img.borrow_mut();
        for px in im.data.chunks_exact_mut(4) {
            let a = sstep(0.0, 0.5, f64::from(px[3]) / 255.0);
            for (c, b) in px.iter_mut().zip(base) {
                *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * a).round().clamp(0.0, 255.0) as u8;
            }
        }
        im.version += 1;
    }
    T.with(|t| t.borrow_mut()[k] = Some(tex.clone()));
    tex
}

/// Dead leaf `k` (one of `t05`'s), its back up or not, a shade darker or
/// lighter; its lowest point at the origin
fn dead_leaf(k: usize, back_up: bool, shade: f64) -> Obj {
    let leaf = group();
    let name = LEAVES[k.min(4)].0;
    if let Some(geo) = prop_part_as_modelled(name) {
        let tint = ((shade.clamp(0.0, 1.0) * 255.0) as u32) * 0x010101;
        let m = mesh(geo, lambert!({ map: &leaf_tex(k), color: tint, alphaTest: 0.5, side: DoubleSide }));
        if back_up {
            m.rotation().x = PI;
            m.position().y = frame(name).1[1];
        }
        leaf.add(&m);
    }
    leaf
}

/// The dish inside the basin (`nature/garden_things.py` `_bowl_profile`): its height over
/// its lowest point and its slope at `r` m from the middle
fn bb_dish(r: f64) -> (f64, f64) {
    const P: [(f64, f64); 5] = [(0.0, 0.0), (0.07, 0.004), (0.12, 0.014), (0.16, 0.030), (0.18, 0.038)];
    for w in P.windows(2) {
        let ((r0, y0), (r1, y1)) = (w[0], w[1]);
        if r <= r1 {
            let s = (y1 - y0) / (r1 - r0);
            return (y0 + (r.max(r0) - r0) * s, s);
        }
    }
    (0.038, 0.4)
}
