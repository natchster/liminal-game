//! Prop placers: Victorian-house props — fire tool sets, botanical prints, floor
//! kettles, specimen domes, secretary desks, tarnished mirrors, sheet-covered
//! furniture, servant bell pulls and hall stands.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod t02;
use t02::place_botanical_prints;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_fire_tool_sets", place_fire_tool_sets));
    r.push(("place_botanical_prints", place_botanical_prints));
    r.push(("place_floor_kettles", place_floor_kettles));
    r.push(("place_specimen_domes", place_specimen_domes));
    r.push(("place_secretary_desks", place_secretary_desks));
    r.push(("place_tarnished_mirrors", place_tarnished_mirrors));
    r.push(("place_sheet_covered_furniture", place_sheet_covered_furniture));
    r.push(("place_servant_bell_pulls", place_servant_bell_pulls));
    r.push(("place_victorian_hall_stands", place_victorian_hall_stands));
}

/// A uniformly random element (one `rnd()` draw). index
#[inline]
fn ri(len: usize) -> usize {
    (rnd() * len as f64).floor() as usize
}

/// Every open (tile 0) interior tile, row-major loops.
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                v.push((x, z));
            }
        }
    }
    v
}

/// Open tiles next to a wall (or the grid edge), one entry per facing wall:
/// `(x, z, dx, dz)` for each side where `!in_grid(nx, nz) || tiles[..] == 1`.
fn wall_candidates(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut candidates = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    candidates.push((x, z, dx, dz));
                }
            }
        }
    }
    candidates
}

// R752 wrought-iron fire tool set on stand (floor): 3% chance per level.
fn place_fire_tool_sets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // Wrought-iron matte black.
    let iron_mat = lambert!({ color: 0x181818 });
    // Optional brass-tipped highlight material (30% variant).
    let use_brass_tips = rnd() < 0.3;
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let tip_mat = if use_brass_tips { brass_mat.clone() } else { iron_mat.clone() };
    let fts = group();
    // Stand base — heavy round disc.
    let base_r = 0.075;
    let base = mesh(pg("FireToolSets__base", cyl_geo!(base_r, base_r + 0.008, 0.012, 12)), &iron_mat);
    base.position().y = 0.006;
    fts.add(&base);
    // Vertical stand rod (60 cm tall).
    let rod_h = 0.60;
    let rod = mesh(pg("FireToolSets__rod", cyl_geo!(0.006, 0.006, rod_h, 6)), &iron_mat);
    rod.position().y = 0.012 + rod_h / 2.0;
    fts.add(&rod);
    // Top ornamental finial.
    let finial = mesh(pg("FireToolSets__finial", sphere_geo!(0.012, 8, 6)), &tip_mat);
    finial.position().y = 0.012 + rod_h + 0.010;
    fts.add(&finial);
    // Cross-bar arms for hanging tools.
    let arm_y = 0.012 + rod_h - 0.030;
    let num_tools = 3 + (rnd() * 2.0).floor() as i32; // 3 or 4 tools
    for i in 0..num_tools {
        let arm_angle = (i as f64 / num_tools as f64) * PI * 2.0;
        let arm = mesh(pg("FireToolSets__arm", cyl_geo!(0.0025, 0.0025, 0.030, 4)), &iron_mat);
        arm.rotation().z = PI / 2.0;
        arm.position().set(arm_angle.cos() * 0.015, arm_y, arm_angle.sin() * 0.015);
        arm.rotation().y = -arm_angle;
        fts.add(&arm);
        // Small hook cap on arm end.
        let hook = mesh(pg("FireToolSets__hook", torus_geo!(0.004, 0.001, 4, 6, PI)), &iron_mat);
        hook.rotation().x = PI / 2.0;
        hook.rotation().z = -arm_angle;
        hook.position().set(arm_angle.cos() * 0.030, arm_y - 0.002, arm_angle.sin() * 0.030);
        fts.add(&hook);
        // Tool hanging from hook.
        let tool_cx = arm_angle.cos() * 0.030;
        let tool_cz = arm_angle.sin() * 0.030;
        let tool_types = ["poker", "tongs", "shovel", "brush"];
        let tool_type = tool_types[(i % 4) as usize];
        if tool_type == "poker" {
            let shaft_len = 0.36;
            let shaft = mesh(pg("FireToolSets__shaft", cyl_geo!(0.003, 0.003, shaft_len, 5)), &iron_mat);
            shaft.position().set(tool_cx, arm_y - shaft_len / 2.0 - 0.005, tool_cz);
            fts.add(&shaft);
            let hook_tip = mesh(pg("FireToolSets__pokerTip", cyl_geo!(0.001, 0.005, 0.020, 4)), &iron_mat);
            hook_tip.position().set(tool_cx + 0.006, arm_y - shaft_len - 0.005, tool_cz);
            hook_tip.rotation().z = 0.5;
            fts.add(&hook_tip);
            let grip = mesh(pg("FireToolSets__grip", cyl_geo!(0.006, 0.006, 0.030, 6)), &tip_mat);
            grip.position().set(tool_cx, arm_y + 0.010, tool_cz);
            fts.add(&grip);
        } else if tool_type == "tongs" {
            let shaft_len = 0.32;
            let shaft = mesh(pg("FireToolSets__shaft", cyl_geo!(0.003, 0.003, shaft_len, 5)), &iron_mat);
            shaft.position().set(tool_cx, arm_y - shaft_len / 2.0 - 0.005, tool_cz);
            fts.add(&shaft);
            // 2 pincer arms V-splayed at bottom.
            for pside in [-1.0f64, 1.0] {
                let pinc = mesh(pg("FireToolSets__pincer", cyl_geo!(0.0025, 0.0025, 0.055, 4)), &iron_mat);
                pinc.position().set(tool_cx + pside * 0.008, arm_y - shaft_len - 0.020, tool_cz);
                pinc.rotation().z = pside * 0.3;
                fts.add(&pinc);
            }
            // Loop grip at top: upright ring across the arm end.
            let grip_loop = mesh(pg("FireToolSets__loop", torus_geo!(0.010, 0.002, 4, 10)), &tip_mat);
            grip_loop.position().set(tool_cx, arm_y + 0.005, tool_cz);
            grip_loop.rotation().y = -arm_angle - PI / 2.0;
            fts.add(&grip_loop);
        } else if tool_type == "shovel" {
            let shaft_len = 0.30;
            let shaft = mesh(pg("FireToolSets__shaft", cyl_geo!(0.003, 0.003, shaft_len, 5)), &iron_mat);
            shaft.position().set(tool_cx, arm_y - shaft_len / 2.0 - 0.005, tool_cz);
            fts.add(&shaft);
            // Small square scoop pan, hanging upright below the shaft.
            let scoop = mesh(pg("FireToolSets__scoop", box_geo!(0.035, 0.005, 0.030)), &iron_mat);
            {
                let mut r = scoop.rotation();
                r.set(PI / 2.0, PI / 2.0 - arm_angle, 0);
                r.order = EulerOrder::YXZ;
            }
            scoop.position().set(arm_angle.cos() * 0.022, arm_y - shaft_len - 0.0195, arm_angle.sin() * 0.022);
            fts.add(&scoop);
            // Scoop back wall (raised).
            let scoop_back = mesh(pg("FireToolSets__scoopBack", box_geo!(0.035, 0.020, 0.003)), &iron_mat);
            {
                let mut r = scoop_back.rotation();
                r.set(PI / 2.0, PI / 2.0 - arm_angle, 0);
                r.order = EulerOrder::YXZ;
            }
            scoop_back.position().set(tool_cx, arm_y - shaft_len - 0.0055, tool_cz);
            fts.add(&scoop_back);
            let grip = mesh(pg("FireToolSets__grip", cyl_geo!(0.006, 0.006, 0.030, 6)), &tip_mat);
            grip.position().set(tool_cx, arm_y + 0.010, tool_cz);
            fts.add(&grip);
        } else {
            // Brush (4th tool): shaft + horizontal bristle head.
            let shaft_len = 0.28;
            let shaft = mesh(pg("FireToolSets__shaft", cyl_geo!(0.003, 0.003, shaft_len, 5)), &iron_mat);
            shaft.position().set(tool_cx, arm_y - shaft_len / 2.0 - 0.005, tool_cz);
            fts.add(&shaft);
            let bristle_mat = lambert!({ color: 0x201810 });
            let bristles = mesh(pg("FireToolSets__bristles", cyl_geo!(0.014, 0.012, 0.028, 8)), &bristle_mat);
            bristles.rotation().z = PI / 2.0;
            bristles.position().set(tool_cx, arm_y - shaft_len - 0.010, tool_cz);
            fts.add(&bristles);
            let grip = mesh(pg("FireToolSets__grip", cyl_geo!(0.006, 0.006, 0.030, 6)), &tip_mat);
            grip.position().set(tool_cx, arm_y + 0.010, tool_cz);
            fts.add(&grip);
        }
    }
    // 30% chance: small ash pile beside the stand.
    if rnd() < 0.3 {
        let ash_mat = lambert!({ color: 0x606060 });
        let ash = mesh(circle_geo!(0.030, 8), &ash_mat);
        ash.rotation().x = -PI / 2.0;
        ash.position().set(base_r + 0.030, 0.001, 0);
        ash.rotation().z = rnd() * PI * 2.0;
        fts.add(&ash);
        // Tiny lumps in the ash.
        for j in 0..3 {
            let lump = mesh(pg("FireToolSets__lump", sphere_geo!(0.004, 5, 4)), lambert!({ color: 0x1a1a1a }));
            let lx = base_r + 0.030 + (rnd() - 0.5) * 0.030;
            let lz = (rnd() - 0.5) * 0.030;
            lump.position().set(lx, 0.0016, lz); // resting in the ash
            lump.scale().y = 0.4;
            fts.add(&lump);
        }
    }
    fts.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    fts.rotation().y = rnd() * PI * 2.0;
    g.world.add(&fts);
}

