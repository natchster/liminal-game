//! Batch `t22`'s feather boa, made real (`docs/PROPS.md` P3, "the party
//! and the shrine"; the Blender batch `assets/src/blender/props/t22.py`,
//! its notes `t22.notes.md`): a child's short chandelle boa dropped on the
//! floor. It keeps the old placer's draws, call for call, and lies along
//! the old one's curve (so it covers the floor the old one did).
use crate::{open_tiles, pick};
use game::prelude::*;

/// R438 small feather boa: 6% chance per level. A child's dress-up boa
/// (turkey feathers twisted on a cord, chandelle, about 90 mm across)
/// dropped where a costume came off, lying in a lazy curve; its dye faded
/// and dusty (the old colours: hot pink, purple, gold, royal blue, white,
/// black), the feathers flattened where it lies. Made of twelve lengths of
/// it (three modelled), overlapping along the curve. Party-costume family.
pub fn place_small_feather_boas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let boa_colors = [0xd85090, 0xa020a0, 0xd8b820, 0x2050d8, 0xf0f0f0, 0x181008];
    let boa_color = pick(&boa_colors);
    // the old colour: its dye, faded and dusty (straight sRGB)
    let dye = match boa_colors.iter().position(|&c| c == boa_color).unwrap_or(0) {
        0 => [0.74, 0.36, 0.52],
        1 => [0.46, 0.22, 0.46],
        2 => [0.76, 0.62, 0.26],
        3 => [0.22, 0.32, 0.62],
        4 => [0.80, 0.78, 0.74],
        _ => [0.13, 0.12, 0.11],
    };
    let props = game::assets::props::prop_part_as_modelled;
    let lengths: Vec<(Option<Geo>, Option<Geo>)> =
        ["A", "B", "C"].iter().map(|v| (props(&format!("Real__boaCore{v}")), fluff(&format!("Real__boaFeathers{v}")))).collect();
    let core_mat = lambert!({ color: hex([dye[0] * 0.8, dye[1] * 0.8, dye[2] * 0.8]) });
    let feather_mat = lambert!({ color: hex(dye) });
    let boa = group();
    // The old curve it lay in, its lengths 4.5 cm apart along it
    let n_segments = 12usize;
    let total_len = 0.7 + rnd() * 0.4;
    let curve_amp = 0.20;
    let curve_phase = rnd() * PI * 2.0;
    let curve = |t: f64| -> (f64, f64) { ((t - 0.5) * total_len, (t * PI + curve_phase).sin() * curve_amp) };
    let mut arc: Vec<f64> = vec![0.0];
    for k in 1..=64usize {
        let (ax, az) = curve((k - 1) as f64 / 64.0);
        let (bx, bz) = curve(k as f64 / 64.0);
        let v = arc[k - 1] + hypot2(bx - ax, bz - az);
        arc.push(v);
    }
    let shrink = 0.045 * (n_segments - 1) as f64 / arc[64];
    for i in 0..n_segments {
        let t = i as f64 / (n_segments - 1) as f64;
        let mut k = 1usize;
        while k < 64 && arc[k] < t * arc[64] {
            k += 1;
        }
        let p = ((k - 1) as f64 + (t * arc[64] - arc[k - 1]) / (arc[k] - arc[k - 1])) / 64.0;
        let (ca, cp) = curve(p);
        // the way the curve runs there
        let (da, dp) = (total_len, (p * PI + curve_phase).cos() * curve_amp * PI);
        // the old puff's draws: its size (a length a little fuller or
        // thinner), how far along it sits, one kept; its turn (a little
        // askew). The lengths take turns (four of each, so each is drawn
        // instanced, as the old puffs were)
        let size = rnd();
        let slide = rnd();
        let _ = rnd(); // keeps the old draw (the puff's depth)
        let askew = rnd();
        let (core, feathers) = &lengths[i % 3];
        let len = group();
        for (geo, mat) in [(core, &core_mat), (feathers, &feather_mat)] {
            if let Some(geo) = geo {
                len.add(&mesh(geo.clone(), mat));
            }
        }
        let s = 0.92 + 0.16 * size;
        len.scale().set(s, s, s);
        let nudge = (slide - 0.5) * 0.008;
        let (ux, uz) = (da / hypot2(da, dp), dp / hypot2(da, dp));
        len.position().set(ca * shrink + ux * nudge, 0.0, cp * shrink + uz * nudge);
        len.rotation().y = (-dp).atan2(da) + (askew - 0.5) * 0.3;
        boa.add(&len);
    }
    boa.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    boa.rotation().y = rnd() * PI * 2.0;
    g.world.add(&boa);
}

/// The feathers `name` lit as fluff: their thin barbs drawn from both
/// sides (each twice, wound both ways), each one's normal turned out from the cord and up toward the
/// light above, so the mass shades as a soft rope, not as blades catching
/// the lamp one by one. A copy of the part, made once and shared
fn fluff(name: &str) -> Option<Geo> {
    thread_local! { static C: std::cell::RefCell<Vec<(String, Geo)>> = const { std::cell::RefCell::new(vec![]) }; }
    if let Some(g) = C.with(|c| c.borrow().iter().find(|(n, _)| n == name).map(|(_, g)| g.clone())) {
        return Some(g);
    }
    let geo = game::assets::props::prop_part_as_modelled(name)?.duplicate();
    {
        let mut gg = geo.g();
        let n = gg.position.len() / 3;
        for i in 0..n {
            // out from the cord (along x, 28 mm up), and up
            let (y, z) = (f64::from(gg.position[i * 3 + 1]) - 0.028, f64::from(gg.position[i * 3 + 2]));
            let r = (y * y + z * z).sqrt().max(1e-6);
            let (nx, ny, nz) = (0.0, 0.85 * y / r + 0.4, 0.85 * z / r);
            let l = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-6);
            gg.normal[i * 3] = (nx / l) as f32;
            gg.normal[i * 3 + 1] = (ny / l) as f32;
            gg.normal[i * 3 + 2] = (nz / l) as f32;
        }
        // each barb twice, wound both ways, sharing those normals (a
        // two-sided material would light its back with them reversed)
        if let Some(idx) = gg.index.as_mut() {
            let back: Vec<u32> = idx.chunks_exact(3).flat_map(|t| [t[0], t[2], t[1]]).collect();
            idx.extend(back);
        }
        gg.touch();
    }
    C.with(|c| c.borrow_mut().push((name.to_string(), geo.clone())));
    Some(geo)
}

/// A colour's hex from straight sRGB 0..1
fn hex(c: [f64; 3]) -> u32 {
    let b = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
    b(c[0]) << 16 | b(c[1]) << 8 | b(c[2])
}
