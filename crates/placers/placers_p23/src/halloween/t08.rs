//! Batch `t08`'s Halloween decorations, made real (`docs/design/PROPS.md` P3, "the
//! party and the shrine"): what a party or a shrine leaves years on, as the
//! cheap shop decorations they were (`assets/src/blender/props/t08.py`,
//! `t08.notes.md`): the cheesecloth ghost, the resin cat, the besom, the
//! hanging bat, the plastic skeleton hand, the plastic cauldron and the
//! carved pumpkin gone soft. Their placers and what only they use; each
//! keeps the old placer's draws, call for call.
use crate::{open_tiles, pick_tile, pick_u32};
use game::prelude::*;
use t08_kit::*;

// Small ghost sheet: 5% chance per level. A cheesecloth ghost (25 cm),
// the tabletop kind: cheesecloth stiffened over a ball on a bottle, a string
// at its neck, the form taken out; two black felt eyes glued on. The cloth
// gone yellow-grey with dust, a glue run dried in it, a tear or two.
pub fn place_small_ghost_sheets(g: &mut Game, _dist: &[i32]) {
    g.pp.ghost_positions = vec![]; // reset per level
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let turn = rnd() * PI * 2.0;
    // the old sway (60%: its phase and period) is kept as draws: a
    // stiffened cloth stands still
    if rnd() < 0.60 {
        let _ = rnd();
        let _ = rnd();
    }
    let salt = salt(px, pz, 1);
    let ghost = group();
    put(&ghost, "Real__ghostCloth", &lambert!({ map: &cheesecloth_tex(salt), alphaTest: 0.5, side: DoubleSide }));
    put(&ghost, "Real__ghostEyes", &lambert!({ color: 0x1c1a19 }));
    ghost.position().set(px, 0, pz);
    ghost.rotation().y = turn;
    g.world.add(&ghost);
    // Track for whisper audio.
    let gp = ghost.pos();
    g.pp.ghost_positions.push(v3(gp.x, 0.20, gp.z));
    // the old glow's pulse phase (no light: a cloth does not shine)
    let _ = rnd();
}

/// Cheesecloth stiffened and left (laid on the cloth from its sides and
/// from above alike): yellowed grey-white, its loose weave, a run of the
/// stiffener dried yellow in it, dust, grime toward its hem, a tear or two
/// (cut out: alpha tested)
fn cheesecloth_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 512, false, move |ctx, w, h| {
        let seed = salt ^ 0x6c0;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 6.0, v * 6.0, 4);
            let mut c = mix([0.80, 0.77, 0.69], [0.70, 0.67, 0.58], n);
            // the weave: threads across and along, a little uneven
            let wu = ((u * 210.0 + 0.4 * vn(seed ^ 3, v * 30.0, u * 4.0)) * PI).sin().abs();
            let wv = ((v * 170.0 + 0.4 * vn(seed ^ 5, u * 30.0, v * 4.0)) * PI).sin().abs();
            c = mix(c, [0.60, 0.57, 0.49], (1.0 - wu.min(wv)).powi(6) * 0.25);
            // the stiffener's runs, dried yellow
            let run = sstep(0.62, 0.78, fbm(seed ^ 7, u * 3.0, v * 9.0, 4));
            c = mix(c, [0.72, 0.64, 0.44], run * 0.55);
            // grime toward the hem, dust everywhere
            c = mix(c, [0.48, 0.45, 0.39], (1.0 - sstep(0.0, 0.25, v)) * 0.5);
            let dust = vn(seed ^ 9, u * 300.0, v * 300.0);
            if dust > 0.82 {
                c = mix(c, [0.50, 0.48, 0.44], (dust - 0.82) * 3.0);
            }
            // a tear or two, frayed
            let tear = fbm(seed ^ 11, u * 9.0, v * 9.0, 3) + 0.08 * vn(seed ^ 13, u * 120.0, v * 120.0);
            let a = if tear > 0.79 { 0.0 } else { 1.0 };
            [c[0], c[1], c[2], a]
        });
    })
}