// R753 a kettle left on the floor (Blender batch `r16`): a 2 l enamel
// stovetop kettle, its bail handle up, its enamel chipped to the black steel
// and rusting at the chips, dented, limescale run white down from its lid
// and spout where it boiled over, scorched brown at its foot; or the same
// kettle in stainless steel, dull and spotted. Still breathing a little
// vapour from its spout now and then. 4% chance per level.
fn place_floor_kettles(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // the old three colours: cream enamel, oxblood enamel, stainless steel
    let finish = ri(3).min(2);
    // (the old brass knob, 40%): a turned wooden knob and grip, else phenolic
    let wooden = rnd() < 0.4;
    // the old spout's bearing: which way it points
    let spout_angle = rnd() * PI * 2.0;
    let kt = group();
    let body = group();
    body.rotation().y = -spout_angle;
    kt.add(&body);
    let salt = (spout_angle * 1.0e6) as u32;
    let shell = if finish == 2 {
        phong!({ map: &kettle_steel_tex(salt), color: 0xffffff, specular: 0x6a6a6a, shininess: 46 })
    } else {
        phong!({ map: &kettle_enamel_tex(finish, salt), color: 0xffffff, specular: 0x262626, shininess: 30 })
    };
    let spout_mat = if finish == 2 {
        phong!({ map: &kettle_spout_tex(2, salt), color: 0xffffff, specular: 0x6a6a6a, shininess: 46 })
    } else {
        phong!({ map: &kettle_spout_tex(finish, salt), color: 0xffffff, specular: 0x262626, shininess: 30 })
    };
    let grip_mat = if wooden { lambert!({ color: 0x5c4028 }) } else { phong!({ color: 0x161412, specular: 0x262626, shininess: 24 }) };
    let wire_mat = phong!({ color: 0x4e4c48, specular: 0x3a3a3a, shininess: 30 });
    for (name, mat) in
        [("Real__kettleBody", &shell), ("Real__kettleSpout", &spout_mat), ("Real__kettleBail", &wire_mat), ("Real__kettleGrip", &grip_mat)]
    {
        if let Some(geo) = prop_part_as_modelled(name) {
            body.add(&mesh(geo, mat));
        }
    }
    // (60%) vapour from the spout, breathing slowly: soft and lit, three
    // wisps (the old draws: where, its phase; the cycle's length and phase)
    if rnd() < 0.6 {
        let steam_mat =
            lambert!({ map: &kettle_vapour_tex(), color: 0xe6e4de, transparent: true, opacity: 0.18, depthWrite: false, side: DoubleSide });
        struct Wisp {
            mesh: Obj,
            phase: f64,
            base_y: f64,
            base_x: f64,
            base_z: f64,
        }
        let mut steam_wisps: Vec<Wisp> = vec![];
        let (tip_x, tip_y) = KETTLE_SPOUT_TIP;
        for i in 0..3 {
            let wisp = group();
            for turn in [PI / 4.0, -PI / 4.0] {
                let sheet = mesh(plane_geo!(0.035, 0.07), &steam_mat);
                sheet.rotation().y = turn;
                sheet.position().y = 0.03;
                wisp.add(&sheet);
            }
            let px = tip_x + 0.006 + (rnd() - 0.5) * 0.015;
            let py = tip_y + i as f64 * 0.030;
            let pz = (rnd() - 0.5) * 0.015;
            wisp.position().set(px, py, pz);
            let phase = rnd() * PI * 2.0;
            steam_wisps.push(Wisp { mesh: wisp.clone(), phase, base_y: py, base_x: px, base_z: pz });
            body.add(&wisp);
        }
        // R755 pressure-breathing cycle: 20-30 s slow oscillation.
        let pressure_cycle = 20.0 + rnd() * 10.0;
        let pressure_phase = rnd() * PI * 2.0;
        let sm = steam_mat.clone();
        g.animate(move |_g, _dt, t| {
            let pressure = 0.95 + ((t * (PI * 2.0) / pressure_cycle) + pressure_phase).sin() * 0.40;
            for w in &steam_wisps {
                {
                    let mut p = w.mesh.position();
                    p.y = w.base_y + (t * 0.7 + w.phase).sin() * 0.015 * pressure;
                    p.x = w.base_x + (t * 0.5 + w.phase).sin() * 0.010;
                    p.z = w.base_z + (t * 0.6 + w.phase * 1.3).cos() * 0.010;
                }
                let s = (0.88 + (t * 0.9 + w.phase).sin() * 0.16) * pressure;
                w.mesh.scale().set(s, s * 1.2, s);
            }
            // the wisps share one material
            sm.set_opacity(0.06 + 0.12 * clamp(pressure * 0.7, 0.0, 1.0));
        });
    }
    // 30% chance: a round cork trivet under it (the old mat's colours: a
    // cork board's browns)
    if rnd() < 0.3 {
        let cork = ri(3).min(2);
        if let Some(geo) = prop_part_as_modelled("Real__kettleTrivet") {
            kt.add(&mesh(geo, lambert!({ map: &kettle_cork_tex(cork) })));
        }
        body.position().y = KETTLE_TRIVET_T;
    }
    kt.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    kt.rotation().y = rnd() * PI * 2.0;
    g.world.add(&kt);
}

/// The spout's mouth in the kettle's frame (x, y; r16.py `KETTLE_SPOUT`)
const KETTLE_SPOUT_TIP: (f64, f64) = (0.1285, 0.1105);
/// The cork trivet's thickness
const KETTLE_TRIVET_T: f64 = 0.010;
/// The kettle body part's height (its bounds' top: the lid, r16.py)
const KETTLE_H: f64 = 0.1367;

fn kettle_hash(seed: u32, x: i32, y: i32) -> f64 {
    let mut h = seed ^ (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h = (h ^ (h >> 13)).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    h as f64 / 4_294_967_296.0
}
/// Smooth value noise in [0, 1], wrapping every `wrap` cells in x
fn kettle_noise(seed: u32, x: f64, y: f64, wrap: i32) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (fx, fy) = (x - xf, y - yf);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xf as i32, yf as i32);
    let w = |v: i32| if wrap > 0 { v.rem_euclid(wrap) } else { v };
    let (a, b) = (kettle_hash(seed, w(xi), yi), kettle_hash(seed, w(xi + 1), yi));
    let (c, d) = (kettle_hash(seed, w(xi), yi + 1), kettle_hash(seed, w(xi + 1), yi + 1));
    a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}

/// A chip in enamel at (x, y) px, `r` across: the black ground coat at its
/// edge, the steel in its middle gone to rust, a sharp light rim where the
/// glass broke away (drawn on the canvas, after the pixels)
fn kettle_chip(ctx: &mut Canvas, r: &mut SplitMix64, x: f64, y: f64, rad: f64, deep: bool) {
    let n = 7;
    let pts: Vec<(f64, f64)> = (0..n)
        .map(|k| {
            let a = k as f64 / n as f64 * PI * 2.0 + r.range(-0.3, 0.3);
            let rr = rad * r.range(0.55, 1.15);
            (x + a.cos() * rr, y + a.sin() * rr * 0.8)
        })
        .collect();
    let path = |ctx: &mut Canvas, k: f64| {
        ctx.begin_path();
        for (i, &(px, py)) in pts.iter().enumerate() {
            let (qx, qy) = (x + (px - x) * k, y + (py - y) * k);
            if i == 0 {
                ctx.move_to(qx, qy);
            } else {
                ctx.line_to(qx, qy);
            }
        }
        ctx.close_path();
    };
    ctx.set_fill_style("rgba(206,204,196,0.55)");
    path(ctx, 1.12);
    ctx.fill();
    ctx.set_fill_style("rgb(26,24,24)");
    path(ctx, 1.0);
    ctx.fill();
    if deep {
        ctx.set_fill_style("rgba(98,52,30,0.9)");
        path(ctx, 0.62);
        ctx.fill();
        ctx.set_fill_style("rgba(70,36,22,0.8)");
        path(ctx, 0.3);
        ctx.fill();
    }
}

