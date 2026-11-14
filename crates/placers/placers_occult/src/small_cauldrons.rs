//! `place_small_cauldrons`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::css;
use placers_kit::paint::fill;
use placers_kit::paint::soft;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::pick_tile;
use placers_kit::place::salt;

// Small cauldron: 6% chance per level. A black plastic cauldron
// (15 cm), the candy-bowl kind: a moulded pot-bellied bowl on three stubby
// feet, a rib round its shoulder, a plastic bail. Half of them hold stale
// water (a drip from above: its plop is the old bubble's sound), tinted by
// what was in it (the old brew's colour: algae, grape, a red punch's rust);
// the rest are dry, dust and two sweet wrappers in the bottom. No glow:
// nothing in it is alight.
pub(crate) fn place_small_cauldrons(g: &mut Game, _dist: &[i32]) {
    g.pp.cauldron_positions = vec![]; // reset per level
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let brew_colors: [u32; 3] = [0x30d040, 0xa020d0, 0xd82040];
    let cauldron = group();
    let is_glowing = rnd() < 0.50;
    // the old brew's draws: its colour, its flicker's phase (now the
    // water's own variety)
    let wet = if is_glowing {
        let brew = pick_hex(&brew_colors);
        let ripple = rnd();
        Some((brew_colors.iter().position(|&c| c == brew).unwrap_or(0), ripple))
    } else {
        None
    };
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 6);
    put(&cauldron, "Real__cauldronPot", &phong!({ map: &pot_tex(salt), specular: 0x2e2e2e, shininess: 28 }));
    put(&cauldron, "Real__cauldronBail", &phong!({ color: 0x2a2928, specular: 0x2e2e2e, shininess: 28 }));
    match wet {
        Some((kind, ripple)) => {
            let salt = salt ^ ((ripple * 65_536.0) as u32);
            put(
                &cauldron,
                "Real__cauldronWater",
                &phong!({ map: &water_tex(kind, salt), specular: 0x5a5a58, shininess: 90, transparent: true, opacity: 0.94, depthWrite: false }),
            );
        }
        None => {
            put(
                &cauldron,
                "Real__cauldronDust",
                &lambert!({ map: &pot_dust_tex(salt), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
            );
            // two sweet wrappers, emptied: a faded orange wax paper, a foil
            for (i, (r, a, mat)) in
                [(0.020, 0.6, lambert!({ color: 0xa8724a })), (0.028, 2.9, phong!({ color: 0x6c5e6a, specular: 0x8a8a8a, shininess: 50 }))]
                    .into_iter()
                    .enumerate()
            {
                if let Some(w) = put(&cauldron, "Real__cauldronWrapper", &mat) {
                    w.position().set(r * f64::cos(a), caul_floor(r + 0.01) - 0.0004, r * f64::sin(a));
                    w.rotation().y = a * 1.7 + i as f64;
                    w.rotation().z = -0.12;
                }
            }
        }
    }
    cauldron.position().set(px, 0, pz);
    cauldron.rotation().y = turn;
    g.world.add(&cauldron);
    // The water's plop (a drip into it), where it stands
    if is_glowing {
        let cp = cauldron.pos();
        g.pp.cauldron_positions.push(v3(cp.x, 0.05, cp.z));
    }
}

fn caul_floor(r: f64) -> f64 {
    // `occult/halloween_things.py` `_caul_floor`: the outside moved in by its 2.4 mm wall
    let pr = [(0.0, 0.0149), (0.0476, 0.0215), (0.0616, 0.0345), (0.0696, 0.052)];
    for w in pr.windows(2) {
        let ((r0, y0), (r1, y1)) = (w[0], w[1]);
        if r <= r1 {
            return y0 + (y1 - y0) * ((r - r0) / (r1 - r0)).clamp(0.0, 1.0);
        }
    }
    0.052
}

/// Moulded black polypropylene (laid round the pot): not quite even, its
/// mould's parting lines down two sides, scuffed pale, dust
fn pot_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xc41;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 12.0, v * 6.0, 4);
            let mut c = mix([0.150, 0.146, 0.142], [0.19, 0.185, 0.18], n);
            for s in [0.25, 0.75] {
                c = mix(c, [0.24, 0.235, 0.23], (1.0 - ((u - s) * 900.0).abs()).max(0.0) * 0.8);
            }
            let dust = vn(seed ^ 3, u * 320.0, v * 160.0);
            if dust > 0.8 {
                c = mix(c, [0.44, 0.42, 0.39], (dust - 0.8) * 2.4);
            }
            // more dust on the shoulder and lip, which face up
            c = mix(c, [0.36, 0.34, 0.31], sstep(0.7, 0.95, v) * 0.25 * fbm(seed ^ 5, u * 30.0, v * 30.0, 2));
            [c[0], c[1], c[2], 1.0]
        });
        // scuffs: short pale scratches where it was knocked about
        ctx.set_line_cap("round");
        for _ in 0..40 {
            let (x, y) = (rnd() * w, h * (0.15 + 0.7 * rnd()));
            let a = (rnd() - 0.5) * 1.2;
            let l = 4.0 + rnd() * 18.0;
            ctx.set_stroke_style(css([0.36, 0.35, 0.34], 0.25 + 0.35 * rnd()));
            ctx.set_line_width(0.6 + rnd() * 1.2);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
}