// Small black cat figurine: 6% chance per level. A cast resin black
// cat (15 cm), sitting, its tail round its feet, glazed black, its eyes
// painted yellow-green with a black slit; the glaze dusty, chipped to the
// grey resin at its ears and paws.
pub fn place_small_black_cats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 2);
    let cat = group();
    put(&cat, "Real__catBody", &phong!({ map: &glaze_tex(salt), specular: 0x3a3a3a, shininess: 45 }));
    // the paint catches the light; it never makes its own
    put(&cat, "Real__catEyes", &phong!({ color: 0x9a9238, specular: 0x5a5a5a, shininess: 70 }));
    put(&cat, "Real__catPupils", &phong!({ color: 0x1c1b1a, specular: 0x5a5a5a, shininess: 70 }));
    cat.position().set(px, 0, pz);
    cat.rotation().y = turn;
    g.world.add(&cat);
}

/// Black glaze on cast resin (laid on it from every side): not quite even,
/// dust settled in it, chips through to the grey resin, a few scratches
fn glaze_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xca7;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 8.0, v * 8.0, 4);
            let mut c = mix([0.15, 0.145, 0.14], [0.20, 0.19, 0.18], n);
            let dust = vn(seed ^ 3, u * 160.0, v * 160.0);
            if dust > 0.84 {
                c = mix(c, [0.32, 0.31, 0.29], (dust - 0.84) * 3.0);
            }
            // a few chips through to the resin
            let chip = vn(seed ^ 5, u * 22.0, v * 22.0) + 0.10 * vn(seed ^ 7, u * 140.0, v * 140.0);
            if chip > 0.985 {
                c = mix(c, [0.50, 0.48, 0.44], sstep(0.985, 1.01, chip));
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

// Small broom: 8% chance per level. A decorative witch's besom
// (90 cm): a crooked hazel stick, its bark on, birch twigs bound round its
// end with two wraps of jute twine, lying on the floor; the twine dusty,
// the twigs dry and brittle, some broken.
pub fn place_small_brooms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    // the old fourteen twigs' draws: each its length and its lift; their
    // lengths' mean sizes the besom a little (+-4%)
    let mut len = 0.0;
    for _ in 0..14 {
        len += 0.11 + rnd() * 0.03;
        let _ = rnd();
    }
    let k = (len / 14.0) / 0.125;
    let px = g.wx(x) + (rnd() - 0.5) * 1.6;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.6;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 3);
    let broom = group();
    let besom = group();
    // modelled from the handle's end along +x, as the old one lay
    besom.scale().set_scalar(k);
    broom.add(&besom);
    put(&besom, "Real__besomHandle", &lambert!({ map: &bark_tex(salt) }));
    put(&besom, "Real__besomTwigs", &lambert!({ color: 0x55402f }));
    put(&besom, "Real__besomTwine", &lambert!({ color: 0x8a7754 }));
    broom.position().set(px, 0, pz);
    broom.rotation().y = turn;
    g.world.add(&broom);
}

/// A hazel stick's bark (laid along it: u along the stick): grey-brown,
/// streaked along its length, its lenticels pale dashes, darker where
/// hands held it
fn bark_tex(salt: u32) -> Tex {
    canvas_tex_opt(1024, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xba7;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 40.0, v * 3.0, 4);
            let mut c = mix([0.40, 0.34, 0.27], [0.52, 0.45, 0.36], n);
            let streak = fbm(seed ^ 3, u * 6.0, v * 30.0, 3);
            c = mix(c, [0.30, 0.25, 0.20], sstep(0.55, 0.75, streak) * 0.5);
            let l = vn(seed ^ 5, u * 900.0, v * 26.0);
            if l > 0.86 {
                c = mix(c, [0.66, 0.60, 0.50], sstep(0.86, 0.92, l) * 0.8);
            }
            // where it was held, a third of the way along
            c = mix(c, [0.28, 0.24, 0.20], (1.0 - ((u - 0.25) / 0.12).abs()).max(0.0) * 0.35);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

// Hanging bat: 5% chance per level. A hanging Halloween bat (48 cm
// across its wings): a flocked foam body on a plastic skeleton, its wings
// plastic over mesh between moulded ribs, spread as in flight, hung level by
// a cord taped to the ceiling; it turns and sways a little on its cord.
pub fn place_hanging_bats(g: &mut Game, _dist: &[i32]) {
    g.pp.bat_positions = vec![]; // reset per level
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let bat_y = WALL_H - 0.15;
    let bat = group();
    bat.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    bat.rotation().y = rnd() * PI * 2.0;
    // modelled hanging from the ceiling (its cord's top at the origin): it
    // swings about where the cord is taped
    let hang = group();
    hang.position().y = WALL_H;
    bat.add(&hang);
    put(&hang, "Real__batWings", &phong!({ color: 0x232120, specular: 0x262626, shininess: 18, side: DoubleSide }));
    put(&hang, "Real__batBody", &lambert!({ color: 0x302b28 }));
    put(&hang, "Real__batEyes", &phong!({ color: 0x5c1e18, specular: 0x6a6a6a, shininess: 70 }));
    put(&hang, "Real__batCord", &lambert!({ color: 0x1e1d1c }));
    put(&hang, "Real__batTape", &lambert!({ color: 0xb4a684, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }));
    // Slow sway on its cord (the old draws: 60%, its phase and period).
    if rnd() < 0.60 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 8.0 + rnd() * 6.0;
        let hang2 = hang.clone();
        g.animate(move |_g, _dt, t| {
            let a = t * (PI * 2.0 / sway_period) + sway_phase;
            hang2.rotation().z = a.sin() * 0.025;
            hang2.rotation().x = (a * 0.71 + 1.3).sin() * 0.015;
            hang2.rotation().y = (a * 0.37).sin() * 0.35;
        });
    }
    g.world.add(&bat);
    // Track for rare squeak audio.
    let bp = bat.pos();
    g.pp.bat_positions.push(v3(bp.x, bat_y, bp.z));
}