/// The enamel, unrolled round the body (u from +z toward +x: the spout at
/// u = 0.25) and up it (v, over the body and lid's 137 mm): its glaze
/// mottled and dulled, grime and a brown scorch at its foot, limescale run
/// white down from the lid and under the spout, the rim's dark band (cream
/// only), chips at the rim, the chime, round the dent and the spout's seam.
fn kettle_enamel_tex(finish: usize, salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let (base, mottle): ([f64; 3], f64) = if finish == 0 { ([214.0, 206.0, 186.0], 10.0) } else { ([104.0, 44.0, 36.0], 8.0) };
        let seed = salt ^ 0xe7a3;
        let vy = |y: f64| (1.0 - y / KETTLE_H) * h;
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let fy = 1.0 - j as f64 / h; // up the kettle
            for i in 0..n {
                let u = i as f64 / w;
                let m = kettle_noise(seed, u * 16.0, fy * 8.0, 16) * 0.6 + kettle_noise(seed ^ 1, u * 64.0, fy * 32.0, 64) * 0.4;
                let mut c = [base[0] + (m - 0.5) * mottle * 2.0, base[1] + (m - 0.5) * mottle * 2.0, base[2] + (m - 0.5) * mottle * 2.0];
                // grime and the stove's scorch, brown, up from the foot
                let low = (1.0 - fy / 0.28).clamp(0.0, 1.0);
                let scorch = (low * low * (0.55 + 0.45 * kettle_noise(seed ^ 2, u * 24.0, fy * 10.0, 24))).min(1.0);
                let sc = [92.0, 64.0, 38.0];
                for k in 0..3 {
                    c[k] += (sc[k] - c[k]) * scorch * 0.75;
                }
                // limescale: chalky runs down from the lid's seat, most under
                // the spout's side, and a crust at the chime
                let du = ((u - 0.25 + 0.5).rem_euclid(1.0) - 0.5).abs();
                let runs = (kettle_noise(seed ^ 3, u * 90.0, fy * 2.0, 90) - 0.55).max(0.0) * 2.6;
                let reach = 0.85 - 0.45 * (1.0 - (du / 0.18).min(1.0));
                let lime = runs * ((fy - reach) / 0.12).clamp(0.0, 1.0) * (0.4 + 0.6 * (1.0 - (du / 0.3).min(1.0)))
                    + 0.5 * (1.0 - (fy / 0.05).min(1.0)) * kettle_noise(seed ^ 4, u * 60.0, fy * 20.0, 60);
                let lime = lime.min(0.8);
                let lc = [222.0, 218.0, 204.0];
                for k in 0..3 {
                    c[k] += (lc[k] - c[k]) * lime;
                }
                // the dust of years, mottled, and the glaze gone dull under it
                let dust = 0.14 + 0.22 * kettle_noise(seed ^ 6, u * 30.0, fy * 14.0, 30) + 0.12 * fy;
                let dc = [150.0, 142.0, 128.0];
                for k in 0..3 {
                    c[k] += (dc[k] - c[k]) * dust;
                }
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // cream enamelware's dark band at the lid seat's bead and the lid's rim
        if finish == 0 {
            ctx.set_fill_style("rgba(34,40,58,0.9)");
            ctx.fill_rect(0, vy(0.1250), w, vy(0.1185) - vy(0.1250));
            ctx.fill_rect(0, vy(0.1270), w, vy(0.1250) - vy(0.1270) + 1.0);
        }
        let mut r = SplitMix64::new(salt, 0xc41b);
        // chips: along the rim's bead, the chime, round the dent (u 0.70),
        // the spout's seam (u 0.25, low)
        let spots: [(f64, f64, f64, usize); 4] =
            [(0.5, 0.120, 0.5, 14), (0.5, 0.004, 0.5, 10), (0.70, 0.052, 0.05, 8), (0.25, 0.032, 0.03, 4)];
        for &(uc, yc, spread, count) in &spots {
            for _ in 0..count {
                let u = (uc + r.range(-spread, spread)).rem_euclid(1.0);
                let y = yc + r.range(-0.006, 0.006);
                let rad = r.range(2.0, 7.0);
                kettle_chip(ctx, &mut r, u * w, vy(y), rad, rad > 3.5);
            }
        }
        // lone chips anywhere, from knocks
        for _ in 0..12 {
            let (x, y, rad) = (r.next() * w, vy(r.range(0.02, 0.11)), r.range(1.5, 4.0));
            kettle_chip(ctx, &mut r, x, y, rad, false);
        }
    })
}

/// Stainless steel gone dull: its polish clouded, water spots dried white,
/// limescale run down from the lid and spout, rainbow heat tint and a brown
/// scorch at its foot, scratches round it
fn kettle_steel_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x57ee;
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let fy = 1.0 - j as f64 / h;
            for i in 0..n {
                let u = i as f64 / w;
                let m = kettle_noise(seed, u * 10.0, fy * 6.0, 10) * 0.5 + kettle_noise(seed ^ 1, u * 120.0, fy * 4.0, 120) * 0.5;
                let g0 = 118.0 + (m - 0.5) * 26.0;
                let mut c = [g0, g0 + 1.0, g0 + 2.0];
                let low = (1.0 - fy / 0.25).clamp(0.0, 1.0);
                let tint = low * low;
                let sc = [96.0, 74.0, 52.0];
                for k in 0..3 {
                    c[k] += (sc[k] - c[k]) * tint * 0.7;
                }
                let du = ((u - 0.25 + 0.5).rem_euclid(1.0) - 0.5).abs();
                let runs = (kettle_noise(seed ^ 3, u * 90.0, fy * 2.0, 90) - 0.55).max(0.0) * 2.4;
                let spots = (kettle_noise(seed ^ 5, u * 140.0, fy * 70.0, 140) - 0.72).max(0.0) * 3.0;
                let lime = (runs * ((fy - 0.55) / 0.12).clamp(0.0, 1.0) * (1.0 - (du / 0.35).min(1.0)) + spots).min(0.7);
                for k in 0..3 {
                    c[k] += (206.0 - c[k]) * lime;
                }
                out[i * 4..i * 4 + 4].copy_from_slice(&[c[0] as u8, c[1] as u8, c[2] as u8, 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        let mut r = SplitMix64::new(salt, 0x5c2a);
        ctx.set_line_width(0.7);
        for _ in 0..60 {
            let (x, y) = (r.next() * w, r.next() * h);
            let a = r.range(-0.4, 0.4);
            let l = r.range(4.0, 22.0);
            ctx.set_stroke_style(&format!("rgba({},{},{},0.35)", if r.next() < 0.5 { 70 } else { 170 }, 168, 166));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
}

/// The spout (the box's texture coordinates over its bounds: its mouth at
/// the top right): the body's finish, a limescale crust at the mouth and run
/// down under it, chips at the mouth and the seam
fn kettle_spout_tex(finish: usize, salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let base = match finish {
            0 => "rgb(212,204,184)",
            1 => "rgb(110,40,34)",
            _ => "rgb(120,121,122)",
        };
        ctx.set_fill_style(base);
        ctx.fill_rect(0, 0, w, h);
        let mut r = SplitMix64::new(salt, 0x590f);
        // the mouth's crust, at the part's top and its outer end
        for _ in 0..40 {
            let (x, y) = (w * r.range(0.62, 1.0), h * r.range(0.0, 0.42));
            let rr = r.range(2.0, 9.0);
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(224,220,206,0.55)");
            gr.add_color_stop(1, "rgba(224,220,206,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        // scorch low on it
        let gl = ctx.create_linear_gradient(0, h, 0, h * 0.6);
        gl.add_color_stop(0, "rgba(80,58,36,0.5)");
        gl.add_color_stop(1, "rgba(80,58,36,0)");
        ctx.set_fill_style(gl);
        ctx.fill_rect(0, 0, w, h);
        if finish < 2 {
            for _ in 0..5 {
                let (x, y) = (w * r.range(0.7, 0.98), h * r.range(0.02, 0.3));
                let rad = r.range(2.0, 5.0);
                kettle_chip(ctx, &mut r, x, y, rad, true);
            }
            for _ in 0..3 {
                let (x, y) = (w * r.range(0.0, 0.2), h * r.range(0.6, 1.0));
                let rad = r.range(2.0, 4.0);
                kettle_chip(ctx, &mut r, x, y, rad, true);
            }
        }
    })
}

/// Cork, its grains: light and dark granules pressed together, stained (the
/// three shades: a natural board, one gone dark with spills, a pale one)
fn kettle_cork_tex(shade: usize) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let base: [f64; 3] = [[150.0, 112.0, 72.0], [96.0, 68.0, 44.0], [176.0, 146.0, 104.0]][shade];
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = (i as f64, j as f64);
                let g = kettle_noise(31, x / 1.6, y / 1.6, 0) * 0.6 + kettle_noise(32, x / 4.0, y / 4.0, 0) * 0.4;
                let stain = kettle_noise(33, x / 30.0, y / 30.0, 0);
                let k = 0.72 + 0.5 * g - 0.25 * (stain - 0.4).max(0.0);
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    (base[0] * k).min(255.0) as u8,
                    (base[1] * k).min(255.0) as u8,
                    (base[2] * k).min(255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A wisp of vapour: soft, drawn out upward and thinning
fn kettle_vapour_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(64, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                for k in 0..36 {
                    let t = k as f64 / 35.0;
                    let x = w / 2.0 + (t * 5.0 + rnd() * 0.8).sin() * w * 0.14 * t;
                    let y = h * (0.97 - t * 0.85);
                    let r = w * (0.12 + t * 0.24);
                    let a = 0.28 * (1.0 - t).powf(0.7);
                    ctx.save();
                    ctx.translate(x, y);
                    ctx.scale(1.0, 1.4);
                    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
                    gr.add_color_stop(0, &format!("rgba(255,255,255,{a})"));
                    gr.add_color_stop(1, "rgba(255,255,255,0)");
                    ctx.set_fill_style(gr);
                    ctx.begin_path();
                    ctx.arc(0, 0, r, 0, 7);
                    ctx.fill();
                    ctx.restore();
                }
            })
        })
        .clone()
    })
}

