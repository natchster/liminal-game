//! `place_coat_racks`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::pick;
use placers_kit::place::salt;
use placers_kit::place::wall_tiles;

// Wall coat rack w/ 2-3 hanging coats: 3% chance per level. A 24 in
// board of stained hardwood screwed to the wall, five cast brass hooks on
// it (batch `t07`), two or three wool overcoats hung by their loops (r06's,
// one over the next where the hooks are close), now and then a knitted
// scarf thrown over a hook (r13b's).
pub(crate) fn place_coat_racks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let wall_tiles = wall_tiles(g);
    if wall_tiles.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wall_tiles[(rnd() * wall_tiles.len() as f64).floor() as usize];
    // the old wood draw: a stain (mid oak, walnut, golden oak, dark)
    let wood_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x504030];
    let wc = pick(&wood_colors);
    let wi = wood_colors.iter().position(|&c| c == wc).unwrap_or(0);
    let salt = salt(g.wx(x), g.wz(z), wi as u32);
    let stain = [[0.40, 0.26, 0.15], [0.26, 0.17, 0.11], [0.50, 0.36, 0.22], [0.24, 0.19, 0.14]][wi];
    let rack = group();
    let mount_y = 1.75;
    // the wall's frame: the board's back on the face, its middle at mount_y
    let wall = group();
    wall.position().set(0, mount_y, WALL_Z);
    put(&wall, "Real__coatRackBoard", &lambert!({ map: &board_tex(stain, salt) }));
    put(&wall, "Real__coatRackScrews", &phong!({ color: 0x5c5850, specular: 0x3a3a36, shininess: 30 }));
    put(&wall, "Real__coatRackPegs", &phong!({ color: 0x6a5632, specular: 0x5a4a2c, shininess: 34 }));
    rack.add(&wall);
    let peg_positions: Vec<f64> = (0..5).map(|i| -0.24 + i as f64 * 0.12).collect();
    // the old coat colours (rich, saturated) as wool after years
    let coat_colors = [0x603030, 0x101020, 0x408060, 0x201010, 0xa02040, 0x203060, 0x604020, 0x2a2a2a];
    let wools = [0x5a3a36u32, 0x2f3442, 0x4a5444, 0x3a2e2a, 0x5e2e32, 0x3e4858, 0x7a6249, 0x464648];
    let wool_map = wool_tex(salt);
    let horn = phong!({ color: 0x2a2018, specular: 0x202020, shininess: 40 });
    // Hang 2-3 coats on random pegs (deduplicated).
    let coat_count = 2 + (rnd() * 2.0).floor() as i32;
    let mut used_pegs: Vec<usize> = vec![]; // a Set
    for _c in 0..coat_count {
        let mut peg_idx;
        let mut attempts = 0;
        loop {
            peg_idx = (rnd() * peg_positions.len() as f64).floor() as usize;
            attempts += 1;
            if !(used_pegs.contains(&peg_idx) && attempts < 10) {
                break;
            }
        }
        if used_pegs.contains(&peg_idx) {
            break;
        }
        let px = peg_positions[peg_idx];
        // over how many coats hung before it, near enough to lie on
        let layer = used_pegs.iter().filter(|&&j| (1..4).contains(&j) && (peg_positions[j] - px).abs() < 0.45).count();
        used_pegs.push(peg_idx);
        let cc = pick(&coat_colors);
        let ci = coat_colors.iter().position(|&c| c == cc).unwrap_or(0);
        let coat = group();
        if (1..4).contains(&peg_idx) {
            // an overcoat, its loop on the hook's cradle, slid out a little
            // over another's
            coat.position().set(px, CRADLE.0, CRADLE.1 + 0.004 * layer as f64);
            let push = 0.06 * layer as f64;
            if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__coatHung") {
                let geo = if layer > 0 { pushed(&geo, push) } else { geo };
                coat.add(&mesh(geo, lambert!({ map: &wool_map, color: wools[ci], side: DoubleSide })));
            }
            if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__hookCoatButtons") {
                let geo = if layer > 0 { pushed(&geo, push) } else { geo };
                coat.add(&mesh(geo, &horn));
            }
        } else {
            // on an end hook, where a coat would stand out past the board
            // (and a wall's end), a scarf thrown over it
            coat.position().set(px, CRADLE.0, CRADLE.1);
            if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__scarfHung") {
                let s = mesh(geo, lambert!({ map: &knit_tex(salt ^ ci as u32), color: wools[ci], side: DoubleSide }));
                s.position().set(0, 0.0465, -0.058);
                coat.add(&s);
            }
        }
        wall.add(&coat);
        // Subtle sway animator — 35% chance per coat.
        if rnd() < 0.35 {
            let sway_phase = rnd() * PI * 2.0;
            let sway_phase2 = rnd() * PI * 2.0;
            let coat2 = coat.clone();
            g.animate(move |_g, _dt, t| {
                coat2.rotation().x = (t * 0.5 + sway_phase).sin() * 0.015;
                coat2.rotation().z = (t * 0.4 + sway_phase2).sin() * 0.010;
            });
        }
    }
    // Optional: a scarf thrown over one empty hook (40% if empty pegs exist).
    if used_pegs.len() < 5 && rnd() < 0.4 {
        let mut scarf_peg_idx;
        loop {
            scarf_peg_idx = (rnd() * peg_positions.len() as f64).floor() as usize;
            if !used_pegs.contains(&scarf_peg_idx) {
                break;
            }
        }
        let scarf_px = peg_positions[scarf_peg_idx];
        let scarf_colors = [0xa02040, 0x408060, 0xd0a040, 0x203060, 0xf0e0a0];
        let sc = pick(&scarf_colors);
        let wool = [0x6a3034u32, 0x4a5a48, 0x8a7244, 0x3c4656, 0x9a9282][scarf_colors.iter().position(|&c| c == sc).unwrap_or(0)];
        if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__scarfHung") {
            // its frame is r06's hook's: the arm's top at (0, -0.0465, 0.058)
            let s = mesh(geo, lambert!({ map: &knit_tex(salt ^ 0x5ca), color: wool, side: DoubleSide }));
            s.position().set(scarf_px, CRADLE.0 + 0.0465, CRADLE.1 - 0.058);
            wall.add(&s);
        }
    }
    rack.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    rack.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&rack);
}

