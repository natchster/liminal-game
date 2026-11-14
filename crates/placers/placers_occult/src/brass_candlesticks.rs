//! `place_brass_candlesticks`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_tile;
use placers_kit::place::salt;

// Brass candlestick with wax drips: 6% chance per level. A turned and
// cast brass candlestick (15.5 cm) of the common kind, its brass gone dull
// and brown, polished only where hands took it; its taper (a 7/8 in dinner
// candle) burnt down, the wax run over the socket's rim and onto the pan
// and the foot, the wick charred. Lit as the old one was: its flame a
// flame, its light the old light.
pub(crate) fn place_brass_candlesticks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    // the old three drips' lengths: the first says how far the taper has
    // burnt (a long stub or a short one)
    let short = rnd() >= 0.5;
    let _ = rnd();
    let _ = rnd();
    // the old wax pool's eight radii (the wax is the candle's own runs now)
    for _ in 0..8 {
        let _ = rnd();
    }
    let lit = rnd() < 0.60;
    let flicker = if lit { Some((rnd() * PI * 2.0, 6.0 + rnd() * 4.0)) } else { None };
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let k = if short { "B" } else { "A" };
    let salt = salt(px, pz, 3);
    let stick = group();
    put(&stick, "Real__brassStickBody", &phong!({ map: &tarnish_tex(salt), color: 0x8a6a34, specular: 0x6a5430, shininess: 30 }));
    put(&stick, &format!("Real__brassStickCandle{k}"), &lambert!({ map: &wax_tex(salt) }));
    put(&stick, &format!("Real__brassStickWick{k}"), &lambert!({ color: 0x1c1915 }));
    if let Some((flicker_phase, flicker_rate)) = flicker {
        let root = STICK_FLAME_ROOT[usize::from(short)];
        // the flame scales from its root as it flickers
        let flame = group();
        flame.position().y = root;
        if let Some(m) =
            put(&flame, &format!("Real__brassStickFlame{k}"), &basic_mat!({ map: &flame_tex(), transparent: true, depthWrite: false }))
        {
            m.position().y = -root;
            m.set_render_order(3);
        }
        stick.add(&flame);
        let glow = point_light!(0xffb060, 0.35, 0.9, 2);
        glow.position().y = root + 0.010;
        stick.add(&glow);
        g.animate(move |_g, _dt, t| {
            let p = t * flicker_rate + flicker_phase;
            let wobble = p.dsin() * 0.5 + (p * 1.7).dsin() * 0.3;
            glow.set_intensity(0.35 + wobble * 0.08);
            flame.scale().y = 1.0 + wobble * 0.12;
        });
    }
    stick.position().set(px, 0, pz);
    stick.rotation().y = turn;
    g.world.add(&stick);
}

/// The candle's stub (`occult/a_shrines_things.py`): the flame's root over the socket's floor
/// (m), the taper burnt down to 70 mm (A) or to a 38 mm stub (B)
const STICK_FLAME_ROOT: [f64; 2] = [0.133 + 0.069, 0.133 + 0.037];

/// Brass gone dull (grey on the brass's colour: how bright): brown tarnish
/// in patches, darker spots, a polished streak where it was held
fn tarnish_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xb7a;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 6.0, v * 6.0, 3);
            let mut c = mix([0.42, 0.38, 0.33], [0.85, 0.82, 0.76], sstep(0.4, 0.75, n));
            let spot = vn(seed ^ 3, u * 40.0, v * 40.0);
            if spot > 0.72 {
                c = mix(c, [0.26, 0.24, 0.21], (spot - 0.72) * 3.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The candle's wax (laid round it, v up): ivory gone grey with dust, the
/// runs a shade clearer, soot near its top
fn wax_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x3a5;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 5.0, v * 12.0, 3);
            let mut c = mix([0.86, 0.82, 0.72], [0.78, 0.74, 0.64], n);
            c = mix(c, [0.40, 0.37, 0.33], sstep(0.88, 1.0, v) * 0.35);
            let dust = vn(seed ^ 5, u * 90.0, v * 90.0);
            if dust > 0.8 {
                c = mix(c, [0.62, 0.60, 0.55], (dust - 0.8) * 2.5);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A candle flame (laid round it, v up its 27 mm): a dim blue root, the
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