// R753 Victorian glass-dome specimen display (floor): 3% chance per level.
fn place_specimen_domes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // Base wood pool.
    let wood_colors: [u32; 4] = [0x2a1810, 0x402015, 0x604020, 0x8b6f47];
    let wood_mat = lambert!({ color: wood_colors[ri(wood_colors.len())] });
    let gold_mat = lambert!({ color: 0xc0a060, emissive: 0x201810 });
    // Very transparent glass for dome.
    let glass_mat = lambert!({ color: 0xd8e0e8, transparent: true, opacity: 0.20 });
    let sd = group();
    // Wooden base — round platform w/ small foot.
    let base_r = 0.075;
    let base = mesh(pg("SpecimenDomes__base", cyl_geo!(base_r, base_r + 0.006, 0.015, 16)), &wood_mat);
    base.position().y = 0.008;
    sd.add(&base);
    // Ornamental base ring (torus).
    let base_ring = mesh(pg("SpecimenDomes__baseRing", torus_geo!(base_r, 0.004, 5, 16)), &gold_mat);
    base_ring.rotation().x = PI / 2.0;
    base_ring.position().y = 0.016;
    sd.add(&base_ring);
    // Small gold plaque on the plinth's sloping face.
    let plaque_mat = gold_mat.clone();
    let plaque = mesh(pg("SpecimenDomes__plaque", box_geo!(0.036, 0.010, 0.002)), &plaque_mat);
    let plaque_cant = 0.4f64.atan();
    plaque.position().set(0, 0.008, base_r + 0.0040);
    plaque.rotation().x = -plaque_cant;
    sd.add(&plaque);
    // Tiny engraving lines on plaque.
    let eng_mat = lambert!({ color: 0x402020 });
    for i in 0..2 {
        let eng_line = mesh(box_geo!(0.024, 0.001, 0.0005), &eng_mat);
        let d = (i as f64 - 0.5) * 0.004;
        eng_line.position().set(
            0,
            0.008 + d * plaque_cant.cos() + 0.0011 * plaque_cant.sin(),
            base_r + 0.0040 - d * plaque_cant.sin() + 0.0011 * plaque_cant.cos(),
        );
        eng_line.rotation().x = -plaque_cant;
        sd.add(&eng_line);
    }
    // Glass hemisphere dome (top-half sphere).
    let dome_r = base_r - 0.008;
    let dome = mesh(sphere_geo!(dome_r, 16, 10, 0, PI * 2.0, 0, PI / 2.0), &glass_mat);
    dome.position().y = 0.016;
    sd.add(&dome);
    // Faint rim outline where dome meets base.
    let dome_rim = mesh(pg("SpecimenDomes__domeRim", torus_geo!(dome_r, 0.002, 4, 20)), &gold_mat);
    dome_rim.rotation().x = PI / 2.0;
    dome_rim.position().y = 0.017;
    sd.add(&dome_rim);
    // 3-variant specimen inside — bird / butterfly / beetle.
    let spec_variant = (rnd() * 3.0).floor() as i32;
    if spec_variant == 0 {
        // Taxidermy bird on perch.
        let bird_colors: [u32; 5] = [0x8b4513, 0x6b8060, 0x502010, 0x605040, 0x804020];
        let bird_color = bird_colors[ri(bird_colors.len())];
        let bird_mat = lambert!({ color: bird_color });
        // Small wooden perch lying on the plinth top, across the bird.
        let perch = mesh(pg("SpecimenDomes__perch", cyl_geo!(0.003, 0.003, 0.040, 5)), &wood_mat);
        perch.rotation().x = PI / 2.0;
        perch.position().set(0.005, 0.0185, 0);
        sd.add(&perch);
        // Bird body, long along X like the head-tail line.
        let bird_body = mesh(pg("SpecimenDomes__birdBody", sphere_geo!(0.014, 10, 7)), &bird_mat);
        bird_body.scale().set(1.3, 0.85, 1.0);
        bird_body.position().y = 0.0325;
        sd.add(&bird_body);
        // Head (smaller sphere).
        let bird_head = mesh(pg("SpecimenDomes__birdHead", sphere_geo!(0.009, 8, 6)), &bird_mat);
        bird_head.position().set(0.015, 0.0415, 0);
        sd.add(&bird_head);
        // Tiny beak (cone), base 1.5 mm into the face.
        let beak = mesh(pg("SpecimenDomes__beak", cone_geo!(0.002, 0.006, 5)), lambert!({ color: 0xa07030 }));
        beak.rotation().z = -PI / 2.0;
        beak.position().set(0.0255, 0.0415, 0);
        sd.add(&beak);
        // Tail (small flat cone) fanning out flat, root in the rump.
        let tail = mesh(pg("SpecimenDomes__tail", cone_geo!(0.007, 0.016, 4)), &bird_mat);
        tail.scale().z = 0.4;
        tail.rotation().x = -PI / 2.0;
        tail.rotation().z = PI / 2.0;
        tail.position().set(-0.023, 0.0305, 0);
        sd.add(&tail);
        // 2 wings.
        for side in [-1.0f64, 1.0] {
            let wing = mesh(pg("SpecimenDomes__wing", sphere_geo!(0.010, 8, 5)), &bird_mat);
            wing.scale().set(0.4, 0.5, 1.3);
            wing.rotation().y = PI / 2.0;
            wing.position().set(-0.002, 0.0345, side * 0.0125);
            sd.add(&wing);
        }
        // 2 legs, standing on the perch.
        for side in [-1.0f64, 1.0] {
            let leg = mesh(cyl_geo!(0.0008, 0.0008, 0.008, 3), lambert!({ color: 0xa07030 }));
            leg.position().set(0.005, 0.0235, side * 0.005);
            sd.add(&leg);
        }
        // Tiny eye dot.
        let eye = mesh(pg("SpecimenDomes__bead", sphere_geo!(0.0015, 4, 4)), lambert!({ color: 0x080810 }));
        eye.position().set(0.020, 0.0435, 0.006);
        sd.add(&eye);
    } else if spec_variant == 1 {
        // Butterfly on pin.
        let wing_colors: [u32; 5] = [0xa02040, 0x2060a0, 0xd0a040, 0x408060, 0x804060];
        let wing_color = wing_colors[ri(wing_colors.len())];
        let wing_mat = lambert!({ color: wing_color, side: DoubleSide });
        let body_mat = lambert!({ color: 0x181818 });
        // Vertical pin (silver needle).
        let pin = mesh(pg("SpecimenDomes__pin", cyl_geo!(0.0005, 0.0005, 0.030, 4)), lambert!({ color: 0xa0a0a8 }));
        pin.position().y = 0.030;
        sd.add(&pin);
        // Pin head.
        let pin_head = mesh(pg("SpecimenDomes__bead", sphere_geo!(0.0015, 5, 4)), lambert!({ color: 0xa0a0a8 }));
        pin_head.position().y = 0.046;
        sd.add(&pin_head);
        // Central body.
        let bf_body = mesh(pg("SpecimenDomes__bfBody", cyl_geo!(0.001, 0.001, 0.016, 4)), &body_mat);
        bf_body.rotation().z = PI / 2.0;
        bf_body.position().y = 0.036;
        sd.add(&bf_body);
        // Head.
        let bf_head = mesh(pg("SpecimenDomes__bfHead", sphere_geo!(0.002, 4, 4)), &body_mat);
        bf_head.position().set(0.010, 0.036, 0);
        sd.add(&bf_head);
        // 4 wing plates (2 upper, 2 lower, symmetric).
        for side in [-1.0f64, 1.0] {
            let upper_wing = mesh(circle_geo!(0.014, 5), &wing_mat);
            upper_wing.scale().set(1.0, 0.7, 1.0);
            upper_wing.rotation().x = -PI / 2.0;
            upper_wing.rotation().z = side * 0.3;
            upper_wing.position().set(0, 0.036, side * 0.014);
            sd.add(&upper_wing);
            let lower_wing = mesh(circle_geo!(0.010, 5), &wing_mat);
            lower_wing.scale().set(1.0, 0.5, 1.0);
            lower_wing.rotation().x = -PI / 2.0;
            lower_wing.rotation().z = side * -0.2;
            lower_wing.position().set(-0.003, 0.036, side * 0.012);
            sd.add(&lower_wing);
            let marking = mesh(circle_geo!(0.003, 5), lambert!({ color: 0x181818, side: DoubleSide }));
            marking.rotation().x = -PI / 2.0;
            marking.position().set(0, 0.0362, side * 0.014);
            sd.add(&marking);
        }
        // 2 antennae.
        for side in [-1.0f64, 1.0] {
            let ant = mesh(pg("SpecimenDomes__antenna", cyl_geo!(0.0004, 0.0004, 0.010, 3)), &body_mat);
            ant.position().set(0.014, 0.038, side * 0.002);
            ant.rotation().z = -0.6;
            sd.add(&ant);
        }
    } else {
        // Beetle display — 3-4 beetles pinned in a row.
        let beetle_count = 3 + (rnd() * 2.0).floor() as i32;
        let beetle_colors: [u32; 4] = [0x1a1a08, 0x2a3010, 0x502010, 0x102030];
        for i in 0..beetle_count {
            let beetle_color = beetle_colors[ri(beetle_colors.len())];
            let beetle_mat = lambert!({ color: beetle_color, emissive: 0x080810 });
            let beetle_group = group();
            // Oval body (squashed sphere).
            let beetle = mesh(pg("SpecimenDomes__beetle", sphere_geo!(0.006, 8, 6)), &beetle_mat);
            beetle.scale().set(1.0, 0.4, 1.6);
            beetle.position().y = 0.022;
            beetle_group.add(&beetle);
            // Small head bump.
            let b_head = mesh(pg("SpecimenDomes__beetleHead", sphere_geo!(0.003, 5, 4)), &beetle_mat);
            b_head.position().set(0, 0.024, 0.009);
            beetle_group.add(&b_head);
            // 6 tiny legs (3 per side).
            for side in [-1.0f64, 1.0] {
                for j in 0..3 {
                    let leg = mesh(cyl_geo!(0.0004, 0.0004, 0.006, 3), &beetle_mat);
                    leg.rotation().z = side * 0.5;
                    leg.position().set(side * 0.006, 0.021, -0.004 + j as f64 * 0.004);
                    beetle_group.add(&leg);
                }
            }
            // Tiny elytra split line down back.
            let split = mesh(box_geo!(0.0004, 0.001, 0.010), lambert!({ color: 0x080810 }));
            split.position().set(0, 0.024, 0);
            beetle_group.add(&split);
            // Pin from the plinth top up through the beetle.
            let b_pin = mesh(pg("SpecimenDomes__pin", cyl_geo!(0.0004, 0.0004, 0.014, 3)), lambert!({ color: 0xa0a0a8 }));
            b_pin.position().set(0, 0.0225, 0);
            beetle_group.add(&b_pin);
            // Position beetles in a row.
            beetle_group.position().x = (i as f64 - (beetle_count - 1) as f64 / 2.0) * 0.020;
            sd.add(&beetle_group);
        }
        // Small paper label in front of each beetle.
        for i in 0..beetle_count {
            let label = mesh(pg("SpecimenDomes__label", box_geo!(0.010, 0.0008, 0.004)), lambert!({ color: 0xf0e8c0 }));
            label.position().set((i as f64 - (beetle_count - 1) as f64 / 2.0) * 0.020, 0.0159, -0.020);
            sd.add(&label);
        }
    }
    sd.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    sd.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sd);
}