// Small skeleton hand: 5% chance per level. A life-size moulded plastic
// skeleton hand (18 cm), the shop kind, lying palm down, its fingers a
// little curled; the plastic gone ivory, grimed in its joints, dusty.
pub fn place_small_skeleton_hands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 5);
    let hand = group();
    put(&hand, "Real__skelHand", &phong!({ map: &bone_plastic_tex(salt), specular: 0x262422, shininess: 14 }));
    hand.position().set(px, 0, pz);
    hand.rotation().y = turn;
    g.world.add(&hand);
}

/// Moulded polystyrene gone ivory (laid on it from above and its sides):
/// yellowed unevenly, grime in patches, dust
fn bone_plastic_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xb0e;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 7.0, v * 7.0, 4);
            let mut c = mix([0.80, 0.76, 0.65], [0.72, 0.66, 0.52], n);
            let grime = sstep(0.55, 0.8, fbm(seed ^ 3, u * 14.0, v * 14.0, 3));
            c = mix(c, [0.46, 0.41, 0.32], grime * 0.55);
            let dust = vn(seed ^ 5, u * 180.0, v * 180.0);
            if dust > 0.8 {
                c = mix(c, [0.55, 0.52, 0.47], (dust - 0.8) * 2.5);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The cauldron (batch `t08`): the height its water stands at, and its
/// inside bottom's height at a radius
const CAUL_WATER: f64 = 0.0620;
fn caul_floor(r: f64) -> f64 {
    // `t08.py` `_caul_floor`: the outside moved in by its 2.4 mm wall
    let pr = [(0.0, 0.0149), (0.0476, 0.0215), (0.0616, 0.0345), (0.0696, 0.052)];
    for w in pr.windows(2) {
        let ((r0, y0), (r1, y1)) = (w[0], w[1]);
        if r <= r1 {
            return y0 + (y1 - y0) * ((r - r0) / (r1 - r0)).clamp(0.0, 1.0);
        }
    }
    0.052
}

// Small cauldron: 6% chance per level. A black plastic cauldron
// (15 cm), the candy-bowl kind: a moulded pot-bellied bowl on three stubby
// feet, a rib round its shoulder, a plastic bail. Half of them hold stale
// water (a drip from above: its plop is the old bubble's sound), tinted by
// what was in it (the old brew's colour: algae, grape, a red punch's rust);
// the rest are dry, dust and two sweet wrappers in the bottom. No glow:
// nothing in it is alight.
pub fn place_small_cauldrons(g: &mut Game, _dist: &[i32]) {
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
        let brew = pick_u32(&brew_colors);
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
            ctx.set_stroke_style(&css([0.36, 0.35, 0.34], 0.25 + 0.35 * rnd()));
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

/// The tea light in the pumpkin (batch `t08`): its wick's tip
const PK_WICK: (f64, f64, f64) = (0.0032, 0.0095 + 0.0105, 0.004);

// Small pumpkin: 5% chance per level. A small carving pumpkin (19 cm)
// carved into a jack-o'-lantern and left: gone soft and slumped, its top
// sunk, the lid fallen in askew, its skin dull, rotting brown in patches
// and furred with mould, the cut flesh dried brown. Six in ten still have a
// tea light burning in them (a flame: its light through the face); the
// rest, a tea light burnt out.
pub fn place_small_pumpkins(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let is_lit = rnd() < 0.60;
    let pumpkin = group();
    if is_lit {
        let glow = point_light!(0xff9a40, 0.4, 0.8, 2);
        glow.position().set(PK_WICK.0, PK_WICK.1 + 0.02, PK_WICK.2);
        pumpkin.add(&glow);
        // the flame over the wick, turned to the eye
        let geo = plane_geo!(0.010, 0.026);
        geo.translate(0, 0.012, 0);
        let flame = mesh(geo, basic_mat!({ map: flame_tex(), transparent: true, opacity: 0.9, depthWrite: false }));
        flame.position().set(PK_WICK.0, PK_WICK.1, PK_WICK.2);
        pumpkin.add(&flame);
        // its flicker (the old draws: phase and rate)
        let flicker_phase = rnd() * PI * 2.0;
        let flicker_rate = 4.0 + rnd() * 3.0;
        g.animate(move |g, _dt, t| {
            let wobble = (t * flicker_rate + flicker_phase).sin() * 0.5 + (t * flicker_rate * 1.7 + flicker_phase).sin() * 0.3;
            glow.set_intensity(0.4 + wobble * 0.08);
            flame.scale().y = 1.0 + wobble * 0.18;
            let m = flame.matrix_world();
            let c = g.camera.pos();
            flame.look_at(c.x, m.e[13], c.z);
        });
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 7);
    put(&pumpkin, "Real__pumpkinSkin", &phong!({ map: &pumpkin_skin_tex(salt), specular: 0x1c1a16, shininess: 12 }));
    put(&pumpkin, "Real__pumpkinFlesh", &lambert!({ map: &pumpkin_flesh_tex(salt) }));
    put(&pumpkin, "Real__pumpkinStem", &lambert!({ color: 0x6c6250 }));
    put(&pumpkin, "Real__pumpkinTealight", &phong!({ color: 0x8c8c8a, specular: 0x9a9a9a, shininess: 60 }));
    put(&pumpkin, "Real__pumpkinWax", &lambert!({ color: if is_lit { 0xd6cdb8 } else { 0xb8ae98 } }));
    pumpkin.position().set(px, 0, pz);
    pumpkin.rotation().y = turn;
    g.world.add(&pumpkin);
}

/// A carved pumpkin's skin gone soft (laid round it: u round, v from its
/// bottom up): a dull orange gone brown, darker down its ten grooves, soft
/// brown rot in patches (more on its sunken top and where it sits), white
/// and blue-green mould in colonies, wrinkled
fn pumpkin_skin_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x9b1;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 10.0, v * 5.0, 4);
            let mut c = mix([0.70, 0.42, 0.18], [0.60, 0.33, 0.14], n);
            // the grooves between the lobes
            let gr = ((u - 0.05) * 10.0 + 0.04 * vn(seed ^ 3, v * 8.0, 0.0)).rem_euclid(1.0);
            let groove = (1.0 - ((gr - 0.5).abs() * 2.0)).powi(3);
            c = mix(c, [0.42, 0.25, 0.12], (1.0 - groove) * 0.0 + (1.0 - ((gr.min(1.0 - gr)) / 0.08).min(1.0)) * 0.45);
            // the rot: soft brown patches, more on the top and the bottom
            let rot = fbm(seed ^ 5, u * 6.0, v * 4.0, 5) + 0.25 * sstep(0.7, 1.0, v) + 0.2 * (1.0 - sstep(0.0, 0.25, v));
            c = mix(c, [0.40, 0.25, 0.13], sstep(0.56, 0.72, rot) * 0.85);
            c = mix(c, [0.22, 0.15, 0.09], sstep(0.76, 0.9, rot) * 0.9);
            // wrinkles
            let wr = vn(seed ^ 7, u * 60.0, v * 140.0);
            c = mix(c, [0.30, 0.19, 0.10], sstep(0.75, 0.9, wr) * 0.35);
            // mould: white fur and blue-green colonies, on the rot
            let m = fbm(seed ^ 9, u * 40.0, v * 20.0, 3) + 0.25 * sstep(0.6, 0.8, rot);
            let fur = vn(seed ^ 13, u * 400.0, v * 200.0);
            c = mix(c, [0.70, 0.69, 0.63], sstep(0.80, 0.95, m) * (0.35 + 0.35 * fur));
            let bg = fbm(seed ^ 11, u * 50.0, v * 25.0, 3) + 0.25 * sstep(0.65, 0.85, rot);
            c = mix(c, [0.40, 0.47, 0.40], sstep(0.86, 0.97, bg) * 0.55);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The cut flesh and the inside (laid round it): dried to a tan leather,
/// browned, the fibres' strings, mould on the cut faces
fn pumpkin_flesh_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xf1e;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 9.0, v * 9.0, 4);
            let mut c = mix([0.42, 0.29, 0.15], [0.28, 0.18, 0.09], n);
            let str_ = vn(seed ^ 3, u * 20.0, v * 120.0);
            c = mix(c, [0.20, 0.13, 0.07], sstep(0.7, 0.85, str_) * 0.5);
            let m = fbm(seed ^ 5, u * 30.0, v * 30.0, 3);
            c = mix(c, [0.62, 0.61, 0.56], sstep(0.76, 0.9, m) * 0.5);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A tea light's flame as it is seen, on a plane whose foot is the flame's
