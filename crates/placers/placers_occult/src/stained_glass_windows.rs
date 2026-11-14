//! `place_stained_glass_windows`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::hang_on_wall;
use placers_kit::place::pick_hex;
use placers_kit::place::salt;
use placers_kit::place::wall_candidates;

// Stained glass window: 4% chance per level. A Victorian leaded panel
// of the terraced-house kind (Blender family `four_larger_things`), taken out of a door or
// a landing window and hung on the wall in a dark-stained pine frame: a
// border of coloured strips with a jewel square at each corner, a field of
// pale rolled glass with a lozenge round a smaller one; the lead gone dull
// grey, the cement black along it, an iron saddle bar across its middle.
// Its glass coloured but muted as old glass is, seen against the wall
// behind it and lit by the room; it gives no light (the old coloured
// light is gone, its draw kept: it chooses the colours).
pub(crate) fn place_stained_glass_windows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let window = group();
    let center_y = 1.70;
    hang_on_wall(g, &window, &c, 0.010, center_y);
    // the old light's colour (no light: old glass on a wall gives none):
    // which colour leads the panel
    let lit_colors: [u32; 3] = [0xf0d020, 0xd8b040, 0xa02040];
    let picked = pick_hex(&lit_colors);
    let lead = lit_colors.iter().position(|&k| k == picked).unwrap_or(0);
    let p = window.pos();
    let salt = salt(p.x, p.z, 0x5617);
    put(&window, "Real__stainedGlassFrame", &lambert!({ map: &frame_tex(salt) }));
    put(&window, "Real__stainedGlassPanes", &phong!({ map: &glass_tex(salt, lead), specular: 0x2e2e2e, shininess: 70 }));
    put(&window, "Real__stainedGlassLead", &phong!({ color: 0x4e4d4a, specular: 0x1c1c1c, shininess: 18 }));
    put(&window, "Real__stainedGlassBar", &lambert!({ color: 0x3b3029 }));
    g.world.add(&window);
}

/// The panel (`victorian/four_larger_things.py` SG_*): the glass in sight, the border's width, the
/// two lozenges' half-widths and half-heights; the panes' part a little
/// larger than the sight (its edge under the frame)
const SG_W: f64 = 0.300;

const SG_H: f64 = 0.440;

const SG_B: f64 = 0.032;

const SG_OUT: (f64, f64) = (0.085, 0.130);

const SG_INNER: (f64, f64) = (0.036, 0.055);

const SG_PANES: (f64, f64) = (SG_W + 0.006, SG_H + 0.006);

const SG_FRAME: (f64, f64) = (0.340, 0.480);

/// The cames' centre lines, as `victorian/four_larger_things.py` `sg_lines` lays them
fn lines() -> Vec<[f64; 4]> {
    let (hw, hh) = (SG_W / 2.0, SG_H / 2.0);
    let (bx, by) = (hw - SG_B, hh - SG_B);
    let (ox, oy) = SG_OUT;
    let (ix, iy) = SG_INNER;
    let mut l = vec![[-bx, -by, bx, -by], [bx, -by, bx, by], [bx, by, -bx, by], [-bx, by, -bx, -by]];
    for sx in [-1.0, 1.0] {
        for sy in [-1.0, 1.0] {
            l.push([sx * bx, sy * by, sx * hw, sy * by]);
            l.push([sx * bx, sy * by, sx * bx, sy * hh]);
        }
        l.push([sx * bx, 0.0, sx * hw, 0.0]);
    }
    for (ax, ay) in [SG_OUT, SG_INNER] {
        l.extend([[0.0, ay, ax, 0.0], [ax, 0.0, 0.0, -ay], [0.0, -ay, -ax, 0.0], [-ax, 0.0, 0.0, ay]]);
    }
    l.extend([[0.0, oy, 0.0, by], [0.0, -oy, 0.0, -by], [ox, 0.0, bx, 0.0], [-ox, 0.0, -bx, 0.0]]);
    l.extend([[0.0, iy, 0.0, oy], [0.0, -iy, 0.0, -oy], [ix, 0.0, ox, 0.0], [-ix, 0.0, -ox, 0.0]]);
    l
}