/// The wall's face in a wall prop's frame (its group stands 15 mm out)
const WALL_Z: f64 = -0.015;

/// A hook's cradle, where a loop rests: its top, out from the wall (batch
/// `t07`'s brass hooks, as r06's: r06's coat and r13b's scarf hang there)
const CRADLE: (f64, f64) = (-0.0005, 0.077);

/// A copy of `geo` pushed out from the wall below its top: a coat hung
/// over the one under it (its loop on its own hook, its body lying on the
/// other's)
fn pushed(geo: &Geo, push: f64) -> Geo {
    let out = geo.duplicate();
    {
        let mut gg = out.g();
        for i in 0..gg.position.len() / 3 {
            let y = gg.position[i * 3 + 1] as f64;
            gg.position[i * 3 + 2] += (push * sstep(0.0, -0.32, y)) as f32;
        }
        gg.touch();
    }
    out.compute_bounding_box();
    out.compute_bounding_sphere();
    out
}

/// The board's face (u along it, v up it): stained hardwood, its grain
/// running along, the varnish worn pale where hands reached for the hooks,
/// grime in the bead and along its foot, dust on its top edge
fn board_tex(stain: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(512, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0xb0a;
        fill(ctx, w, h, |u, v| {
            let grain = fbm(seed, u * 6.0, v * 40.0 + fbm(seed ^ 1, u * 3.0, v * 3.0, 2) * 6.0, 4);
            let lines = (vn(seed ^ 2, u * 3.0, v * 160.0) - 0.5).abs();
            let mut c = mix(stain, [stain[0] * 0.62, stain[1] * 0.6, stain[2] * 0.58], grain * 1.3 - 0.2);
            c = mix(c, [stain[0] * 0.5, stain[1] * 0.48, stain[2] * 0.46], sstep(0.12, 0.0, lines) * 0.5);
            // rubbed pale round each hook, where coats were taken down
            let mut rub: f64 = 0.0;
            for k in 0..5 {
                let du = (u - (0.5 + (-0.24 + 0.12 * k as f64) / 0.61)) * 0.61 / 0.03;
                let dv = (v - 0.5) * 0.09 / 0.03;
                rub = rub.max((-(du * du + dv * dv)).exp());
            }
            c = mix(c, mix(stain, [0.62, 0.56, 0.46], 0.4), rub * 0.45);
            c = mix(c, [0.16, 0.13, 0.10], sstep(0.12, 0.0, v) * 0.4 + sstep(0.86, 1.0, v) * 0.15);
            c = mix(c, [0.44, 0.42, 0.38], fbm(seed ^ 7, u * 9.0, v * 4.0, 3) * 0.18);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Wool cloth after years on a hook (white, for its colour to tint): a
/// twill, fuzz and pills, faded on the shoulders, dust lying on them
fn wool_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, true, move |ctx, w, h| {
        let seed = salt ^ 0x3001;
        fill(ctx, w, h, |u, v| {
            let tw = ((u * w + v * h) / 3.0).fract();
            let twill = if tw < 0.5 { 0.94 } else { 1.0 };
            let n = fbm(seed, u * 16.0, v * 16.0, 4);
            let pill = vn(seed ^ 9, u * 140.0, v * 140.0);
            let k = twill * (0.86 + 0.14 * n) * if pill > 0.88 { 1.08 } else { 1.0 };
            let dust = sstep(0.75, 1.0, v) * 0.12;
            [k * (1.0 - dust) + dust * 0.9, k * (1.0 - dust) + dust * 0.88, k * (1.0 - dust) + dust * 0.84, 1.0]
        });
    })
}

/// A hand-knitted scarf (white, tinted): ribbed down its length, fuzz and
/// pills, a moth hole
fn knit_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x9e7;
        fill(ctx, w, h, |u, v| {
            let rib = (u * 34.0).fract();
            let stitch = ((v * 120.0 + if rib < 0.5 { 0.0 } else { 0.5 }).fract() - 0.5).abs();
            let k = (if rib < 0.42 { 0.84 } else { 1.0 }) * (0.92 + 0.08 * stitch) * (0.9 + 0.1 * fbm(seed, u * 10.0, v * 20.0, 3));
            [k, k * 0.99, k * 0.97, 1.0]
        });
        for _ in 0..(salt % 7) {
            let _ = rnd();
        }
        ctx.set_fill_style("rgba(10,8,6,0.8)");
        ctx.begin_path();
        ctx.ellipse(rnd() * w, h * (0.3 + 0.6 * rnd()), 1.5 + rnd() * 1.5, 1.0 + rnd(), rnd() * 3.0, 0, 7);
        ctx.fill();
    })
}
