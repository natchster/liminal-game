//! `place_hanging_brass_lanterns`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt;

// Hanging brass lantern: 4% chance per level, hanging from the
// ceiling. A small brass candle lantern of the Victorian hall kind (Blender
// family `four_larger_things`): a square tray on a turned drop, four panes between corner
// posts (the front one a door on two hinges), a pyramid roof up to a
// vented chimney, its ring on a jack chain from a cup on the ceiling. The
// brass gone brown and dull, the panes sooted from inside where the flame
// licked; a candle burnt down in its cup, lit, its flame a flame and its
// light the old light, flickering as before; it sways on its chain (from
// the ceiling's hook now, not from the floor) as the old one swayed.
pub(crate) fn place_hanging_brass_lanterns(g: &mut Game, _dist: &[i32]) {
    g.pp.brass_lantern_positions.clear(); // reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    // the old draws, call for call: the chain's length, the flicker's phase
    // and rate, the slide off the tile, the turn, whether and how it sways
    let chain_len = 0.5 + rnd() * 0.4;
    let lantern_y = WALL_H - chain_len - 0.10;
    let flicker_phase = rnd() * PI * 2.0;
    let flicker_rate = 5.0 + rnd() * 3.0;
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let turn = rnd() * PI * 2.0;
    let is_swaying = rnd() < 0.60;
    let sway = if is_swaying { Some((rnd() * PI * 2.0, 8.0 + rnd() * 6.0)) } else { None };
    let salt = salt(px, pz, 21);
    let brass = phong!({ map: &tarnish_tex(salt), color: 0x8a6a34, specular: 0x6a5430, shininess: 30 });
    let lantern = group();
    // it hangs from the hook in the ceiling's cup and swings from there
    let pivot = group();
    pivot.position().y = WALL_H + PIVOT;
    lantern.add(&pivot);
    if let Some(m) = put(&pivot, "Real__lanternCanopy", &brass) {
        m.position().y = CANOPY_TOP - PIVOT;
    }
    // the lantern, its light at the old light's height
    let body = group();
    body.position().y = lantern_y - (WALL_H + PIVOT);
    pivot.add(&body);
    put(&body, "Real__lanternBrass", &brass);
    // the roof's underside, black with the candle's soot
    put(&body, "Real__lanternSoot", &lambert!({ color: 0x1d1915 }));
    if let Some(m) = put(&body, "Real__lanternGlass", &lantern_glass_mat(&soot_tex(salt))) {
        m.set_render_order(2);
    }
    put(&body, "Real__lanternCandle", &lambert!({ map: &wax_tex(salt) }));
    // t09's wick and flame (its stick's B, burnt to 38 mm), stood in this cup
    let stick = group();
    stick.position().y = LT_SOCKET_FLOOR - STICK_SOCKET_FLOOR;
    body.add(&stick);
    put(&stick, "Real__brassStickWickB", &lambert!({ color: 0x1c1915 }));
    // the flame scales from its root as it flickers
    let flame = group();
    flame.position().y = LT_FLAME_ROOT;
    body.add(&flame);
    if let Some(m) = put(&flame, "Real__brassStickFlameB", &basic_mat!({ map: &flame_tex(), transparent: true, depthWrite: false })) {
        m.position().y = LT_SOCKET_FLOOR - STICK_SOCKET_FLOOR - LT_FLAME_ROOT;
        m.set_render_order(3);
    }
    // the old light (0xffb060, 0.7, 1.5, decay 2), 10 mm over the flame's root
    let glow = point_light!(0xffb060, 0.7, 1.5, 2);
    glow.position().y = 0.0;
    body.add(&glow);
    // the chain: from the ring up into the ceiling's cup, link by link, in
    // runs of 16, 4 and 1 (each run turned so the links keep alternating)
    let foot = lantern_y + LT_CHAIN_FOOT;
    let n = ((WALL_H - 0.003 - foot - LINK_L) / LINK_P).floor() as i32 + 1;
    let mut at = 0;
    for (k, name) in [(16, "Real__lanternChain16"), (4, "Real__lanternChain4"), (1, "Real__lanternChain1")] {
        while n - at >= k {
            if let Some(m) = put(&pivot, name, &brass) {
                m.position().y = foot + at as f64 * LINK_P - (WALL_H + PIVOT);
                if at % 2 == 1 {
                    m.rotation().y = PI / 2.0;
                }
            }
            at += k;
        }
    }
    // the old flicker: the light's intensity and the flame's height
    {
        let (glow, flame) = (glow.clone(), flame.clone());
        g.animate(move |_g, _dt, t| {
            let p = t * flicker_rate + flicker_phase;
            let wobble = p.sin() * 0.5 + (p * 1.7).sin() * 0.3;
            glow.set_intensity(0.7 + wobble * 0.15);
            flame.scale().y = 1.0 + wobble * 0.10;
        });
    }
    lantern.position().set(px, 0, pz);
    lantern.rotation().y = turn;
    // the old slow sway (its phase and period), about the ceiling's hook
    if let Some((sway_phase, sway_period)) = sway {
        let pivot2 = pivot.clone();
        g.animate(move |_g, _dt, t| {
            pivot2.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.03;
        });
    }
    g.world.add(&lantern);
    // Track swaying lanterns for creak audio.
    if is_swaying {
        let lp = lantern.pos();
        g.pp.brass_lantern_positions.push(v3(lp.x, lantern_y, lp.z));
    }
}

