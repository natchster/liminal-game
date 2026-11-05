//! Batch `t13`'s party food, made real (`docs/PROPS.md` P3, "the party and
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

/// The cake as modelled (`t13.py`): the bottom tier's radius
const CAKE_R1: f64 = 0.098;
/// The stand's plate (its bounds: half width, and heights)
const STAND_PLATE: (f64, f64, f64) = (0.139_866, 0.1000, 0.1203);
/// The candle's flame root, over the wick (`t13.py` prints it)
const FLAME_ROOT: (f64, f64, f64) = (0.0208, 0.2881, 0.0107);
/// `t09`'s flame (`Real__brassStickFlameB`): its foot's height; and how
/// much smaller a birthday candle's flame is than a dinner candle's
const FLAME_FOOT: f64 = 0.170;
const FLAME_SCALE: f64 = 0.55;

// R679 cake stand: 5% chance per level. An 11 in pressed glass cake stand
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
    let glass = if milk {
        phong!({ map: &dust, color: 0xffffff, specular: 0x2c2c2c, shininess: 40 })
    } else {
        clear_glass_mat(&dust)
    };
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
        f[0] = 0.30;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // dust: how much of the map's grey, its colour (linear)
        f[8] = 0.75;
        f[9] = 0.20;
        f[10] = 0.19;
        f[11] = 0.17;
        f[12] = 0.0;
        // thick pressed glass: a little more of it seen square on
        f[13] = 0.10;
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
                d = 0.55 + 0.35 * n;
                if cake && rr < CAKE_R1 + 0.004 {
                    d *= 0.15;
                }
                // a ring of grime where the cake's foot met the glass
                if cake {
                    d += 0.3 * sstep(0.006, 0.0, (rr - CAKE_R1 - 0.006).abs());
                }
                top = true;
            } else {
                d = 0.18 + 0.25 * n;
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
            let net = (1.0 - ((vn(seed ^ 1, cu * 7.0, cv * 7.0) - 0.5).abs() * 22.0).min(1.0)).powi(2);
            let craze = (1.0 - ((vn(seed ^ 2, cu * 26.0, cv * 26.0) - 0.5).abs() * 18.0).min(1.0)) * 0.35;
            c = mix(c, sponge, net * 0.8);
            c = mix(c, mix(aged, [0.0, 0.0, 0.0], 0.4), craze);
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
const MUFFIN_CLEAR: f64 = 0.0365;
/// The crumbs (`r16.py`'s `Real__breadCrumbs`, on its board) moved to the
/// middle of the muffins, on the floor
const CRUMBS_AT: (f64, f64, f64) = (-0.066, -0.0178, 0.011);

// R680 cornbread muffin cluster: 6% chance per level. Three to five
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
    // the old golden colours: which muffin (two shapes) and how brown
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
        put_split(&muffin, if k % 2 == 0 { "Real__cornMuffinA" } else { "Real__cornMuffinB" }, &crowns[k / 2]);
        let r2 = ring + (rr - 0.02) * 0.4;
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
            _ => [0.84, 0.82, 0.76],
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
            c = mix(c, [0.62, 0.52, 0.34], blot * 0.45);
            c = mix(c, [0.45, 0.36, 0.22], tide * 0.35);
            // dust on the rim, grime at the foot
            c = mix(c, [0.58, 0.56, 0.51], 0.12 + 0.2 * sstep(0.025, 0.033, y));
            c = mix(c, [0.38, 0.35, 0.30], 0.4 * sstep(0.004, 0.0, y));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The muffin, laid as `put_split` lays it (its sides round and up below,
/// its crown from above in the upper left): cornbread's golden crust gone
/// dull and grey-brown, pebbled with the coarse meal, browner at the
/// shoulder; mould colonies (the loaf's); dust on the crown. `k`: the old
/// golden colours in two pairs, a paler and a browner bake
fn muffin_tex(salt: u32, k: usize) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x3fb ^ (k as u32 * 0x91);
        let mut r = SplitMix64::new(seed, 0x10af);
        let cols: Vec<(f64, f64, f64)> = (0..7)
            .map(|i| {
                if i < 4 {
                    (r.range(0.1, 0.9) * w * 0.5, r.range(0.05, 0.45) * h, if i < 1 { r.range(6.0, 10.0) } else { r.range(2.0, 5.0) })
                } else {
                    (r.next() * w, h * r.range(0.5, 0.62), r.range(2.0, 5.0))
                }
            })
            .collect();
        let bake = if k == 0 { [0.70, 0.54, 0.30] } else { [0.62, 0.45, 0.24] };
        fill(ctx, w, h, |u, v| {
            let (px, py) = (u * w, (1.0 - v) * h);
            let top = v >= 0.5;
            let n = fbm(seed, u * 10.0, v * 10.0, 3);
            let mut c = mix(bake, [0.50, 0.44, 0.36], 0.35 + 0.15 * n);
            // the coarse meal: flecks paler and darker
            let g1 = vn(seed ^ 2, px * 0.9, py * 0.9);
            let g2 = vn(seed ^ 3, px * 1.3 + 7.0, py * 1.3);
            c = mix(c, [0.78, 0.66, 0.42], sstep(0.72, 0.9, g1) * 0.5);
            c = mix(c, [0.30, 0.22, 0.14], sstep(0.75, 0.92, g2) * 0.45);
            if top {
                // browner toward the shoulder (the crown's rim, from above)
                let fz = (u * 2.0).fract();
                let fx = (v - 0.5) * 2.0;
                let rr = ((fx * 2.0 - 1.0).powi(2) + (fz * 2.0 - 1.0).powi(2)).sqrt();
                c = mix(c, [0.40, 0.30, 0.19], sstep(0.6, 0.95, rr) * 0.35);
                c = mix(c, [0.58, 0.56, 0.51], 0.22 + 0.15 * n);
            } else {
                // the side: the crumb paler in the case, the crust above it
                let y = v * 2.0;
                c = mix(c, [0.66, 0.56, 0.38], sstep(0.62, 0.5, y) * 0.5);
            }
            let (s, wr, st) = loaf_mould(&cols, px, py, seed ^ 7);
            c = mix(c, [0.52, 0.46, 0.32], st * 0.3);
            c = mix(c, [0.80, 0.80, 0.76], wr * 0.6);
            let sp = vn(seed ^ 9, px / 1.2, py / 1.2);
            c = mix(c, [0.34 + 0.1 * sp, 0.40 + 0.08 * sp, 0.33 + 0.05 * sp], s * 0.9);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

mod kit {
    use game::assets::props::prop_part_as_modelled;
    use game::prelude::*;

    pub(super) fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    pub(super) fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    pub(super) fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    pub(super) fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    pub(super) fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v
    /// up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    pub(super) fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
        let iw = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                for k in 0..4 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    }
    /// A salt from where a thing is (no draw): its painters' strokes differ
    /// from floor to floor
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
    /// The same, its texture coordinates folded so its sides and its tops
    /// get their own places on one map: the sides (round and up, as the
    /// game lays a cylinder's) in the lower half, the faces that look up or
    /// down (the disc the game lays on them) in the upper left quarter (and
    /// a seam's copies of them right of it)
    pub(super) fn put_split(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = prop_part_as_modelled(name)?.duplicate();
        {
            let mut gg = geo.g();
            let n = gg.position.len() / 3;
            for i in 0..n {
                if gg.normal[i * 3 + 1].abs() > 0.75 {
                    gg.uv[i * 2] *= 0.5;
                    gg.uv[i * 2 + 1] = 0.5 + gg.uv[i * 2 + 1] * 0.5;
                } else {
                    gg.uv[i * 2 + 1] *= 0.5;
                }
            }
            gg.touch();
        }
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