// R754 vertical secretary desk with drop-front writing surface (floor): 2%.
fn place_secretary_desks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // 4-color wood pool.
    let wood_colors: [u32; 4] = [0x604020, 0x2a1810, 0x8b6f47, 0x502010];
    let wood_mat = lambert!({ color: wood_colors[ri(wood_colors.len())] });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let sec = group();
    // Main dimensions.
    let (body_w, body_d, body_h) = (0.55, 0.35, 0.60);
    let leg_h = 0.20;
    // Cabinet body.
    let cabinet = mesh(pg("SecretaryDesks__cabinet", box_geo!(body_w, body_h, body_d)), &wood_mat);
    cabinet.position().y = leg_h + body_h / 2.0;
    sec.add(&cabinet);
    // 4 tapered legs at corners.
    for cx in [-body_w * 0.42, body_w * 0.42] {
        for cz in [-body_d * 0.42, body_d * 0.42] {
            let leg = mesh(pg("SecretaryDesks__leg", cyl_geo!(0.014, 0.020, leg_h, 6)), &wood_mat);
            leg.position().set(cx, leg_h / 2.0, cz);
            sec.add(&leg);
        }
    }
    // 2 drawers on cabinet front + brass drawer pulls.
    let drawer_y0 = leg_h + body_h * 0.14;
    let drawer_h = body_h * 0.28;
    for i in 0..2 {
        let d_y = drawer_y0 + i as f64 * (drawer_h + 0.008);
        let drawer = mesh(pg("SecretaryDesks__drawer", box_geo!(body_w * 0.95, drawer_h, 0.005)), lambert!({ color: 0x4a3020 }));
        drawer.position().set(0, d_y, body_d / 2.0 + 0.003);
        sec.add(&drawer);
        for pside in [-1.0f64, 1.0] {
            let pull = mesh(pg("SecretaryDesks__pull", cyl_geo!(0.008, 0.008, 0.006, 8)), &brass_mat);
            pull.rotation().x = PI / 2.0;
            pull.position().set(pside * body_w * 0.20, d_y, body_d / 2.0 + 0.008);
            sec.add(&pull);
            let backplate = mesh(pg("SecretaryDesks__backplate", box_geo!(0.020, 0.010, 0.001)), &brass_mat);
            backplate.position().set(pside * body_w * 0.20, d_y, body_d / 2.0 + 0.006);
            sec.add(&backplate);
        }
    }
    // Top compartment — drop-front writing surface (currently open).
    let top_y = leg_h + body_h;
    let drop_w = body_w * 0.85;
    let drop_h = 0.30;
    // The group sits on the hinge line; local +Z runs out along the open fall.
    let drop_surf = group();
    drop_surf.position().set(0, top_y, body_d / 2.0);
    drop_surf.rotation().x = 0.05; // Almost fully open
    let drop_panel = mesh(pg("SecretaryDesks__dropPanel", box_geo!(drop_w, 0.010, drop_h)), &wood_mat);
    drop_panel.position().set(0, -0.005, drop_h / 2.0);
    drop_surf.add(&drop_panel);
    // Small brass hinges at the joint.
    for hside in [-1.0f64, 1.0] {
        let hinge = mesh(pg("SecretaryDesks__hinge", box_geo!(0.012, 0.012, 0.002)), &brass_mat);
        hinge.position().set(hside * drop_w * 0.30, -0.012, 0.001);
        drop_surf.add(&hinge);
    }
    sec.add(&drop_surf);
    // Support stays holding the drop-surface open.
    for cside in [-1.0f64, 1.0] {
        let arm = mesh(pg("SecretaryDesks__stay", cyl_geo!(0.001, 0.001, 0.14, 3)), &brass_mat);
        arm.position().set(cside * drop_w * 0.45, top_y - 0.066, body_d / 2.0 + 0.045);
        arm.rotation().x = 0.7;
        sec.add(&arm);
    }
    // Upper compartment (behind the dropped panel).
    let upper_comp_h = 0.30;
    let upper_comp = mesh(pg("SecretaryDesks__upper", box_geo!(body_w * 0.85, upper_comp_h, body_d * 0.75)), &wood_mat);
    upper_comp.position().set(0, top_y + upper_comp_h / 2.0, -body_d * 0.05);
    sec.add(&upper_comp);
    // Pigeonhole cubbies.
    let hole_count = 6;
    let hole_w = body_w * 0.85 / hole_count as f64 * 0.85;
    let hole_h = upper_comp_h * 0.55;
    for i in 0..hole_count {
        let hole_x = -body_w * 0.85 / 2.0 + (i as f64 + 0.5) * (body_w * 0.85 / hole_count as f64);
        let hole_back = mesh(box_geo!(hole_w, hole_h, 0.001), lambert!({ color: 0x0a0a0a }));
        hole_back.position().set(hole_x, top_y + upper_comp_h * 0.55, body_d * 0.32);
        sec.add(&hole_back);
        // 30% chance small rolled paper visible in cubby.
        if rnd() < 0.3 {
            let paper_mat = lambert!({ color: 0xf0e8d0 });
            let roll = mesh(pg("SecretaryDesks__roll", cyl_geo!(0.004, 0.004, hole_w * 0.7, 6)), &paper_mat);
            roll.rotation().z = PI / 2.0;
            roll.position().set(hole_x, top_y + upper_comp_h * 0.4, body_d * 0.32);
            sec.add(&roll);
        }
    }
    // 4 small drawers above pigeonholes.
    let small_drawer_y = top_y + upper_comp_h * 0.10;
    for i in 0..4 {
        let sd_x = -body_w * 0.85 / 2.0 + (i as f64 + 0.5) * (body_w * 0.85 / 4.0);
        let sdrawer = mesh(pg("SecretaryDesks__sdrawer", box_geo!(body_w * 0.85 / 4.0 * 0.85, 0.05, 0.005)), lambert!({ color: 0x4a3020 }));
        sdrawer.position().set(sd_x, small_drawer_y, body_d * 0.32 + 0.003);
        sec.add(&sdrawer);
        let knob = mesh(pg("SecretaryDesks__knob", sphere_geo!(0.003, 5, 4)), &brass_mat);
        knob.position().set(sd_x, small_drawer_y, body_d * 0.32 + 0.008);
        sec.add(&knob);
    }
    // 60% chance: papers on the dropped writing surface (children of drop_surf).
    if rnd() < 0.6 {
        let paper_mat = lambert!({ color: 0xf0e8d0 });
        let paper_count = 1 + (rnd() * 3.0).floor() as i32;
        for i in 0..paper_count {
            let paper = mesh(pg("SecretaryDesks__paper", box_geo!(0.10, 0.001, 0.14)), &paper_mat);
            paper.position().set((rnd() - 0.5) * drop_w * 0.5, 0.0005 + i as f64 * 0.001, drop_h * 0.5);
            paper.rotation().y = (rnd() - 0.5) * 0.3;
            drop_surf.add(&paper);
        }
        // 40% chance: small inkwell on surface.
        if rnd() < 0.4 {
            let ink_glass_mat = lambert!({ color: 0x505860, transparent: true, opacity: 0.55 });
            let inkwell_jar = mesh(cyl_geo!(0.012, 0.012, 0.014, 8), &ink_glass_mat);
            inkwell_jar.position().set(drop_w * 0.35, 0.007, drop_h * 0.3);
            drop_surf.add(&inkwell_jar);
            let ink_top = mesh(circle_geo!(0.010, 8), lambert!({ color: 0x080810 }));
            ink_top.rotation().x = -PI / 2.0;
            ink_top.position().set(drop_w * 0.35, 0.0125, drop_h * 0.3);
            drop_surf.add(&ink_top);
        }
    }
    // Top ornamental cornice.
    let cornice = mesh(pg("SecretaryDesks__cornice", box_geo!(body_w + 0.020, 0.020, body_d * 0.85 + 0.020)), &wood_mat);
    cornice.position().y = top_y + upper_comp_h + 0.010;
    sec.add(&cornice);
    sec.position().set(g.wx(x) + (rnd() - 0.5) * 0.6, 0, g.wz(z) + (rnd() - 0.5) * 0.6);
    sec.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sec);
}