/// (as the haunted floors' candles draw theirs): a teardrop, yellow-white
/// through its middle, orange at its edge, blue at its foot, a faint glow
fn flame_tex() -> Tex {
    canvas_tex_opt(32, 64, false, move |ctx, w, h| {
        fill(ctx, w, h, |uu, v| {
            let u = uu * 2.0 - 1.0;
            let (v0, v1, half) = (0.07, 0.84, 0.46);
            let t = (v - v0) / (v1 - v0);
            let hw = if t <= 0.0 || t >= 1.0 {
                0.0
            } else if t < 0.3 {
                half * (t / 0.3).sqrt()
            } else {
                half * ((1.0 - t) / 0.7).powf(1.2)
            };
            let q = if hw > 1e-6 { u.abs() / hw } else { 9.0 };
            let body = 1.0 - sstep(0.55, 1.05, q);
            let edge = sstep(0.3, 1.0, q).max(sstep(0.55, 1.0, t));
            let (r, g, b) = (1.0, 0.9 - 0.32 * edge, 0.7 - 0.52 * edge);
            let foot = (1.0 - sstep(0.04, 0.24, t)) * (0.35 + 0.65 * sstep(0.2, 0.8, q));
            let dark = (1.0 - sstep(0.0, 0.3, t)) * (1.0 - sstep(0.0, 0.5, q)) * 0.6;
            let (r, g, b) = (r + (0.35 - r) * foot, g + (0.5 - g) * foot, b + (1.0 - b) * foot);
            let a = body * (1.0 - dark) * (1.0 - 0.45 * foot);
            let d = (q - 1.0).max(0.0) * hw.max(0.2) + (v0 - v).max(0.0) * 1.5 + (v - v1).max(0.0) * 1.2;
            let halo = 0.14 * (-(d / 0.22).powi(2)).exp();
            [r, g, b, a + halo * (1.0 - a)]
        });
    })
}

/// The painting kit of batch `t08`'s painters (as batch `t07`'s): value
/// noise and a per-pixel fill (pure, so `par_rows` may run it on every
/// core), soft spots, a salt from where a thing is, a part placed.
#[allow(dead_code)]
mod t08_kit {
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
    pub(super) fn css(c: [f64; 3], a: f64) -> String {
        let k = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("rgba({},{},{},{:.3})", k(c[0]), k(c[1]), k(c[2]), a.clamp(0.0, 1.0))
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
    /// A soft spot at `at`, `r` px, `k` times as tall as wide, turned `turn`,
    /// fading out from its middle (the context squashed, so no edge cuts it)
    pub(super) fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: [f64; 3], a: f64) {
        ctx.save();
        ctx.translate(at.0, at.1);
        ctx.rotate(turn);
        ctx.scale(1, k);
        let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
        gr.add_color_stop(0, &css(c, a));
        gr.add_color_stop(1, &css(c, 0.0));
        ctx.set_fill_style(&gr);
        ctx.begin_path();
        ctx.arc(0, 0, r, 0, 7);
        ctx.fill();
        ctx.restore();
    }
    /// A salt from where a thing is (no draw): its painters' strokes differ
    /// from floor to floor
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