/// Which piece of glass (x, y) is in: its kind (0 a corner jewel, 1 a
/// border strip, 2 the field, 3 the ring of the lozenge, 4 its centre) and
/// a number for the piece
fn piece(x: f64, y: f64) -> (usize, i32) {
    let (bx, by) = (SG_W / 2.0 - SG_B, SG_H / 2.0 - SG_B);
    let q = i32::from(x > 0.0) + 2 * i32::from(y > 0.0);
    if x.abs() > bx && y.abs() > by {
        return (0, q);
    }
    if y.abs() > by {
        return (1, 4 + i32::from(y > 0.0));
    }
    if x.abs() > bx {
        return (1, 6 + q);
    }
    let (ox, oy) = SG_OUT;
    let (ix, iy) = SG_INNER;
    if x.abs() / ix + y.abs() / iy < 1.0 {
        (4, 20)
    } else if x.abs() / ox + y.abs() / oy < 1.0 {
        (3, 12 + q)
    } else {
        (2, 16 + q)
    }
}

/// The glass as it shows on the wall (laid across the panes' part: u right,
/// v up): each piece its sheet's colour (the lead colour leads the
/// border, the jewels and the lozenge), muted, the light through it and
/// back off the wall; cathedral glass rippled, seeds in the rolled glass,
/// a tone apart from piece to piece; the black cement along the lead, the
/// grime of years in each piece's foot, dust along the top of every came
/// that lies flat, one piece cracked across
fn glass_tex(salt: u32, lead: usize) -> Tex {
    canvas_tex_opt(192, 280, false, move |ctx, w, h| {
        let seed = salt ^ 0x6e55;
        let segs = lines();
        // the colours by lead: [jewels, border, field, ring, centre] (sRGB)
        const AMBER: [f64; 3] = [0.56, 0.40, 0.17];
        const GOLD: [f64; 3] = [0.60, 0.50, 0.22];
        const RUBY: [f64; 3] = [0.38, 0.10, 0.10];
        const BLUE: [f64; 3] = [0.14, 0.19, 0.34];
        const GREEN: [f64; 3] = [0.20, 0.32, 0.21];
        const PALE: [f64; 3] = [0.50, 0.51, 0.45];
        let pal = match lead {
            0 => [BLUE, GOLD, PALE, GREEN, RUBY],
            1 => [RUBY, GREEN, PALE, AMBER, BLUE],
            _ => [GREEN, AMBER, PALE, RUBY, GOLD],
        };
        let cracked = 16 + (hash(seed, 9, 9) * 4.0) as i32;
        fill(ctx, w, h, |u, v| {
            let x = (u - 0.5) * SG_PANES.0;
            let y = (v - 0.5) * SG_PANES.1;
            let (kind, id) = piece(x, y);
            let mut c = pal[kind];
            // each piece cut from its own sheet: a tone apart
            let tone = 0.88 + 0.22 * hash(seed, id, 1);
            c = c.map(|k| k * tone);
            // cathedral glass: ripples along the sheet's roll; the rolled
            // field glass: seeds (small bubbles)
            let ang = hash(seed, id, 2) * PI;
            let (sa, ca) = ang.sin_cos();
            let (ru, rv) = (x * ca + y * sa, -x * sa + y * ca);
            let rip = fbm(seed ^ id as u32, ru * 30.0, rv * 140.0, 3);
            c = c.map(|k| k * (0.84 + 0.30 * rip));
            if kind == 2 {
                let sd = vn(seed ^ 0x5eed, x * 900.0, y * 900.0);
                if sd > 0.86 {
                    c = mix(c, [0.62, 0.62, 0.58], (sd - 0.86) * 4.0);
                }
            }
            // the lead's cement, black along each came, and the dust lying on
            // the top of the flat ones
            let mut d = f64::INFINITY;
            let mut above_flat = f64::INFINITY;
            for s in &segs {
                let (ax, ay, bx2, by2) = (s[0], s[1], s[2], s[3]);
                let (dx, dy) = (bx2 - ax, by2 - ay);
                let l2 = dx * dx + dy * dy;
                let t = (((x - ax) * dx + (y - ay) * dy) / l2).clamp(0.0, 1.0);
                let (px, py) = (ax + t * dx - x, ay + t * dy - y);
                let dd = (px * px + py * py).sqrt();
                d = d.min(dd);
                if dy.abs() < 1e-9 && y > ay && t > 0.0 && t < 1.0 {
                    above_flat = above_flat.min(y - ay);
                }
            }
            let edge_x = SG_W / 2.0 - x.abs();
            let edge_y = SG_H / 2.0 - y.abs();
            d = d.min(edge_x).min(edge_y);
            let cement = 1.0 - sstep(0.0028, 0.0060, d);
            let n = fbm(seed ^ 0xce, x * 260.0, y * 260.0, 2);
            c = mix(c, [0.07, 0.065, 0.06], cement * (0.6 + 0.4 * n));
            let dust = (1.0 - sstep(0.003, 0.012, above_flat)) * (0.35 + 0.4 * n);
            c = mix(c, [0.46, 0.44, 0.40], dust * 0.6);
            // the grime of years settled in each piece's foot and at the
            // panel's foot; a haze over the whole
            let foot = 1.0 - sstep(0.0, 0.10, v);
            let haze = fbm(seed ^ 0x4a2e, u * 5.0, v * 7.0, 3);
            c = mix(c, [0.26, 0.24, 0.20], 0.10 + 0.20 * foot + 0.10 * haze);
            // one field piece cracked across, the crack dark, its two sides
            // a little apart in tone
            if id == cracked {
                let cr =
                    (x * 0.8 + y * 0.6 - (hash(seed, 3, 3) - 0.5) * 0.06).abs() - 0.0006 * (1.0 + vn(seed ^ 0xc4, x * 400.0, y * 400.0));
                if cr < 0.0 {
                    c = c.map(|k| k * 0.35);
                } else if x * 0.8 + y * 0.6 > 0.0 {
                    c = c.map(|k| k * 0.93);
                }
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The frame's pine under a walnut stain (laid across its face: u right, v
/// up): the grain along each member, the members mitred at the corners;
/// the stain worn to the pale wood on the arrises, darker in the grain,
/// dust on the top rail
fn frame_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 180, false, move |ctx, w, h| {
        let seed = salt ^ 0xf4a3;
        fill(ctx, w, h, |u, v| {
            let x = (u - 0.5) * SG_FRAME.0;
            let y = (v - 0.5) * SG_FRAME.1;
            let (dx, dy) = (SG_FRAME.0 / 2.0 - x.abs(), SG_FRAME.1 / 2.0 - y.abs());
            // along the member: a rail's grain runs across, a stile's up
            let (along, across, member) = if dy < dx { (x, y, 1 + i32::from(y > 0.0)) } else { (y, x, 3 + i32::from(x > 0.0)) };
            let fig = fbm(seed ^ member as u32, along * 6.0, across * 90.0, 3);
            let ring = ((across * 260.0 + fig * 9.0).sin() * 0.5 + 0.5).powf(3.0);
            let mut c = mix([0.30, 0.20, 0.12], [0.20, 0.13, 0.08], ring * 0.7 + fig * 0.3);
            // the stain rubbed off the arrises (the frame's outer and sight
            // edges)
            let arris = dx.min(dy);
            let sight = (dx - 0.020).abs().min((dy - 0.020).abs());
            let rub = (1.0 - sstep(0.0, 0.004, arris)).max(1.0 - sstep(0.0, 0.003, sight));
            let rn = vn(seed ^ 7, along * 120.0, across * 120.0);
            c = mix(c, [0.52, 0.40, 0.27], rub * (0.4 + 0.5 * rn));
            // dust on the top rail
            if y > 0.0 && dy < 0.022 {
                c = mix(c, [0.48, 0.45, 0.40], 0.25 * (0.5 + 0.5 * fbm(seed ^ 9, x * 40.0, y * 40.0, 2)));
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = game::assets::props::prop_part_as_modelled(name)?;
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}