// R754 tarnished gilded ornate wall mirror texture.
game::lazy_static_tex!(
    tarnished_mirror_texture,
    canvas_tex(256, 320, |ctx, w, h| {
        // Base mirror gradient — dark silver reflection tone.
        let grad = ctx.create_linear_gradient(0, 0, w, h);
        grad.add_color_stop(0, "#3a3a45");
        grad.add_color_stop(0.5, "#5a5a68");
        grad.add_color_stop(1, "#2e2e38");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Tarnish patches — dark green/brown blotches at corners + edges.
        ctx.set_fill_style("rgba(30,40,20,0.45)");
        for _ in 0..6 {
            let tx = if rnd() < 0.6 {
                if rnd() < 0.5 {
                    0.0
                } else {
                    w
                }
            } else {
                rnd() * w
            };
            let ty = if rnd() < 0.6 {
                if rnd() < 0.5 {
                    0.0
                } else {
                    h
                }
            } else {
                rnd() * h
            };
            ctx.begin_path();
            let rx = 25.0 + rnd() * 30.0;
            let ry = 20.0 + rnd() * 25.0;
            let rot = rnd() * PI;
            ctx.ellipse(tx, ty, rx, ry, rot, 0, PI * 2.0);
            ctx.fill();
        }
        // Rust-red small spots scattered.
        ctx.set_fill_style("rgba(100,50,20,0.4)");
        for _ in 0..20 {
            ctx.begin_path();
            let (ax, ay, ar) = (rnd() * w, rnd() * h, 2.0 + rnd() * 4.0);
            ctx.arc(ax, ay, ar, 0, PI * 2.0);
            ctx.fill();
        }
        // 3-4 crack lines emanating from a starburst point.
        let ox = w * (0.3 + rnd() * 0.4);
        let oy = h * (0.3 + rnd() * 0.4);
        ctx.set_stroke_style("rgba(15,15,20,0.85)");
        ctx.set_line_width(1.5);
        let crack_count = 3 + (rnd() * 2.0).floor() as i32;
        for i in 0..crack_count {
            let angle = (i as f64 / crack_count as f64) * PI * 2.0 + rnd() * 0.5;
            // Zigzag crack line — up to 5 segments with random angle drift.
            let (mut cx, mut cy) = (ox, oy);
            let mut ang = angle;
            ctx.begin_path();
            ctx.move_to(cx, cy);
            for _ in 0..5 {
                let seg_len = 25.0 + rnd() * 30.0;
                ang += (rnd() - 0.5) * 0.6;
                cx += ang.cos() * seg_len;
                cy += ang.sin() * seg_len;
                ctx.line_to(cx, cy);
                if cx < 0.0 || cx > w || cy < 0.0 || cy > h {
                    break;
                }
            }
            ctx.stroke();
            // Secondary hairline branch (30% per crack).
            if rnd() < 0.3 {
                ctx.set_stroke_style("rgba(20,20,30,0.5)");
                ctx.set_line_width(0.7);
                ctx.begin_path();
                let mid_x = ox + angle.cos() * 40.0;
                let mid_y = oy + angle.sin() * 40.0;
                ctx.move_to(mid_x, mid_y);
                ctx.line_to(mid_x + (angle + 0.8).cos() * 30.0, mid_y + (angle + 0.8).sin() * 30.0);
                ctx.stroke();
                ctx.set_stroke_style("rgba(15,15,20,0.85)");
                ctx.set_line_width(1.5);
            }
        }
        // Small highlight dot at crack origin (impact point).
        ctx.set_fill_style("rgba(240,240,240,0.65)");
        ctx.begin_path();
        ctx.arc(ox, oy, 3, 0, PI * 2.0);
        ctx.fill();
        // Faint glass-warp reflection lines (2 subtle diagonals).
        ctx.set_stroke_style("rgba(200,210,220,0.10)");
        ctx.set_line_width(30);
        ctx.begin_path();
        ctx.move_to(0, h * 0.3);
        ctx.line_to(w, h * 0.2);
        ctx.stroke();
        ctx.begin_path();
        ctx.move_to(0, h * 0.7);
        ctx.line_to(w, h * 0.8);
        ctx.stroke();
    })
);

fn place_tarnished_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let (cx, cz, cdx, cdz) = candidates[ri(candidates.len())];
    // Ornate gilded frame — gold w/ tarnish tinge.
    let frame_colors: [u32; 4] = [0xa08040, 0x8b6f30, 0x907040, 0x604828];
    let frame_mat = lambert!({ color: frame_colors[ri(frame_colors.len())], emissive: 0x1a1408 });
    let mirror_mat = lambert!({ map: tarnished_mirror_texture(), side: DoubleSide });
    let tm = group();
    // Frame outer bars.
    let (mw, mh, mt, mfw) = (0.40, 0.52, 0.020, 0.036);
    for (bar_w, bar_h, bx, by) in [
        (mw + mfw * 2.0, mfw, 0.0, mh / 2.0 + mfw / 2.0),
        (mw + mfw * 2.0, mfw, 0.0, -mh / 2.0 - mfw / 2.0),
        (mfw, mh, -mw / 2.0 - mfw / 2.0, 0.0),
        (mfw, mh, mw / 2.0 + mfw / 2.0, 0.0),
    ] {
        let bar = mesh(pg("TarnishedMirrors__bar", box_geo!(bar_w, bar_h, mt)), &frame_mat);
        bar.position().set(bx, by, 0);
        tm.add(&bar);
    }
    // Ornate corner decorations.
    for (cx_or, cy_or) in [
        (-mw / 2.0 - mfw / 2.0, mh / 2.0 + mfw / 2.0),
        (mw / 2.0 + mfw / 2.0, mh / 2.0 + mfw / 2.0),
        (-mw / 2.0 - mfw / 2.0, -mh / 2.0 - mfw / 2.0),
        (mw / 2.0 + mfw / 2.0, -mh / 2.0 - mfw / 2.0),
    ] {
        let corner_sphere = mesh(pg("TarnishedMirrors__rosette", sphere_geo!(0.020, 8, 6)), &frame_mat);
        corner_sphere.position().set(cx_or, cy_or, mt / 2.0);
        tm.add(&corner_sphere);
    }
    // Ornate top pediment.
    let pediment = mesh(pg("TarnishedMirrors__pediment", box_geo!(mw * 0.6, 0.025, mt * 0.7)), &frame_mat);
    pediment.position().set(0, mh / 2.0 + mfw + 0.012, 0);
    tm.add(&pediment);
    // Central decorative rosette on pediment.
    let rosette = mesh(pg("TarnishedMirrors__rosette", sphere_geo!(0.014, 8, 6)), &frame_mat);
    rosette.position().set(0, mh / 2.0 + mfw + 0.025, mt / 2.0);
    tm.add(&rosette);
    // Bottom pendant.
    let pendant = mesh(pg("TarnishedMirrors__pendant", sphere_geo!(0.012, 6, 5)), &frame_mat);
    pendant.position().set(0, -mh / 2.0 - mfw - 0.010, mt / 2.0);
    tm.add(&pendant);
    // Mirror surface w/ tarnish + crack texture.
    let surface = mesh(plane_geo!(mw, mh), &mirror_mat);
    surface.position().set(0, 0, mt / 2.0 + 0.001);
    tm.add(&surface);
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.014);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.014);
    let center_y = 1.65;
    if cdx.abs() == 1 {
        tm.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        tm.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.4);
    } else {
        tm.rotation().y = if cdz > 0 { PI } else { 0.0 };
        tm.position().set(wall_x + (rnd() - 0.5) * 0.4, center_y, wall_z);
    }
    tm.rotation().z = (rnd() - 0.5) * 0.05;
    g.world.add(&tm);
}

