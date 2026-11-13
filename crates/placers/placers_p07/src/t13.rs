//! Batch `t13`'s party food, made real (`docs/design/PROPS.md` P3, "the party and
//! the shrine"): a pressed glass cake stand and the cake nobody ate on it,
//! and cornbread muffins in their paper cases, years on
//! (`assets/src/blender/props/t13.py`; what the real ones are and where
//! their sizes come from: `t13.notes.md`). The mould is the stale loaf's
//! (`loaf_mould`, batch `r16`'s). Each keeps the old placer's draws, call
//! for call (the comments at each say what a draw now chooses).
use crate::{loaf_mould, open_tiles, pick};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use kit::*;

/// The cake as modelled (`t13.py`): the bottom tier's radius; the cake
/// part's heights (its bounds) and where the top tier's foot is
const CAKE_R1: f64 = 0.098;
const CAKE_Y: (f64, f64) = (0.1152, 0.2572);
const CAKE_TIER2: f64 = 0.1915;
/// The stand's plate (its bounds: half width, and heights)
const STAND_PLATE: (f64, f64, f64) = (0.139_866, 0.1000, 0.1203);
/// The candle's flame root, over the wick (`t13.py` prints it)
const FLAME_ROOT: (f64, f64, f64) = (0.0208, 0.2881, 0.0107);
/// `t09`'s flame (`Real__brassStickFlameB`): its foot's height; and how
/// much smaller a birthday candle's flame is than a dinner candle's
const FLAME_FOOT: f64 = 0.170;
const FLAME_SCALE: f64 = 0.55;

// Cake stand: 5% chance per level. An 11 in pressed glass cake stand
// (a trumpet foot in twelve panels, a plate rayed underneath, its rim
// scalloped), milk glass or clear, dusty; on it (55%) a two-tier
// buttercream cake nobody ate, years on: dried and shrunk, slumped, its
// frosting cracked to the sponge and yellowed, mould in it; now and then a
// birthday candle burnt down in it, sometimes still burning.
pub(crate) fn place_cake_stands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour draw: white is milk glass, cream clear glass
    let milk = rnd() < 0.5;
    // (55%) the cake; its frosting the old colour's (vanilla, pink, lilac,
    // chocolate); (30%) a candle in it, (60% of those) lit, its flicker's
    // two phases
    let cake = if rnd() < 0.55 {
        let cake_colors: [u32; 4] = [0xf0e0c0, 0xf8d0d0, 0x8060a0, 0xa04020];
        let c = pick(&cake_colors);
        let flavour = cake_colors.iter().position(|&k| k == c).unwrap_or(0);
        let candle = if rnd() < 0.3 {
            if rnd() < 0.6 {
                let flicker_phase = rnd() * PI * 2.0;
                let flicker_phase2 = rnd() * PI * 2.0;
                Some(Some((flicker_phase, flicker_phase2)))
            } else {
                Some(None)
            }
        } else {
            None
        };
        Some((flavour, candle))
    } else {
        None
    };
    let px = g.wx(x) + (rnd() - 0.5) * 1.6;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.6;
    let yaw = rnd() * PI * 2.0;
    let salt = salt(px, pz, 679);
    let stand = group();
    let dust = stand_dust_tex(salt, cake.is_some(), milk);
    let glass = if milk { phong!({ map: &dust, color: 0xffffff, specular: 0x2c2c2c, shininess: 40 }) } else { clear_glass_mat(&dust) };
    if let Some(m) = put_split(&stand, "Real__cakeStandFoot", &glass) {
        // the foot writes depth: the placement measures the stand by it
        m.mat().depthWrite(true);
        m.set_render_order(2);
    }
    if let Some(m) = put_split(&stand, "Real__cakeStandPlate", &glass) {
        m.set_render_order(2);
    }
    if let Some((flavour, candle)) = cake {
        put_split(&stand, "Real__cakeStandCake", &lambert!({ map: &cake_tex(flavour, salt) }));
        if let Some(lit) = candle {
            put(&stand, "Real__cakeStandCandle", &lambert!({ map: &wax_tex(salt) }));
            put(&stand, "Real__cakeStandWick", &lambert!({ color: if lit.is_some() { 0x15120f } else { 0x2a2520 } }));
            if let Some((flicker_phase, flicker_phase2)) = lit {
                // the flame (`t09`'s, smaller), scaled from its root as it
                // flickers; the old light
                let flame = group();
                flame.position().set(FLAME_ROOT.0, FLAME_ROOT.1, FLAME_ROOT.2);
                flame.scale().set(FLAME_SCALE, FLAME_SCALE, FLAME_SCALE);
                if let Some(m) =
                    put(&flame, "Real__brassStickFlameB", &basic_mat!({ map: &flame_tex(), transparent: true, depthWrite: false }))
                {
                    m.position().y = -FLAME_FOOT;
                    m.set_render_order(3);
                }
                stand.add(&flame);
                let light = point_light!(0xffa040, 0.35, 1.6, 2);
                light.position().set(FLAME_ROOT.0, FLAME_ROOT.1 + 0.006, FLAME_ROOT.2);
                stand.add(&light);
                g.animate(move |_g, _dt, t| {
                    // the old flicker: two sines (5 and 9 Hz)
                    let f = 0.85 + (t * 5.0 + flicker_phase).sin() * 0.10 + (t * 9.0 + flicker_phase2).sin() * 0.06;
                    flame.scale().set(FLAME_SCALE * (0.9 + 0.1 * f), FLAME_SCALE * f * 1.1, FLAME_SCALE * (0.9 + 0.1 * f));
                    light.set_intensity(0.35 * f);
                });
            }
        }
    }
    stand.position().set(px, 0, pz);
    stand.rotation().y = yaw;
    g.world.add(&stand);
}