/// Water stood in it for years (seen from above): tinted by what was in it
/// (0 algae green-brown, 1 grape gone wine-dark, 2 a red punch's rust),
/// darker in the middle, a scum of dust floating, a dead fly
fn water_tex(kind: usize, salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x3a7;
        let tint = [[0.20, 0.22, 0.14], [0.17, 0.13, 0.15], [0.24, 0.15, 0.10]][kind.min(2)];
        fill(ctx, w, h, |u, v| {
            let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
            let mut c = mix([tint[0] * 0.6, tint[1] * 0.6, tint[2] * 0.6], tint, d);
            let scum = fbm(seed, u * 7.0, v * 7.0, 4);
            c = mix(c, [0.36, 0.34, 0.29], sstep(0.55, 0.75, scum) * 0.6);
            let speck = vn(seed ^ 3, u * 200.0, v * 200.0);
            if speck > 0.85 {
                c = mix(c, [0.42, 0.40, 0.35], (speck - 0.85) * 4.0);
            }
            // a ring of scum where it meets the wall
            c = mix(c, [0.32, 0.30, 0.25], sstep(0.86, 0.97, d) * 0.6);
            [c[0], c[1], c[2], 1.0]
        });
        // a fly, drowned
        let (fx, fy) = (w * (0.3 + 0.4 * rnd()), h * (0.3 + 0.4 * rnd()));
        soft(ctx, (fx, fy), 3.0, 1.6, rnd() * 3.0, [0.06, 0.05, 0.05], 0.9);
        soft(ctx, (fx + 3.0, fy - 1.0), 2.6, 0.6, 0.4, [0.30, 0.30, 0.30], 0.5);
    })
}

/// The dust in a dry one's bottom (seen from above, transparent round it):
/// grey-brown, thicker in the middle, crumbs of something
fn pot_dust_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xd57;
        fill(ctx, w, h, |u, v| {
            let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
            let n = fbm(seed, u * 9.0, v * 9.0, 4);
            let a = (1.0 - sstep(0.4, 1.0, d)) * (0.35 + 0.45 * n);
            let c = mix([0.42, 0.39, 0.34], [0.52, 0.49, 0.43], n);
            let crumb = vn(seed ^ 3, u * 90.0, v * 90.0);
            if crumb > 0.9 && d < 0.8 {
                return [0.34, 0.24, 0.16, 0.9];
            }
            [c[0], c[1], c[2], a]
        });
    })
}