// R756 dust-sheet-covered furniture (floor): 3.5% chance per level. Each
// piece is one sheet of the Watcher's linen thrown over a chair, a table or
// a wingback (`bodies.bin`'s `sheet_*`, `assets/src/creatures/sheeted.py`):
// at a glance one is the other. (The draws of the old box-built shapes are
// kept, in their order, so the placers after this one draw what they did.)
fn place_sheet_covered_furniture(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.965 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let _sheet_colour = ri(4);
    let variant = (rnd() * 3.0).floor() as i32;
    match variant {
        0 => {
            // (three folds: where and how turned)
            for _ in 0..9 {
                rnd();
            }
        }
        1 => {
            // (a fold's place, and 30%: a bump's)
            rnd();
            if rnd() < 0.3 {
                rnd();
                rnd();
            }
        }
        _ => {
            // (four folds)
            for _ in 0..12 {
                rnd();
            }
        }
    }
    let scf = group();
    let name = ["sheet_chair", "sheet_table", "sheet_wingback"][variant.clamp(0, 2) as usize];
    if let Some(rig) = game::entities::body::build_body(name, &game::entities::watcher::material) {
        scf.add(&rig.root);
    }
    // 40% chance: thin dust layer on floor around base.
    if rnd() < 0.4 {
        let dust_mat = lambert!({ color: 0xa89880, transparent: true, opacity: 0.35 });
        let dust = mesh(circle_geo!(0.55, 12), &dust_mat);
        dust.rotation().x = -PI / 2.0;
        dust.position().y = 0.001;
        scf.add(&dust);
    }
    scf.position().set(g.wx(x) + (rnd() - 0.5) * 0.8, 0, g.wz(z) + (rnd() - 0.5) * 0.8);
    scf.rotation().y = rnd() * PI * 2.0;
    g.world.add(&scf);
    // R757 the wingback's sheet breathes, very slightly (as it did)
    if variant >= 2 {
        let breath_freq = (PI * 2.0) / (30.0 + rnd() * 15.0);
        let breath_phase = rnd() * PI * 2.0;
        let sheet = scf.children().into_iter().next();
        if let Some(sheet) = sheet {
            g.animate(move |_g, _dt, t| {
                sheet.position().y = (t * breath_freq + breath_phase).sin() * 0.0025;
            });
        }
    }
}

// R756 Victorian servant's bell pull system (wall+ceiling): 2.5% chance per level.
fn place_servant_bell_pulls(g: &mut Game, _dist: &[i32]) {
    g.pp.bell_pull_positions.clear(); // R757 reset per level
    if rnd() < 0.975 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let (cx, cz, cdx, cdz) = candidates[ri(candidates.len())];
    // 4-color cord pool: burgundy, forest, navy, dark gold.
    let cord_colors: [u32; 4] = [0x501020, 0x203010, 0x102040, 0x604020];
    let cord_mat = lambert!({ color: cord_colors[ri(cord_colors.len())] });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let gold_mat = lambert!({ color: 0xc0a060, emissive: 0x201810 });
    let bp = group();
    // Wall-mounted brass bell bracket at top of wall.
    let bracket_y = 2.30;
    let bracket = mesh(pg("ServantBellPulls__bracket", box_geo!(0.08, 0.020, 0.030)), &brass_mat);
    bracket.position().set(0, bracket_y, 0.015);
    bp.add(&bracket);
    // Brass bell body — open bell shape.
    let bell = mesh(pg("ServantBellPulls__bell", cyl_geo!(0.045, 0.030, 0.075, 10, 1, true)), &brass_mat);
    bell.position().set(0, bracket_y - 0.045, 0.045);
    bp.add(&bell);
    // Bell top cap.
    let bell_top = mesh(pg("ServantBellPulls__bellTop", cyl_geo!(0.030, 0.030, 0.008, 10)), &brass_mat);
    bell_top.position().set(0, bracket_y - 0.008, 0.045);
    bp.add(&bell_top);
    // Clapper.
    let clapper = mesh(pg("ServantBellPulls__clapper", sphere_geo!(0.008, 5, 4)), &brass_mat);
    clapper.position().set(0, bracket_y - 0.055, 0.045);
    bp.add(&clapper);
    // Small mounting screws visible on bracket.
    for side in [-1.0f64, 1.0] {
        let screw = mesh(pg("ServantBellPulls__screw", cyl_geo!(0.003, 0.003, 0.003, 5)), &brass_mat);
        screw.rotation().x = PI / 2.0;
        screw.position().set(side * 0.028, bracket_y, 0.003);
        bp.add(&screw);
    }
    // Spring coil connecting bracket to top of pull cord.
    let spring = mesh(pg("ServantBellPulls__spring", torus_geo!(0.008, 0.001, 4, 6)), &brass_mat);
    spring.rotation().x = PI / 2.0;
    spring.position().set(0, bracket_y - 0.020, 0.020);
    bp.add(&spring);
    // Long fabric pull cord.
    let cord_top_y = bracket_y - 0.020;
    let cord_bot_y = 1.20;
    let cord_len = cord_top_y - cord_bot_y;
    let cord = mesh(pg("ServantBellPulls__cord", cyl_geo!(0.005, 0.005, cord_len, 6)), &cord_mat);
    cord.position().set(0, (cord_top_y + cord_bot_y) / 2.0, 0.020);
    bp.add(&cord);
    // Twisted-braid ridges on cord.
    let mut braids: Vec<Obj> = vec![];
    for i in 0..3 {
        let braid = mesh(pg("ServantBellPulls__braid", torus_geo!(0.006, 0.001, 4, 8)), &cord_mat);
        braid.rotation().x = PI / 2.0;
        braid.position().set(0, cord_bot_y + (i as f64 + 1.0) * (cord_len / 4.0), 0.020);
        bp.add(&braid);
        braids.push(braid);
    }
    // Ornate tassel at bottom — bulb.
    let bulb = mesh(pg("ServantBellPulls__bulb", sphere_geo!(0.025, 8, 6)), &cord_mat);
    bulb.scale().y = 1.3;
    bulb.position().set(0, cord_bot_y - 0.020, 0.020);
    bp.add(&bulb);
    // Gold band around bulb top.
    let bulb_band = mesh(pg("ServantBellPulls__band", torus_geo!(0.021, 0.003, 4, 10)), &gold_mat);
    bulb_band.rotation().x = PI / 2.0;
    bulb_band.position().set(0, cord_bot_y - 0.005, 0.020);
    bp.add(&bulb_band);
    // 8 tassel fringe strands hanging from bulb bottom.
    for i in 0..8 {
        let angle = (i as f64 / 8.0) * PI * 2.0;
        let strand = mesh(pg("ServantBellPulls__strand", cyl_geo!(0.001, 0.001, 0.055, 3)), &cord_mat);
        strand.position().set(angle.cos() * 0.010, cord_bot_y - 0.065, 0.020 + angle.sin() * 0.010);
        strand.rotation().x = angle.sin() * 0.15;
        strand.rotation().z = -angle.cos() * 0.15;
        bp.add(&strand);
    }
    // R756 subtle sway animator on the cord + tassel + strands + braids.
    let sway_group = group(); // unused
    let sway_freq = 0.28 + rnd() * 0.20;
    let sway_phase = rnd() * PI * 2.0;
    let sway_center_y = cord_bot_y - 0.020;
    let mut swayers: Vec<Obj> = vec![cord.clone(), bulb.clone(), bulb_band.clone()];
    // Also all 8 strands (last 8 added to bp).
    let kids = bp.children();
    let strands_start = kids.len() - 8;
    for i in strands_start..kids.len() {
        swayers.push(kids[i].clone());
    }
    // Also the 3 braid rings.
    for b in &braids {
        swayers.push(b.clone());
    }
    let sway_base_x: Vec<f64> = swayers.iter().map(|s| s.pos().x).collect();
    g.animate(move |_g, _dt, t| {
        let sway_angle = (t * sway_freq + sway_phase).sin() * 0.020;
        for i in 0..swayers.len() {
            let s = &swayers[i];
            let y_delta = s.pos().y - cord_top_y; // negative
            s.position().x = sway_base_x[i] + sway_angle.sin() * y_delta * -1.0;
        }
        // The cord pivots at its top instead of sliding sideways off the spring.
        cord.rotation().z = sway_angle;
    });
    // Mount at wall position.
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.005);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.005);
    if cdx.abs() == 1 {
        bp.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        bp.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 0.4);
    } else {
        bp.rotation().y = if cdz > 0 { PI } else { 0.0 };
        bp.position().set(wall_x + (rnd() - 0.5) * 0.4, 0, wall_z);
    }
    g.world.add(&bp);
    let p = bp.pos();
    g.pp.bell_pull_positions.push(v3(p.x, 0, p.z)); // R757
}