/// Pressed glass as `Patch::Glass` lays it (the wine glass's): the room
/// seen in it by Fresnel, its dust from the map's grey
fn clear_glass_mat(map: &Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 220, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "cake_stand_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it, and nothing lit behind it
        f[0] = 0.40;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // dust: how much of the map's grey, its colour (linear)
        f[8] = 1.0;
        f[9] = 0.20;
        f[10] = 0.19;
        f[11] = 0.17;
        // thick pressed glass is never quite clear: a little of the room
        // held in it
        f[12] = 0.10;
        // and more of the room seen in it square on than a thin glass
        f[13] = 0.16;
    }
    m
}

/// The stand's dust, laid as `put_split` lays it (the sides round and up
/// below, the tops from above in the upper left): grey where dust lies
/// (clear glass), or the milk glass's white under its dust. Thick on the
/// plate's top round the cake (none under it), on the knop and the foot's
/// shoulders; a hand's smears on the rim; grime on the foot's ground rim
fn stand_dust_tex(salt: u32, cake: bool, milk: bool) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x57a;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 16.0, v * 16.0, 3);
            let (mut d, top);
            if v >= 0.5 {
                // from above: the plate's top (or its underside, the foot's
                // ground rim)
                let fz = (u * 2.0).fract();
                let fx = (v - 0.5) * 2.0;
                let (xx, zz) = ((fx * 2.0 - 1.0) * STAND_PLATE.0, (fz * 2.0 - 1.0) * STAND_PLATE.0);
                let rr = (xx * xx + zz * zz).sqrt();
                d = 0.62 + 0.33 * n;
                if cake && rr < CAKE_R1 + 0.004 {
                    d *= 0.15;
                }
                // a ring of grime where the cake's foot met the glass
                if cake {
                    d += 0.3 * sstep(0.006, 0.0, (rr - CAKE_R1 - 0.006).abs());
                }
                top = true;
            } else {
                d = 0.32 + 0.3 * n;
                // the rim's smears: where hands took it
                let sm = vn(seed ^ 3, u * 6.0, v * 40.0);
                d += 0.3 * sstep(0.65, 0.9, sm) * sstep(0.42, 0.48, v);
                top = false;
            }
            let fleck = vn(seed ^ 7, u * 120.0, v * 120.0);
            d += 0.25 * sstep(0.8, 0.95, fleck);
            if milk {
                let base = [0.86, 0.85, 0.81];
                let c = mix(base, [0.58, 0.55, 0.49], d * if top { 0.5 } else { 0.35 });
                [c[0], c[1], c[2], 1.0]
            } else {
                let g = d.clamp(0.0, 1.0);
                [g, g, g, 1.0]
            }
        });
    })
}