/// The lantern (`victorian/four_larger_things.py` LT_*, LINK_*): where the chain's foot hooks into
/// its ring above the light (y = 0 at the light), the chain's pitch and a
/// link's length; the candle's cup floor below the light and the flame's
/// root as t09's stick has it (`Real__brassStickFlameB` stands on a floor
/// at 0.133)
const LT_CHAIN_FOOT: f64 = 0.2295 + 0.0130 + 0.0025 - 0.0024;

const LINK_P: f64 = 0.022 - 0.0024;

const LINK_L: f64 = 0.022;

const LT_SOCKET_FLOOR: f64 = -0.047;

const STICK_SOCKET_FLOOR: f64 = 0.133;

const LT_FLAME_ROOT: f64 = LT_SOCKET_FLOOR + 0.037;

/// The ceiling cup: its top 2 mm into the ceiling (as the old chain's top),
/// 30 mm deep; the hook the lantern swings from 12 mm under the ceiling
const CANOPY_TOP: f64 = 0.002;

const PIVOT: f64 = -0.012;

/// Glass as the windows' and t09's (`Patch::Glass`): the room seen in it by
/// Fresnel; the candle's soot on its inside (the map's grey, `f[8]` of it)
fn lantern_glass_mat(map: &Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 220, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "lantern_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it; the flame behind it is its own light
        f[0] = 0.40;
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // soot: how much of the map's grey, its colour (linear)
        f[8] = 0.7;
        f[9] = 0.035;
        f[10] = 0.031;
        f[11] = 0.027;
        f[12] = 0.0;
        // two faces of glass: 4% each, square on
        f[13] = 0.07;
    }
    m
}

/// The candle's soot on the panes (laid round them, v up): a brown-black
/// film thickening toward their heads, heaviest over the flame (the pane's
/// middle), streaked where it ran with condensation; dust on the outside
fn soot_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x50a7;
        fill(ctx, w, h, |u, v| {
            let mid = 1.0 - sstep(0.0, 0.5, (u - 0.5).abs());
            let up = sstep(0.55, 1.0, v);
            let n = fbm(seed, u * 6.0, v * 9.0, 3);
            let streak = (vn(seed ^ 3, u * 40.0, v * 3.0) - 0.6).max(0.0) * 0.6;
            let dust = 0.06 + 0.08 * vn(seed ^ 7, u * 30.0, v * 30.0);
            let g = (dust + up * (0.30 + 0.35 * mid) * (0.7 + 0.5 * n) - streak * up).clamp(0.0, 1.0);
            [g, g, g, 1.0]
        });
    })
}

/// Brass gone dull (t09's: grey on the brass's colour, how bright): brown
/// tarnish in patches, darker spots, the black of soot round the chimney
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
            // soot up the roof and the chimney (the part's top fifth)
            c = mix(c, [0.16, 0.15, 0.14], sstep(0.80, 0.92, v) * (0.5 + 0.4 * n));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The candle's wax (t09's, laid round it, v up): ivory gone grey with
/// dust, the runs a shade clearer, soot near its top
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

/// A candle flame (t09's, laid round it, v up its 27 mm): a dim blue root,
/// the bright yellow-white body, orange toward its tip, soft at its edge
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

/// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = game::assets::props::prop_part_as_modelled(name)?;
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}