// R758 Victorian floor hall stand (floor): 2.5% chance per level.
fn place_victorian_hall_stands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // 4-color wood pool.
    let wood_colors: [u32; 4] = [0x604020, 0x2a1810, 0x8b6f47, 0x502010];
    let wood_mat = lambert!({ color: wood_colors[ri(wood_colors.len())] });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let hs = group();
    // Base tray for umbrellas.
    let tray_r = 0.16;
    let tray = mesh(pg("VictorianHallStands__tray", cyl_geo!(tray_r, tray_r, 0.020, 14)), &wood_mat);
    tray.position().y = 0.010;
    hs.add(&tray);
    // Tray rim (raised).
    let tray_rim = mesh(pg("VictorianHallStands__rim", torus_geo!(tray_r, 0.006, 5, 14)), &wood_mat);
    tray_rim.rotation().x = PI / 2.0;
    tray_rim.position().y = 0.020;
    hs.add(&tray_rim);
    // Central vertical post — 180 cm tall.
    let post_h = 1.80;
    let post = mesh(pg("VictorianHallStands__post", cyl_geo!(0.020, 0.030, post_h, 8)), &wood_mat);
    post.position().y = 0.020 + post_h / 2.0;
    hs.add(&post);
    // Ornamental turned rings on post at 3 heights.
    for ry in [0.35, 0.65, 1.10] {
        let ring = mesh(pg("VictorianHallStands__ring", torus_geo!(0.024, 0.005, 5, 12)), &wood_mat);
        ring.rotation().x = PI / 2.0;
        ring.position().y = 0.020 + ry;
        hs.add(&ring);
    }
    // 6 hooks around post at top (3 upper + 3 middle).
    let hook_ys = [post_h - 0.020, post_h - 0.180];
    for level in 0..2 {
        for i in 0..3 {
            let angle = (i as f64 / 3.0) * PI * 2.0 + level as f64 * (PI / 3.0);
            let arm = mesh(pg("VictorianHallStands__arm", cyl_geo!(0.004, 0.004, 0.055, 4)), &wood_mat);
            arm.rotation().z = PI / 2.0;
            arm.rotation().y = -angle;
            arm.position().set(angle.cos() * 0.030, 0.020 + hook_ys[level], angle.sin() * 0.030);
            hs.add(&arm);
            // Hook cap — small brass ball.
            let cap = mesh(pg("VictorianHallStands__cap", sphere_geo!(0.008, 6, 5)), &brass_mat);
            cap.position().set(angle.cos() * 0.058, 0.020 + hook_ys[level] - 0.006, angle.sin() * 0.058);
            hs.add(&cap);
        }
    }
    // Central top ornamental finial.
    let finial = mesh(pg("VictorianHallStands__finial", sphere_geo!(0.024, 8, 6)), &wood_mat);
    finial.position().y = 0.020 + post_h + 0.015;
    hs.add(&finial);
    // Small oval mirror at mid-height, hanging flat against the post.
    let mirror_angle = rnd() * PI * 2.0;
    let mirror = group();
    let mirror_r = 0.09;
    let m_frame = mesh(pg("VictorianHallStands__mirror", torus_geo!(mirror_r, 0.008, 5, 16)), &wood_mat);
    mirror.add(&m_frame);
    let m_surf = mesh(circle_geo!(mirror_r - 0.005, 16), lambert!({ color: 0x707078, emissive: 0x101014 }));
    mirror.add(&m_surf);
    m_frame.scale().y = 1.30; // Slight oval elongation.
    m_surf.scale().y = 1.30;
    mirror.position().set(mirror_angle.cos() * 0.033, 0.85, mirror_angle.sin() * 0.033);
    mirror.rotation().y = -mirror_angle + PI / 2.0;
    hs.add(&mirror);
    // 2-3 umbrellas in tray base (60% chance).
    if rnd() < 0.6 {
        let umb_count = 2 + (rnd() * 2.0).floor() as i32;
        let umb_colors: [u32; 5] = [0x181818, 0x2a1010, 0x102040, 0x203010, 0x604020];
        for i in 0..umb_count {
            let umb_mat = lambert!({ color: umb_colors[ri(umb_colors.len())] });
            // Umbrella shaft (leaning outward).
            let u_angle = (i as f64 / umb_count as f64) * PI * 2.0 + rnd() * 0.5;
            let shaft = mesh(pg("VictorianHallStands__umbrella", cyl_geo!(0.004, 0.004, 0.75, 4)), &umb_mat);
            shaft.position().set(u_angle.cos() * tray_r * 0.5, 0.35, u_angle.sin() * tray_r * 0.5);
            shaft.rotation().x = u_angle.sin() * 0.12;
            shaft.rotation().z = -u_angle.cos() * 0.12;
            hs.add(&shaft);
            // J-hook handle at top: upright half-ring whose stick end sits on the shaft top.
            let handle_hook = mesh(pg("VictorianHallStands__crook", torus_geo!(0.018, 0.004, 4, 8, PI)), &umb_mat);
            let mut shaft_top = v3(0, 0.375, 0);
            shaft_top.apply_euler(&shaft.rot());
            shaft_top.add(&shaft.pos());
            handle_hook.rotation().y = PI - u_angle;
            handle_hook.position().set(shaft_top.x + u_angle.cos() * 0.018, shaft_top.y - 0.002, shaft_top.z + u_angle.sin() * 0.018);
            hs.add(&handle_hook);
        }
    }
    // 50% chance: 1-2 hats on upper hooks.
    if rnd() < 0.5 {
        let hat_count = 1 + (rnd() * 2.0).floor() as i32;
        let hat_colors: [u32; 4] = [0x181818, 0x2a1810, 0x604020, 0x502010];
        for i in 0..hat_count {
            let hat_mat = lambert!({ color: hat_colors[ri(hat_colors.len())] });
            let hat_angle0 = (i as f64 / hat_count as f64) * PI * 2.0 + rnd() * 0.5;
            // Hang it on the nearest upper hook.
            let hat_angle = round_half_up(hat_angle0 / (PI * 2.0 / 3.0)) * (PI * 2.0 / 3.0);
            let brim_y = 0.020 + hook_ys[0] + 0.004;
            // Top hat / bowler variant.
            let is_top_hat = rnd() < 0.5;
            if is_top_hat {
                let crown = mesh(pg("VictorianHallStands__topHat", cyl_geo!(0.060, 0.058, 0.090, 10)), &hat_mat);
                crown.position().set(hat_angle.cos() * 0.10, brim_y + 0.004 + 0.045, hat_angle.sin() * 0.10);
                hs.add(&crown);
                let brim = mesh(pg("VictorianHallStands__topHatBrim", cyl_geo!(0.080, 0.080, 0.008, 12)), &hat_mat);
                brim.position().set(hat_angle.cos() * 0.10, brim_y + 0.004, hat_angle.sin() * 0.10);
                hs.add(&brim);
            } else {
                let bowler_crown = mesh(pg("VictorianHallStands__bowler", sphere_geo!(0.055, 10, 6, 0, PI * 2.0, 0, PI * 0.55)), &hat_mat);
                bowler_crown.position().set(hat_angle.cos() * 0.10, brim_y + 0.003, hat_angle.sin() * 0.10);
                hs.add(&bowler_crown);
                let bowler_brim = mesh(pg("VictorianHallStands__bowlerBrim", cyl_geo!(0.065, 0.065, 0.006, 12)), &hat_mat);
                bowler_brim.position().set(hat_angle.cos() * 0.10, brim_y + 0.003, hat_angle.sin() * 0.10);
                hs.add(&bowler_brim);
            }
        }
    }
    // 40% chance: single scarf hanging from middle hook.
    if rnd() < 0.4 {
        let scarf_colors: [u32; 5] = [0xa02040, 0x2060a0, 0xd0a040, 0x408060, 0x804060];
        let scarf_mat = lambert!({ color: scarf_colors[ri(scarf_colors.len())] });
        let s_angle = rnd() * PI * 2.0;
        let scarf = mesh(pg("VictorianHallStands__scarf", box_geo!(0.005, 0.35, 0.045)), &scarf_mat);
        scarf.position().set(s_angle.cos() * 0.070, 0.020 + hook_ys[1] - 0.175, s_angle.sin() * 0.070);
        hs.add(&scarf);
        // Slight fringe bumps at bottom.
        for i in 0..3 {
            let fringe = mesh(pg("VictorianHallStands__fringe", box_geo!(0.005, 0.010, 0.008)), &scarf_mat);
            fringe.position().set(s_angle.cos() * 0.070, 0.020 + hook_ys[1] - 0.360, s_angle.sin() * 0.070 + (i as f64 - 1.0) * 0.010);
            hs.add(&fringe);
        }
    }
    hs.position().set(g.wx(x) + (rnd() - 0.5) * 0.6, 0, g.wz(z) + (rnd() - 0.5) * 0.6);
    hs.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hs);
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