/// The frosting (sRGB 0..1) by the old colour, as it was and as years left
/// it (the fat gone yellow and grey), and the sponge under it, dried
const FROSTING: [([f64; 3], [f64; 3]); 4] = [
    ([0.88, 0.82, 0.66], [0.70, 0.62, 0.46]),
    ([0.86, 0.68, 0.66], [0.70, 0.56, 0.50]),
    ([0.56, 0.44, 0.60], [0.50, 0.44, 0.48]),
    ([0.40, 0.25, 0.17], [0.33, 0.24, 0.18]),
];
const SPONGE: [[f64; 3]; 4] = [[0.60, 0.48, 0.30], [0.60, 0.48, 0.30], [0.58, 0.46, 0.30], [0.24, 0.15, 0.10]];

/// The cake, laid as `put_split` lays it: its sides (round and up) below,
/// its tops (from above) in the upper left. Buttercream gone yellow and
/// grey, cracked in a net down to the dried sponge, crazed fine between;
/// chocolate bloomed white; mould colonies (grey-green spores, white fuzz, a
/// stain round them), most low on the sides; dust thick on the tops
fn cake_tex(flavour: usize, salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xca4e;
        let mut r = SplitMix64::new(salt, 0x6a7);
        // mould colonies (px): on the sides, most near the foot; a few on top
        let cols: Vec<(f64, f64, f64)> = (0..12)
            .map(|k| {
                if k < 9 {
                    (r.next() * w, h - r.next().powf(1.8) * h * 0.4, if k < 2 { r.range(10.0, 18.0) } else { r.range(3.0, 7.0) })
                } else {
                    (r.range(0.1, 0.4) * w, r.range(0.1, 0.4) * h, r.range(4.0, 9.0))
                }
            })
            .collect();
        let (fresh, aged) = FROSTING[flavour];
        let sponge = SPONGE[flavour];
        fill(ctx, w, h, |u, v| {
            let (px, py) = (u * w, (1.0 - v) * h);
            let top = v >= 0.5;
            let (cu, cv) = if top { ((u * 2.0).fract() * 3.0, (v - 0.5) * 6.0) } else { (u * 9.0, v * 4.0) };
            let n = fbm(seed, cu * 6.0, cv * 6.0, 3);
            let mut c = mix(fresh, aged, 0.75 + 0.25 * n);
            // the net of cracks the frosting split into as it dried, and the
            // fine crazing between
            let e = (vn(seed ^ 1, cu * 3.5, cv * 3.5) - 0.5).abs() + 0.02 * (n - 0.5);
            let craze = (1.0 - ((vn(seed ^ 2, cu * 14.0, cv * 14.0) - 0.5).abs() * 10.0).min(1.0)) * 0.3;
            // the frosting lifted at a crack's lips, the sponge down in it,
            // dark at its bottom
            c = mix(c, sponge, sstep(0.07, 0.035, e) * 0.85);
            c = mix(c, [0.16, 0.12, 0.09], sstep(0.022, 0.006, e) * 0.85);
            c = mix(c, mix(aged, [0.0, 0.0, 0.0], 0.4), craze);
            if !top {
                // fat seeped out and darkened round each tier's foot
                let y = CAKE_Y.0 + v * 2.0 * (CAKE_Y.1 - CAKE_Y.0);
                let foot = sstep(0.012, 0.0, y - CAKE_Y.0) + sstep(0.010, 0.0, (y - CAKE_TIER2).abs()) * 0.7;
                c = mix(c, mix(aged, [0.20, 0.15, 0.10], 0.6), foot * (0.4 + 0.3 * n));
            }
            if flavour == 3 {
                // fat bloom on the chocolate: grey-white
                c = mix(c, [0.62, 0.58, 0.52], sstep(0.55, 0.8, fbm(seed ^ 5, cu * 3.0, cv * 3.0, 2)) * 0.5);
            }
            // the mould
            let (s, wr, st) = loaf_mould(&cols, px, py, seed ^ 7);
            c = mix(c, [0.56, 0.50, 0.36], st * 0.35);
            c = mix(c, [0.80, 0.80, 0.76], wr * 0.6);
            let sp = vn(seed ^ 9, px / 1.5, py / 1.5);
            c = mix(c, [0.36 + 0.1 * sp, 0.42 + 0.07 * sp, 0.35 + 0.05 * sp], s * 0.9);
            // dust: thick on the tops, a film on the sides
            c = mix(c, [0.58, 0.56, 0.51], if top { 0.30 + 0.15 * n } else { 0.10 + 0.08 * n });
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A birthday candle's wax (round and up): a pastel faded and dusty, the
/// burnt top darker
fn wax_tex(salt: u32) -> Tex {
    canvas_tex_opt(32, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x3a5;
        let base = [[0.86, 0.80, 0.56], [0.76, 0.82, 0.88], [0.88, 0.74, 0.78], [0.78, 0.86, 0.74]][(salt >> 6) as usize % 4];
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 4.0, v * 8.0, 2);
            let mut c = mix(base, [0.72, 0.70, 0.64], 0.3 + 0.2 * n);
            c = mix(c, [0.30, 0.27, 0.24], sstep(0.85, 1.0, v) * 0.5);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A candle flame (`t09`'s, laid round it, v up): a dim blue root, the
/// bright yellow-white body, orange toward its tip, soft at its edge
fn flame_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(16, 64, false, |ctx, w, h| {
                fill(ctx, w, h, |_u, v| {
                    let root = 1.0 - sstep(0.04, 0.2, v);
                    let tip = sstep(0.35, 1.0, v);
                    let c = mix([1.0, 0.86, 0.56], [1.0, 0.56, 0.22], tip);
                    let c = mix(c, [0.36, 0.40, 0.80], root);
                    let a = (0.85 - 0.5 * root) * (1.0 - 0.55 * sstep(0.7, 1.0, v));
                    [c[0], c[1], c[2], a]
                });
            })
        })
        .clone()
    })
}

