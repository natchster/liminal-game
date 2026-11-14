//! `place_cake_stands`
use crate::shared::put;
use crate::shared::put_split;
use game::prelude::*;
use placers_kit::baking::mould;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt;

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
                    let f = 0.85 + (t * 5.0 + flicker_phase).dsin() * 0.10 + (t * 9.0 + flicker_phase2).dsin() * 0.06;
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

/// The cake as modelled (`party/a_party_tables_leavings.py`): the bottom tier's radius; the cake
/// part's heights (its bounds) and where the top tier's foot is
const CAKE_R1: f64 = 0.098;

const CAKE_Y: (f64, f64) = (0.1152, 0.2572);

const CAKE_TIER2: f64 = 0.1915;

/// The stand's plate (its bounds: half width, and heights)
const STAND_PLATE: (f64, f64, f64) = (0.139_866, 0.1000, 0.1203);

/// The candle's flame root, over the wick (`party/a_party_tables_leavings.py` prints it)
const FLAME_ROOT: (f64, f64, f64) = (0.0208, 0.2881, 0.0107);

/// `t09`'s flame (`Real__brassStickFlameB`): its foot's height; and how
/// much smaller a birthday candle's flame is than a dinner candle's
const FLAME_FOOT: f64 = 0.170;

const FLAME_SCALE: f64 = 0.55;

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
                    (r.next() * w, h - r.next().dpowf(1.8) * h * 0.4, if k < 2 { r.range(10.0, 18.0) } else { r.range(3.0, 7.0) })
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
            let (s, wr, st) = mould(&cols, px, py, seed ^ 7);
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