/// The muffin's case: how far its ring had to open for real muffins to sit
/// side by side (a case's top is 71 mm across)
const MUFFIN_CLEAR: f64 = 0.0356;
/// The crumbs (`r16.py`'s `Real__breadCrumbs`, on its board) moved to the
/// middle of the muffins, on the floor
const CRUMBS_AT: (f64, f64, f64) = (-0.066, -0.0178, 0.011);

// Cornbread muffin cluster: 6% chance per level. Three to five
// cornbread muffins left in their paper baking cups, years on: dried hard
// and shrunk from their cases, the crowns cracked, gone dull and grey,
// mould in soft colonies; the cases stained with grease; crumbs.
pub(crate) fn place_cornbread_muffins(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old golden colours: which muffin (two shapes, as they split)
    let golden_colors: [u32; 4] = [0xe0b060, 0xd8a850, 0xd0a044, 0xc89840];
    let count = 3 + (rnd() * 3.0).floor() as i32;
    // each muffin's old draws: its colour, where round the middle, how far
    // out, its turn
    let mut each = vec![];
    for i in 0..count {
        let c = pick(&golden_colors);
        let k = golden_colors.iter().position(|&g| g == c).unwrap_or(0);
        let ang = (i as f64 / count as f64) * PI * 2.0 + rnd() * 0.3;
        let rr = 0.02 + rnd() * 0.06;
        let turn = rnd() * PI * 2.0;
        each.push((k, ang, rr, turn));
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    let yaw = rnd() * PI * 2.0;
    let salt = salt(px, pz, 680);
    let case = lambert!({ map: &case_tex(salt), side: DoubleSide });
    let crowns = [lambert!({ map: &muffin_tex(salt, 0) }), lambert!({ map: &muffin_tex(salt, 1) })];
    let cluster = group();
    // real muffins cannot sit 2 cm apart: the old ring opened out until
    // their cases clear each other, the old spread kept over that
    let ring = MUFFIN_CLEAR / (PI / f64::from(count)).sin();
    for &(k, ang, rr, turn) in &each {
        let muffin = group();
        put(&muffin, "Real__cornMuffinCase", &case);
        put_split(&muffin, if k % 2 == 0 { "Real__cornMuffinA" } else { "Real__cornMuffinB" }, &crowns[k % 2]);
        let r2 = ring + (rr - 0.02) * 0.3;
        muffin.position().set(ang.cos() * r2, 0, ang.sin() * r2);
        muffin.rotation().y = turn;
        cluster.add(&muffin);
    }
    if let Some(geo) = prop_part_as_modelled("Real__breadCrumbs") {
        let crumbs = mesh(geo, lambert!({ color: 0x9a7e50 }));
        crumbs.position().set(CRUMBS_AT.0, CRUMBS_AT.1, CRUMBS_AT.2);
        cluster.add(&crumbs);
    }
    cluster.position().set(px, 0, pz);
    cluster.rotation().y = yaw;
    g.world.add(&cluster);
}

/// The muffin's case as modelled (`t13.py`): its height
const CASE_H: f64 = 0.033;

/// The baking cups (round and up; the base below): plain white paper, kraft
/// brown, or printed with small dots; grease soaked through in blotches
/// with dark tide edges, most low down; the pleats' creases; dust on the
/// rim, grime at the foot
fn case_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xcace;
        let kind = (salt >> 7) % 4;
        let paper = match kind {
            2 => [0.60, 0.48, 0.34],
            _ => [0.86, 0.82, 0.72],
        };
        fill(ctx, w, h, |u, v| {
            let y = v * CASE_H;
            let n = fbm(seed, u * 12.0, v * 6.0, 3);
            let mut c = mix(paper, [0.70, 0.67, 0.60], 0.25 * n);
            if kind == 3 {
                let (fx, fy) = ((u * 48.0).fract() - 0.5, (v * 6.0).fract() - 0.5);
                if fx * fx + fy * fy * 0.5 < 0.04 {
                    c = [0.74, 0.46, 0.44];
                }
            }
            // the pleats' creases
            c = mix(c, [0.0, 0.0, 0.0], 0.08 * (0.5 + 0.5 * (u * PI * 48.0).sin()));
            // grease: blotches, darker at their tide edges, most low down
            let gb = fbm(seed ^ 4, u * 9.0, v * 3.0, 3) + 0.25 * (1.0 - v);
            let blot = sstep(0.52, 0.62, gb);
            let tide = sstep(0.03, 0.0, (gb - 0.56).abs());
            c = mix(c, [0.66, 0.54, 0.32], blot * 0.55);
            c = mix(c, [0.46, 0.36, 0.20], tide * 0.4);
            // dust on the rim, grime at the foot
            c = mix(c, [0.60, 0.57, 0.50], 0.08 + 0.2 * sstep(0.025, 0.033, y));
            c = mix(c, [0.38, 0.35, 0.30], 0.4 * sstep(0.004, 0.0, y));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The cracks across each muffin's crown (`t13.py`'s MUFFIN_CRACKS: x0,
/// z0, x1, z1 in m) and the crown's half width
const MUFFIN_CRACKS: [&[(f64, f64, f64, f64)]; 2] = [
    &[(-0.024, 0.006, 0.022, -0.004), (0.004, 0.0, 0.010, 0.020)],
    &[(-0.018, -0.016, 0.0, 0.002), (0.0, 0.002, 0.021, -0.006), (0.0, 0.002, -0.004, 0.022)],
];
const CROWN_R: f64 = 0.033;

/// The muffin, laid as `put_split` lays it (its sides round and up below,
/// its crown from above in the upper left): cornbread's crust, golden gone
/// dull and dusty, pebbled with the coarse meal, browner at the shoulder;
/// the cracks across the crown (shape `k`'s) dark, the paler crumb at
/// their lips; mould colonies (the loaf's); dust on the crown
fn muffin_tex(salt: u32, k: usize) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x3fb ^ (k as u32 * 0x91);
        let mut r = SplitMix64::new(seed, 0x10af);
        let cols: Vec<(f64, f64, f64)> = (0..7)
            .map(|i| {
                if i < 4 {
                    (r.range(0.1, 0.9) * w * 0.5, r.range(0.05, 0.45) * h, if i < 1 { r.range(5.0, 8.0) } else { r.range(2.0, 4.0) })
                } else {
                    (r.next() * w, h * r.range(0.5, 0.62), r.range(2.0, 5.0))
                }
            })
            .collect();
        let bake = [0.74, 0.56, 0.28];
        fill(ctx, w, h, |u, v| {
            let (px, py) = (u * w, (1.0 - v) * h);
            let top = v >= 0.5;
            let n = fbm(seed, u * 10.0, v * 10.0, 3);
            let mut c = mix(bake, [0.56, 0.47, 0.34], 0.25 + 0.15 * n);
            // the coarse meal: flecks paler and darker
            let g1 = vn(seed ^ 2, px * 0.9, py * 0.9);
            let g2 = vn(seed ^ 3, px * 1.3 + 7.0, py * 1.3);
            c = mix(c, [0.84, 0.70, 0.42], sstep(0.70, 0.9, g1) * 0.5);
            c = mix(c, [0.36, 0.24, 0.12], sstep(0.74, 0.92, g2) * 0.45);
            if top {
                let (x, z) = (((v - 0.5) * 2.0 * 2.0 - 1.0) * CROWN_R, ((u * 2.0).fract() * 2.0 - 1.0) * CROWN_R);
                let rr = (x * x + z * z).sqrt() / CROWN_R;
                // browner toward the shoulder
                c = mix(c, [0.46, 0.32, 0.17], sstep(0.6, 0.95, rr) * 0.4);
                // the cracks: dark down in them, the crumb paler at their lips
                let mut dist = 1.0f64;
                for &(x0, z0, x1, z1) in MUFFIN_CRACKS[k] {
                    let (ax, az) = (x1 - x0, z1 - z0);
                    let t = (((x - x0) * ax + (z - z0) * az) / (ax * ax + az * az)).clamp(0.0, 1.0);
                    let d =
                        ((x - x0 - ax * t).powi(2) + (z - z0 - az * t).powi(2)).sqrt() + 0.0015 * (1.0 - (PI * (0.15 + 0.85 * t)).sin());
                    dist = dist.min(d);
                }
                let wob = 0.0005 * (vn(seed ^ 12, x * 900.0, z * 900.0) - 0.5);
                c = mix(c, [0.82, 0.68, 0.42], sstep(0.0028, 0.0016, dist + wob) * 0.6);
                c = mix(c, [0.24, 0.16, 0.08], sstep(0.0016, 0.0006, dist + wob) * 0.85);
                c = mix(c, [0.60, 0.57, 0.51], 0.16 + 0.12 * n);
            } else {
                // the side: the crumb paler down in the case, the crust above
                let y = v * 2.0;
                c = mix(c, [0.72, 0.60, 0.38], sstep(0.62, 0.5, y) * 0.5);
                c = mix(c, [0.60, 0.57, 0.51], 0.08);
            }
            let (s, wr, st) = loaf_mould(&cols, px, py, seed ^ 7);
            c = mix(c, [0.56, 0.50, 0.32], st * 0.3);
            c = mix(c, [0.82, 0.82, 0.78], wr * 0.6);
            let sp = vn(seed ^ 9, px / 1.2, py / 1.2);
            c = mix(c, [0.34 + 0.1 * sp, 0.40 + 0.08 * sp, 0.33 + 0.05 * sp], s * 0.9);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

mod kit {
    use game::assets::props::prop_part_as_modelled;
    use game::prelude::*;

    pub(super) use placers_kit::noise::fbm;
    pub(super) use placers_kit::noise::hash;
    pub(super) use placers_kit::noise::mix;
    pub(super) use placers_kit::noise::sstep;
    pub(super) use placers_kit::noise::vn;
    pub(super) use placers_kit::paint::fill;
    pub(super) use placers_kit::place::salt;
    /// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
    pub(super) use placers_kit::geometry::put_top;
    /// The same, its texture coordinates folded so its sides and its tops
    /// each get their own place on one map: a triangle that mostly looks up
    /// or down takes the disc the game lays on a cylinder's caps (from
    /// above), folded into the upper left quarter; any other the round and
    /// up of its sides, folded into the lower half. A corner the two kinds
    /// share is doubled, so no triangle runs across the map from one to
    /// the other.
    pub(super) fn put_split(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = prop_part_as_modelled(name)?.duplicate();
        {
            let mut gg = geo.g();
            let idx = gg.index.clone()?;
            let n = gg.position.len() / 3;
            let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
            for i in 0..n {
                for k in 0..3 {
                    let p = f64::from(gg.position[i * 3 + k]);
                    lo[k] = lo[k].min(p);
                    hi[k] = hi[k].max(p);
                }
            }
            let ex = [0, 1, 2].map(|k| if hi[k] - lo[k] > 1e-9 { hi[k] - lo[k] } else { 1.0 });
            let ny = |i: usize| gg.normal[i * 3 + 1];
            let cap = |i: usize| ny(i).abs() > 0.75;
            let (mut pos, mut nrm) = (gg.position.clone(), gg.normal.clone());
            let mut uv: Vec<f32> = Vec::with_capacity(gg.uv.len());
            for i in 0..n {
                let (u, v) = (gg.uv[i * 2], gg.uv[i * 2 + 1]);
                if cap(i) {
                    uv.extend_from_slice(&[u * 0.5, 0.5 + v * 0.5]);
                } else {
                    uv.extend_from_slice(&[u, v * 0.5]);
                }
            }
            let mut out = Vec::with_capacity(idx.len());
            let mut dup: std::collections::HashMap<(u32, u8), u32> = std::collections::HashMap::new();
            for t in idx.chunks(3) {
                let tri_cap = t.iter().map(|&i| ny(i as usize).abs()).sum::<f32>() / 3.0 > 0.6;
                let up = t.iter().map(|&i| ny(i as usize)).sum::<f32>() >= 0.0;
                let side_u = t.iter().map(|&i| i as usize).find(|&i| !cap(i)).map(|i| uv[i * 2]);
                for &i in t {
                    let iu = i as usize;
                    if cap(iu) == tri_cap {
                        out.push(i);
                        continue;
                    }
                    let f = [0, 1, 2].map(|k| (f64::from(gg.position[iu * 3 + k]) - lo[k]) / ex[k]);
                    let (u, v, key) = if tri_cap {
                        (f[2] * 0.5, 0.5 + if up { f[0] } else { 1.0 - f[0] } * 0.5, u8::from(up))
                    } else {
                        let mut u = ((f[0] * 2.0 - 1.0).atan2(f[2] * 2.0 - 1.0) / (2.0 * PI) + 1.0) % 1.0;
                        let mut key = 2;
                        if let Some(m) = side_u.map(f64::from) {
                            if u - m > 0.5 {
                                u -= 1.0;
                                key = 3;
                            } else if m - u > 0.5 {
                                u += 1.0;
                                key = 4;
                            }
                        }
                        (u, f[1] * 0.5, key)
                    };
                    let d = *dup.entry((i, key)).or_insert_with(|| {
                        pos.extend_from_slice(&[gg.position[iu * 3], gg.position[iu * 3 + 1], gg.position[iu * 3 + 2]]);
                        nrm.extend_from_slice(&[gg.normal[iu * 3], gg.normal[iu * 3 + 1], gg.normal[iu * 3 + 2]]);
                        uv.extend_from_slice(&[u as f32, v as f32]);
                        (pos.len() / 3 - 1) as u32
                    });
                    out.push(d);
                }
            }
            gg.position = pos;
            gg.normal = nrm;
            gg.uv = uv;
            gg.index = Some(out);
            gg.touch();
        }
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
