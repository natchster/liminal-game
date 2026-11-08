//! Batch `r13a`'s props in this crate, made real (`docs/PROPS.md` P2): its
//! placers and what only they use.
use super::*;

/// Soap bar colour pool per weather; other keys get `[0xf0e8d0, 0xd8c0d0, 0xa8d0d8, 0xe8d0a0]`
fn soap_bar_color_by_weather(key: &str) -> &'static [u32] {
    match key {
        "wet" => &[0xa8d0d8, 0xf0e8d0],
        "electric" => &[0xe8d0a0, 0xf0e8d0],
        "haunted" => &[0x8a3040, 0x603030],
        "derelict" => &[0x8a7050, 0xa08060],
        "still" => &[0xf0e8d0, 0xd8c0d0],
        _ => &[0xf0e8d0, 0xd8c0d0, 0xa8d0d8, 0xe8d0a0],
    }
}

/// Toothbrush colour pool per weather; other keys get `[0x40a0d8, 0xf0f0f0, 0xd0d060, 0xd06060]`
fn toothbrush_color_by_weather(key: &str) -> &'static [u32] {
    match key {
        "wet" => &[0x40a0d8, 0xf0f0f0],
        "electric" => &[0xd0d060, 0xf0f0f0],
        "haunted" => &[0xd06060, 0x8a3030],
        "derelict" => &[0x8a7040, 0x6a5030],
        "still" => &[0xf0f0f0, 0x40a0d8],
        _ => &[0x40a0d8, 0xf0f0f0, 0xd0d060, 0xd06060],
    }
}

// Wall mirror: 20% chance per level, 1 framed mirror on a wall (batch r13a).
pub(super) fn place_wall_mirrors(g: &mut Game, _dist: &[i32]) {
    g.pp.bathroom_bundle_positions.clear(); // reset per level
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let key = g.weather_key();
    // A domestic wall mirror, 42 x 60 cm (Blender batch `r13a`): 4 mm
    // silvered glass in a moulded frame, a stained hardwood one or a gilt
    // one, hung flat on four brass mirror plates. The glass gives back the
    // room's light (`Patch::Glass`), its silver going black from the edges;
    // dust on it, now and then hands' smears, now and then struck and
    // cracked, its splinters on the floor below (docs/PROPS.md)
    use game::assets::props::prop_part_as_modelled;
    // 30% chance smudged; 10% chance cracked (the old overlays' draws)
    let smudged = rnd() < 0.30;
    let mirror_cracked = rnd() < 0.10;
    let mirror = group();
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let height_draw = rnd();
    let mirror_height = 1.55 + height_draw * 0.25; // eye-level range
    if c.dx.abs() == 1 {
        mirror.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        mirror.position().set(wall_x, mirror_height, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        mirror.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        mirror.position().set(wall_x + (rnd() - 0.5) * 1.2, mirror_height, wall_z);
    }
    {
        let p = mirror.pos();
        let vary = wm_vary(p.x, p.z, height_draw);
        // which frame: a gilt one about one in three, from its height's draw
        let gilt = (height_draw * 7919.0).fract() < 0.35;
        // the parts are modelled with the wall's face at z = 0: 8 mm behind the spot
        let body = group();
        body.position().z = -0.008;
        mirror.add(&body);
        let frame_name = if gilt { "Real__wallMirrorFrameGilt" } else { "Real__wallMirrorFrameWood" };
        if let Some(geo) = prop_part_as_modelled(frame_name) {
            let mat = if gilt {
                phong!({ map: &wm_gilt_tex(vary), specular: 0x3a3020, shininess: 24 })
            } else {
                lambert!({ map: &moulding_grain_tex(WM_SIZE[0], WM_SIZE[1], WM_MOULD, [0x56, 0x3a, 0x26], vary, 900.0) })
            };
            body.add(&mesh(geo, &mat));
        }
        if let Some(geo) = prop_part_as_modelled("Real__wallMirrorPlates") {
            body.add(&mesh(geo, phong!({ color: 0x5e4c2c, specular: 0x4a3c24, shininess: 36 })));
        }
        let brk = if mirror_cracked { WmBreak::struck(WM_GLASS[0], WM_GLASS[1], u64::from(vary) << 9 | 0x3b) } else { WmBreak::default() };
        // the black paint on the glass's back, seen where the silver has gone
        let back = mesh(plane_geo!(WM_GLASS[0] + 0.01, WM_GLASS[1] + 0.01), lambert!({ color: 0x141312 }));
        back.position().z = 0.0050;
        body.add(&back);
        let glass = mesh(plane_geo!(WM_GLASS[0] + WM_UNDER, WM_GLASS[1] + WM_UNDER), wm_glass_mat(wm_silver_tex(&brk, vary)));
        glass.position().z = WM_GLASS[2] - 0.0001;
        glass.set_render_order(2);
        body.add(&glass);
        let film_mat = lambert!({
            map: &wm_film_tex(&brk, smudged, gilt, vary), transparent: true, depthWrite: false,
            polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2,
        });
        let film = mesh(plane_geo!(WM_GLASS[0], WM_GLASS[1]), &film_mat);
        film.position().z = WM_GLASS[2] + 0.0003;
        film.set_render_order(3);
        body.add(&film);
    }
    g.world.add(&mirror);
    // 2% chance: mirror imperceptibly sways.
    if rnd() < 0.02 {
        let base_rot_z = mirror.rot().z;
        let sway_phase = rnd() * PI * 2.0;
        let m2 = mirror.clone();
        g.animate(move |_g, _dt, t| {
            m2.rotation().z = base_rot_z + (t * 0.3 * PI + sway_phase).sin() * 0.02;
        });
    }
    let mp = mirror.pos();
    let m_rot_y = mirror.rot().y;
    // If mirror is cracked, 3-6 splinters of it on the floor below: the
    // silvered face up or its painted back, in front of the wall
    if mirror_cracked {
        let face_up = phong!({ color: 0x1c1e1e, specular: 0x8a8c8c, shininess: 300 });
        let face_down = lambert!({ color: 0x2c2a28 });
        let (along, out) = ((m_rot_y.cos(), -m_rot_y.sin()), (m_rot_y.sin(), m_rot_y.cos()));
        let shard_count = 3 + (rnd() * 4.0).floor() as i32;
        for _sh in 0..shard_count {
            let sw = 0.015 + rnd() * 0.02;
            let shh = 0.020 + rnd() * 0.02;
            let turn = rnd() * PI * 2.0;
            let sh_ang = rnd() * PI * 2.0;
            let sh_rad = 0.10 + rnd() * 0.35;
            let up = (turn * 7.0).fract() < 0.6;
            let Some(geo) = prop_part_as_modelled("Real__glassShard") else { continue };
            let shard = mesh(geo, if up { &face_up } else { &face_down });
            let k = ((sw + shh) / 0.05).clamp(0.85, 1.15);
            shard.scale().set(k, k, k);
            shard.rotation().y = turn;
            let (a, n) = (sh_ang.cos() * sh_rad, 0.03 + sh_ang.sin().abs() * sh_rad);
            shard.position().set(mp.x + along.0 * a + out.0 * n, 0.001, mp.z + along.1 * a + out.1 * n);
            g.world.add(&shard);
        }
    }
    // 25% chance: small wall clock adjacent to the mirror — bathroom-vestibule bundle.
    if rnd() < 0.25 {
        let clocks = clock_textures();
        let clock_tex = clocks[(rnd() * clocks.len() as f64).floor() as usize].clone();
        let clock_mat = lambert!({ map: clock_tex, transparent: true, side: DoubleSide });
        // the school clock's black steel case and domed glass (batch r04)
        // round its printed dial, a little smaller (32 cm), its back on the wall
        let adj_clock = group();
        {
            const K: f64 = 0.944;
            let body = group();
            body.scale().set(K, K, K);
            body.position().z = -0.008;
            adj_clock.add(&body);
            if let Some(geo) = prop_part_as_modelled("Real__clockCase") {
                body.add(&mesh(geo, phong!({ color: 0x1b1b1c, specular: 0x242424, shininess: 30 })));
            }
            let dial = mesh(circle_geo!(0.1499, 48), &clock_mat);
            dial.position().z = 0.0301;
            body.add(&dial);
            if let Some(geo) = prop_part_as_modelled("Real__clockGlass") {
                body.add(&mesh(
                    geo,
                    phong!({ color: 0x9a9a96, transparent: true, opacity: 0.18, depthWrite: false, specular: 0x4a4a4a, shininess: 80 }),
                ));
            }
        }
        let adj_offset = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.60 + rnd() * 0.40);
        let adj_height = 1.90 + rnd() * 0.30;
        adj_clock.rotation().y = m_rot_y;
        if c.dx.abs() == 1 {
            adj_clock.position().set(wall_x, adj_height, mp.z + adj_offset);
        } else {
            adj_clock.position().set(mp.x + adj_offset, adj_height, wall_z);
        }
        // Slight ±0.03 rad Z tilt for bundle clock too.
        adj_clock.rotation().z = (rnd() - 0.5) * 0.06;
        g.world.add(&adj_clock);
        let cp = adj_clock.pos();
        // Track for tick audio events (as the wall clocks are).
        g.pp.wall_clock_positions.push(v3(cp.x, adj_height, cp.z));
        // Track bathroom-bundle center for rare faucet-drip audio.
        let bundle_cx = (mp.x + cp.x) * 0.5;
        let bundle_cz = (mp.z + cp.z) * 0.5;
        g.pp.bathroom_bundle_positions.push(v3(bundle_cx, 1.0, bundle_cz));
        // The basin under the mirror (batch r13a: a wall-hung vitreous china
        // basin, its rim 0.895 m up, where the soap and the toothbrush lie;
        // its chrome waste and bottle trap). The first draw (the old dark bar
        // of a "sink outline", 60%) now says whether its taps are still on
        // it at all; the second (50% of those) whether they are: else the deck
        // shows the two holes they were taken from
        let sink_placed = rnd() < 0.60;
        let basin = group();
        basin.rotation().y = m_rot_y;
        if c.dx.abs() == 1 {
            basin.position().set(wall_x, 0.0, mp.z);
        } else {
            basin.position().set(mp.x, 0.0, wall_z);
        }
        let glaze = phong!({ map: &wm_basin_tex(wm_vary(mp.x, mp.z, 0.75), false), specular: 0x3c3c3a, shininess: 60 });
        let bowl = phong!({ map: &wm_basin_tex(wm_vary(mp.x, mp.z, 0.5), true), specular: 0x3c3c3a, shininess: 60 });
        let chrome = phong!({ color: 0x55585a, specular: 0x8a8c8e, shininess: 70 });
        for (name, mat) in [("Real__wallMirrorBasin", &glaze), ("Real__wallMirrorBasinBowl", &bowl), ("Real__wallMirrorBasinTrap", &chrome)]
        {
            if let Some(geo) = prop_part_as_modelled(name) {
                let m = mesh(geo, mat);
                m.position().z = -0.008;
                basin.add(&m);
            }
        }
        if sink_placed && rnd() < 0.50 {
            if let Some(geo) = prop_part_as_modelled("Real__wallMirrorBasinTaps") {
                // dull with limescale
                let m = mesh(geo, phong!({ color: 0x626462, specular: 0x6a6c6c, shininess: 50 }));
                m.position().z = -0.008;
                basin.add(&m);
            }
        }
        g.world.add(&basin);
        // 30% chance: small soap dispenser on the opposite side from the clock.
        let mut dispenser_placed = false;
        let mut dispenser_offset = 0.0;
        if rnd() < 0.30 {
            let dispenser_mat = lambert!({ color: 0x4a4038 });
            let nozzle_mat = lambert!({ color: 0x181410 });
            let dispenser = group();
            let d_body = mesh(pg("WallMirrors__dispenser", cyl_geo!(0.025, 0.025, 0.16, 6)), &dispenser_mat);
            dispenser.add(&d_body);
            let nozzle = mesh(pg("WallMirrors__nozzle", cyl_geo!(0.006, 0.006, 0.02, 4)), &nozzle_mat);
            nozzle.rotation().z = PI / 2.0;
            nozzle.position().set(0.020, -0.075, 0);
            dispenser.add(&nozzle);
            let d_offset = -sign_of(adj_offset) * (0.30 + rnd() * 0.10);
            let d_height = 1.15 + rnd() * 0.10;
            dispenser.rotation().y = m_rot_y;
            if c.dx.abs() == 1 {
                dispenser.position().set(wall_x - c.dx as f64 * 0.028, d_height, mp.z + d_offset);
            } else {
                dispenser.position().set(mp.x + d_offset, d_height, wall_z - c.dz as f64 * 0.028);
            }
            g.world.add(&dispenser);
            dispenser_placed = true;
            dispenser_offset = d_offset;
        }
        // 30% chance (if dispenser present): small soap bar on the sink shelf.
        if dispenser_placed && rnd() < 0.30 {
            let bar_pool = soap_bar_color_by_weather(key);
            let bar_mat = lambert!({ color: pick(bar_pool) });
            let soap_bar = mesh(pg("WallMirrors__soap", box_geo!(0.06, 0.020, 0.035)), &bar_mat);
            soap_bar.rotation().y = m_rot_y + (rnd() - 0.5) * 0.4;
            // on the basin's deck, at its end under the dispenser
            let bar_pos_offset = (dispenser_offset + (rnd() - 0.5) * 0.08).clamp(-0.205, 0.205);
            if c.dx.abs() == 1 {
                soap_bar.position().set(wall_x - c.dx as f64 * 0.05, 0.905, mp.z + bar_pos_offset);
            } else {
                soap_bar.position().set(mp.x + bar_pos_offset, 0.905, wall_z - c.dz as f64 * 0.05);
            }
            g.world.add(&soap_bar);
        }
        // 40% chance: small dark drip stain on the floor at bundle center.
        if rnd() < 0.40 {
            // (lit, soft and ragged: an unlit disc had glowed in the dark)
            let stain_mat = lambert!({
                map: &wm_stain_tex(wm_vary(bundle_cx, bundle_cz, 0.25)), transparent: true, depthWrite: false,
                polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2,
            });
            let stain = mesh(circle_geo!(0.08 + rnd() * 0.07, 24), &stain_mat);
            stain.rotation().x = -PI / 2.0;
            stain.rotation().z = rnd() * PI * 2.0;
            stain.scale().set(1.3, 0.9, 1);
            let nudge_x = -(c.dx as f64) * 0.20;
            let nudge_z = -(c.dz as f64) * 0.20;
            stain.position().set(bundle_cx + nudge_x, 0.004, bundle_cz + nudge_z);
            g.world.add(&stain);
        }
        // 40% chance: RESTROOM wayfinding sign above the mirror (window sign #3).
        if rnd() < 0.40 {
            let sign_mat = lambert!({ map: window_sign_textures()[3].clone(), transparent: true, side: DoubleSide });
            let restroom_sign = mesh(plane_geo!(0.22, 0.11), &sign_mat);
            restroom_sign.rotation().y = m_rot_y;
            let sign_height = 2.20 + rnd() * 0.15; // above adj_clock's height
            let sign_offset = -sign_of(adj_offset) * (0.20 + rnd() * 0.25);
            if c.dx.abs() == 1 {
                restroom_sign.position().set(wall_x, sign_height, mp.z + sign_offset);
            } else {
                restroom_sign.position().set(mp.x + sign_offset, sign_height, wall_z);
            }
            g.world.add(&restroom_sign);
        }
        // 25% chance: tiny toothbrush laid on the sink shelf area.
        if rnd() < 0.25 {
            let brush_pool = toothbrush_color_by_weather(key);
            let brush_mat = lambert!({ color: pick(brush_pool) });
            let bristle_mat = lambert!({ color: 0xe8e0d0 });
            let brush = group();
            let handle = mesh(pg("WallMirrors__brush", cyl_geo!(0.004, 0.004, 0.10, 5)), &brush_mat);
            handle.rotation().z = PI / 2.0;
            brush.add(&handle);
            let bristles = mesh(pg("WallMirrors__bristles", box_geo!(0.012, 0.006, 0.008)), &bristle_mat);
            bristles.position().set(0.056, 0, 0);
            brush.add(&bristles);
            brush.rotation().y = m_rot_y + (rnd() - 0.5) * 0.4;
            let br_offset = (rnd() - 0.5) * 0.30;
            if c.dx.abs() == 1 {
                brush.position().set(wall_x - c.dx as f64 * 0.05, 0.905, mp.z + br_offset);
            } else {
                brush.position().set(mp.x + br_offset, 0.905, wall_z - c.dz as f64 * 0.05);
            }
            g.world.add(&brush);
        }
    }
}

/// Something of where a thing was put, to vary what its painters paint
/// (a painter's own stream starts the same on every floor)
fn wm_vary(x: f64, z: f64, k: f64) -> u32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [x, z, k] {
        h ^= v.to_bits();
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h ^ (h >> 29) ^ (h >> 47)) as u32
}

/// A mirror's breakage, in metres on its glass (x right and y up from its
/// middle): each crack's points and weight (1 a fine one), where it was
/// struck and how wide the crushed spot is. From a seed (no level draws), so
/// the silver and the film agree
#[derive(Clone, Default)]
struct WmBreak {
    lines: Vec<(Vec<[f64; 2]>, f64)>,
    hits: Vec<([f64; 2], f64)>,
    /// the radial cracks in turn round the blow
    radials: Vec<Vec<[f64; 2]>>,
}

impl WmBreak {
    /// struck once: radial cracks from the blow to the edges (some stopping
    /// short, a few branching), rings between them near it
    fn struck(gw: f64, gh: f64, seed: u64) -> WmBreak {
        let (hw, hh) = (gw / 2.0, gh / 2.0);
        let mut r = SplitMix64::new(0, seed);
        let mut b = WmBreak::default();
        let inside = |p: [f64; 2]| p[0].abs() < hw + 0.003 && p[1].abs() < hh + 0.003;
        let run = |r: &mut SplitMix64, p: [f64; 2], a: f64, reach: f64| -> Vec<[f64; 2]> {
            let mut pts = vec![p];
            let (mut q, mut a, mut l) = (p, a, 0.0);
            while l < reach && inside(q) {
                let step = r.range(0.008, 0.022);
                a += r.range(-0.12, 0.12);
                q = [q[0] + a.cos() * step, q[1] + a.sin() * step];
                l += step;
                pts.push(q);
            }
            pts
        };
        // the blow: chest high, a little to one side
        let p = [r.range(-0.28, 0.28) * gw, r.range(-0.1, 0.3) * gh];
        let crush = r.range(0.005, 0.01);
        b.hits.push((p, crush));
        let n = 7 + (r.next() * 5.0) as usize;
        let a0 = r.range(0.0, PI * 2.0);
        let mut dirs: Vec<Vec<[f64; 2]>> = vec![];
        for i in 0..n {
            let a = a0 + PI * 2.0 * i as f64 / n as f64 + r.range(-0.25, 0.25);
            let start = [p[0] + a.cos() * crush, p[1] + a.sin() * crush];
            let reach = if r.next() < 0.25 { r.range(0.05, 0.16) } else { 9.0 };
            let pts = run(&mut r, start, a, reach);
            if r.next() < 0.4 && pts.len() > 5 {
                let k = pts.len() * (3 + (r.next() * 3.0) as usize) / 10;
                let side = if r.next() < 0.5 { -1.0 } else { 1.0 };
                let turn = a + side * r.range(0.3, 0.6);
                let reach = if r.next() < 0.4 { r.range(0.04, 0.12) } else { 9.0 };
                let br = run(&mut r, pts[k], turn, reach);
                b.lines.push((br, 0.8));
            }
            dirs.push(pts.clone());
            b.lines.push((pts, 1.25));
        }
        let at = |pts: &Vec<[f64; 2]>, rad: f64| pts.iter().find(|q| (q[0] - p[0]).hypot(q[1] - p[1]) >= rad).copied();
        let rings = [r.range(0.018, 0.035), r.range(0.05, 0.09)];
        for (ri, &rad) in rings.iter().enumerate() {
            let chance = if ri == 0 { 0.55 } else { 0.3 };
            for i in 0..n {
                let j = (i + 1) % n;
                if r.next() > chance {
                    continue;
                }
                let r0 = rad * r.range(0.75, 1.3);
                let r1 = r0 * r.range(0.8, 1.2);
                let (Some(q0), Some(q1)) = (at(&dirs[i], r0), at(&dirs[j], r1)) else { continue };
                let m = [(q0[0] + q1[0]) / 2.0 + r.range(-0.002, 0.002), (q0[1] + q1[1]) / 2.0 + r.range(-0.002, 0.002)];
                b.lines.push((vec![q0, m, q1], 0.75));
            }
        }
        b.radials = dirs;
        b
    }
}

/// A soft blob, squashed whole (`k`) and turned, so its fade is never cut
fn wm_blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, stops: &[(f64, String)]) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(turn);
    ctx.scale(1.0, k);
    let g = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    for (at, c) in stops {
        g.add_color_stop(*at, c);
    }
    ctx.set_fill_style(&g);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// The glass: silver behind it (`Patch::Glass`, nearly all of the light
/// back), its map where the silver has gone to the black paint behind
fn wm_glass_mat(silver: Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x2a2a2a, shininess: 4000, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(silver);
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "mirror_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it (the light where it hangs, off what it faces), nothing behind it
        f[0] = 0.13;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // the map's grey is the silver gone, to the black paint behind (linear)
        f[8] = 1.0;
        f[9] = 0.016;
        f[10] = 0.015;
        f[11] = 0.013;
        f[12] = 0.0;
        // silvered: most of the light comes back at any angle
        f[13] = 0.86;
    }
    m
}

/// Where the silver has gone (grey: how much, to the black paint behind):
/// a black edge creeping in from the frame in stretches with a wandering,
/// feathered front (deepest along the foot and in the corners), spots of it
/// ahead, pinholes; the silver flaked along the cracks and round the blow
fn wm_silver_tex(b: &WmBreak, vary: u32) -> Tex {
    let b = b.clone();
    canvas_tex_opt(256, 384, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let (gw, gh) = (WM_GLASS[0] + WM_UNDER, WM_GLASS[1] + WM_UNDER);
            let px = |p: [f64; 2]| ((p[0] / gw + 0.5) * w, (0.5 - p[1] / gh) * h);
            ctx.set_fill_style("#000");
            ctx.fill_rect(0, 0, w, h);
            // the room in it, darker low down
            let gr = ctx.create_linear_gradient(0, 0, 0, h);
            gr.add_color_stop(0, "rgba(255,255,255,0)");
            gr.add_color_stop(0.4, "rgba(255,255,255,0.05)");
            gr.add_color_stop(1, "rgba(255,255,255,0.26)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(0, 0, w, h);
            let spot = |ctx: &mut Canvas, x: f64, y: f64, r: f64, a: f64| {
                let stops = [
                    (0.0, format!("rgba(255,255,255,{a:.3})")),
                    (0.5, format!("rgba(255,255,255,{:.3})", a * 0.75)),
                    (1.0, "rgba(255,255,255,0)".into()),
                ];
                wm_blob(ctx, x, y, r, 0.55 + 0.45 * rnd(), rnd() * 3.0, &stops);
            };
            // the black edge along each side (how deep at most in px, the side's length)
            let sides: [(f64, f64); 4] = [(13.0, w), (6.0, h), (6.0, h), (4.0, w)]; // foot, left, right, top
            for (si, &(deep, len)) in sides.iter().enumerate() {
                let ph = [rnd() * 6.3, rnd() * 6.3, rnd() * 6.3];
                let fr = [2.0 + rnd() * 2.0, 6.0 + rnd() * 5.0, 15.0 + rnd() * 10.0];
                let depth = |t: f64| -> f64 {
                    let wave = 0.5
                        + 0.22 * (fr[0] * t * 6.28 + ph[0]).sin()
                        + 0.16 * (fr[1] * t * 6.28 + ph[1]).sin()
                        + 0.1 * (fr[2] * t * 6.28 + ph[2]).sin();
                    let on = ((wave - 0.38) / 0.4).clamp(0.0, 1.0);
                    let corner = 1.0 + 1.6 * ((1.0 - (t * 2.0 - 1.0).abs()) * -9.0).exp();
                    deep * (on * on * (3.0 - 2.0 * on)) * wave * corner
                };
                let at = |t: f64, d: f64| -> (f64, f64) {
                    match si {
                        0 => (t * w, h - d),
                        1 => (d, h - t * h),
                        2 => (w - d, h - t * h),
                        _ => (t * w, d),
                    }
                };
                ctx.set_fill_style("rgba(255,255,255,0.85)");
                ctx.begin_path();
                let (x0, y0) = at(0.0, 0.0);
                ctx.move_to(x0, y0);
                let n = (len / 3.0) as usize;
                for k in 0..=n {
                    let t = k as f64 / n as f64;
                    let (x, y) = at(t, depth(t));
                    ctx.line_to(x, y);
                }
                let (x1, y1) = at(1.0, 0.0);
                ctx.line_to(x1, y1);
                ctx.close_path();
                ctx.fill();
                for _ in 0..(len / 2.5) as usize {
                    let t = rnd();
                    let d = depth(t);
                    if d < 0.5 {
                        continue;
                    }
                    let (x, y) = at(t, d * (0.8 + 0.4 * rnd()));
                    spot(ctx, x, y, 1.0 + rnd() * d.min(10.0) * 0.45, 0.3 + 0.4 * rnd());
                }
                for _ in 0..(len / 6.0) as usize {
                    let t = rnd();
                    let d = depth(t).max(1.5) * (1.3 + 2.5 * rnd().powi(2));
                    let (x, y) = at(t, d);
                    spot(ctx, x, y, 0.6 + rnd() * rnd() * 3.0, 0.4 + 0.5 * rnd());
                }
            }
            // pinholes, all over but most low down
            for _ in 0..60 {
                let (x, y) = (rnd() * w, h * rnd().powf(0.6));
                ctx.set_fill_style(format!("rgba(255,255,255,{:.2})", 0.5 + 0.5 * rnd()));
                ctx.begin_path();
                ctx.arc(x, y, 0.35 + rnd() * rnd() * 1.1, 0, 7);
                ctx.fill();
            }
            // along the cracks the silver flaked; round the blow more of it
            ctx.set_line_cap("round");
            ctx.set_line_join("round");
            for (pts, k) in &b.lines {
                ctx.set_stroke_style("rgba(255,255,255,0.55)");
                ctx.set_line_width(0.9 * k);
                ctx.begin_path();
                for (i, p) in pts.iter().enumerate() {
                    let (x, y) = px(*p);
                    if i == 0 {
                        ctx.move_to(x, y);
                    } else {
                        ctx.line_to(x, y);
                    }
                }
                ctx.stroke();
            }
            for (p, cr) in &b.hits {
                let (x, y) = px(*p);
                let rr = cr / gw * w;
                for _ in 0..10 {
                    let (a, d) = (rnd() * 7.0, rr * (1.0 + 2.5 * rnd()));
                    spot(ctx, x + a.cos() * d, y + a.sin() * d, 1.0 + rnd() * rr * 0.8, 0.4 + 0.4 * rnd());
                }
            }
        })
    })
}

/// The mirror's face: a film of dust, a wiped smear and finger marks, the
/// frame's wood seen again round its edge, and the breakage (each crack a
/// dark line where the silver broke and a bright one where its edge catches
/// the light, the crushed spot of the blow); clear elsewhere
fn wm_film_tex(b: &WmBreak, smudged: bool, gilt: bool, vary: u32) -> Tex {
    let b = b.clone();
    canvas_tex_opt(320, 484, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let (gw, gh) = (WM_GLASS[0], WM_GLASS[1]);
            let px = |p: [f64; 2]| ((p[0] / gw + 0.5) * w, (0.5 - p[1] / gh) * h);
            ctx.clear_rect(0, 0, w, h);
            // the moulding seen again in the glass, the glass's thickness a dark line between
            ctx.set_fill_style(if gilt { "rgba(128,104,58,0.5)" } else { "rgba(74,50,34,0.5)" });
            for (x, y, rw, rh) in [(0.0, 0.0, w, 3.0), (0.0, h - 3.0, w, 3.0), (0.0, 0.0, 3.0, h), (w - 3.0, 0.0, 3.0, h)] {
                ctx.fill_rect(x, y, rw, rh);
            }
            ctx.set_stroke_style("rgba(18,16,14,0.45)");
            ctx.set_line_width(1.1);
            ctx.stroke_rect(3.5, 3.5, w - 7.0, h - 7.0);
            // dust, unevenly, and more of it lower down
            for _ in 0..14 {
                let (x, y, r) = (rnd() * w, h * rnd().powf(0.7), w * (0.12 + 0.3 * rnd()));
                let g = ctx.create_radial_gradient(x, y, 0, x, y, r);
                g.add_color_stop(0, &format!("rgba(176,172,162,{:.3})", 0.05 + 0.06 * rnd()));
                g.add_color_stop(1, "rgba(176,172,162,0)");
                ctx.set_fill_style(&g);
                ctx.fill_rect(0, 0, w, h);
            }
            // the last cleaning's swirls, faint; smudged: hands' smears
            // over them, a palm where someone leaned, fingers down its side
            ctx.set_line_cap("round");
            for _ in 0..if smudged { 5 } else { 2 } {
                let (x, y, r) = (w * (0.2 + 0.6 * rnd()), h * (0.3 + 0.5 * rnd()), w * (0.1 + 0.2 * rnd()));
                ctx.set_stroke_style(format!("rgba(186,182,172,{:.3})", if smudged { 0.04 + 0.04 * rnd() } else { 0.015 + 0.02 * rnd() }));
                ctx.set_line_width(8.0 + 12.0 * rnd());
                ctx.begin_path();
                let a = rnd() * 6.3;
                ctx.arc(x, y, r, a, a + 0.8 + rnd() * 1.4);
                ctx.stroke();
            }
            if smudged {
                for _ in 0..2 {
                    let side = rnd() < 0.5;
                    let x0 = if side { w * (0.05 + 0.08 * rnd()) } else { w * (0.87 - 0.08 * rnd()) };
                    let y0 = h * (0.25 + 0.4 * rnd());
                    let tilt = (rnd() - 0.5) * 0.6;
                    for f in 0..4 {
                        let stops = [
                            (0.0, "rgba(198,192,180,0.2)".to_string()),
                            (0.7, "rgba(198,192,180,0.1)".into()),
                            (1.0, "rgba(198,192,180,0)".into()),
                        ];
                        let (fx, fy) = (x0 + tilt * f as f64 * 11.0 + (rnd() - 0.5) * 3.0, y0 + f as f64 * 11.0);
                        wm_blob(ctx, fx, fy, 5.0 + rnd() * 2.0, 0.65, 1.4 + tilt, &stops);
                    }
                }
                let (px0, py0) = (w * (0.3 + 0.4 * rnd()), h * (0.45 + 0.25 * rnd()));
                let stops = [
                    (0.0, "rgba(198,192,180,0.12)".to_string()),
                    (0.6, "rgba(198,192,180,0.07)".into()),
                    (1.0, "rgba(198,192,180,0)".into()),
                ];
                wm_blob(ctx, px0, py0, 26.0 + 8.0 * rnd(), 0.8, rnd() * 3.0, &stops);
            }
            // the cracks: dark where the silver broke, bright where the edge catches the light
            ctx.set_line_join("round");
            let path = |ctx: &mut Canvas, pts: &[[f64; 2]], dx: f64, dy: f64| {
                ctx.begin_path();
                for (i, p) in pts.iter().enumerate() {
                    let (x, y) = px(*p);
                    if i == 0 {
                        ctx.move_to(x + dx, y + dy);
                    } else {
                        ctx.line_to(x + dx, y + dy);
                    }
                }
                ctx.stroke();
            };
            // each piece between the radials sits a hair off the others and
            // gives back the room a little differently
            let nr = b.radials.len();
            for i in 0..nr {
                let (ra, rb) = (&b.radials[i], &b.radials[(i + 1) % nr]);
                let v = if rnd() < 0.5 { "rgba(220,218,210," } else { "rgba(10,10,9," };
                ctx.set_fill_style(format!("{v}{:.3})", 0.02 + 0.035 * rnd()));
                ctx.begin_path();
                for (k, q) in ra.iter().chain(rb.iter().rev()).enumerate() {
                    let (x, y) = px(*q);
                    if k == 0 {
                        ctx.move_to(x, y);
                    } else {
                        ctx.line_to(x, y);
                    }
                }
                ctx.close_path();
                ctx.fill();
            }
            for (pts, k) in &b.lines {
                ctx.set_stroke_style("rgba(12,11,10,0.7)");
                ctx.set_line_width(1.2 * k);
                path(ctx, pts, 0.0, 0.0);
                ctx.set_stroke_style(format!("rgba(226,224,216,{:.2})", 0.28 + 0.22 * rnd()));
                ctx.set_line_width(0.6 * k);
                path(ctx, pts, 0.6, -0.5);
            }
            // the crushed spot: glass ground to white, splinters out of it
            for (p, cr) in &b.hits {
                let (x, y) = px(*p);
                let rr = cr / gw * w;
                ctx.set_fill_style("rgba(214,212,204,0.85)");
                ctx.begin_path();
                for i in 0..14 {
                    let a = i as f64 / 14.0 * PI * 2.0;
                    let q = rr * (0.6 + 0.6 * rnd());
                    if i == 0 {
                        ctx.move_to(x + a.cos() * q, y + a.sin() * q);
                    } else {
                        ctx.line_to(x + a.cos() * q, y + a.sin() * q);
                    }
                }
                ctx.close_path();
                ctx.fill();
                for _ in 0..28 {
                    let a = rnd() * 7.0;
                    let (r0, r1) = (rr * rnd(), rr * (1.0 + 1.6 * rnd()));
                    let v = if rnd() < 0.5 { 22 } else { 226 };
                    ctx.set_stroke_style(format!("rgba({v},{v},{},0.7)", v - 6));
                    ctx.set_line_width(0.6 + 0.4 * rnd());
                    ctx.begin_path();
                    ctx.move_to(x + a.cos() * r0, y + a.sin() * r0);
                    ctx.line_to(x + a.cos() * r1, y + a.sin() * r1);
                    ctx.stroke();
                }
            }
        })
    })
}

/// A moulded frame's wood as its front is mapped (the frame's bounds, `w` x
/// `h` m, its rails `mould` m wide, `ppm` px a metre): each rail's grain
/// along it, mitred at the corners, the joints opened a hair; stained `base`
/// (sRGB), its figure darker, the stain rubbed thin along its outer edge and
/// its sight edge, knocks and scratches, dust along the top rail. `vary`
/// mixes the frame's place into its stream
fn moulding_grain_tex(w: f64, h: f64, mould: f64, base: [u8; 3], vary: u32, ppm: f64) -> Tex {
    let (pw, ph) = ((w * ppm).round(), (h * ppm).round());
    canvas_tex_opt(pw, ph, false, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let m = mould * ppm;
            let [r0, g0, b0] = base.map(f64::from);
            let col = |k: f64, a: f64| format!("rgba({:.0},{:.0},{:.0},{a:.3})", r0 * k, g0 * k, b0 * k);
            ctx.set_fill_style(col(1.0, 1.0));
            ctx.fill_rect(0, 0, cw, ch);
            // the four rails: (outline, along x?)
            let rails: [([(f64, f64); 4], bool); 4] = [
                ([(0.0, 0.0), (cw, 0.0), (cw - m, m), (m, m)], true),
                ([(0.0, ch), (cw, ch), (cw - m, ch - m), (m, ch - m)], true),
                ([(0.0, 0.0), (m, m), (m, ch - m), (0.0, ch)], false),
                ([(cw, 0.0), (cw - m, m), (cw - m, ch - m), (cw, ch)], false),
            ];
            for (poly, along_x) in rails.iter() {
                ctx.save();
                ctx.begin_path();
                ctx.move_to(poly[0].0, poly[0].1);
                for q in &poly[1..] {
                    ctx.line_to(q.0, q.1);
                }
                ctx.close_path();
                ctx.clip();
                // each rail its own board: a shade of its own
                ctx.set_fill_style(col(0.88 + 0.24 * rnd(), 0.5));
                ctx.fill_rect(0, 0, cw, ch);
                let len = if *along_x { cw } else { ch };
                let (lo, hi) = match (poly[0], along_x) {
                    ((_, y), true) if y < 1.0 => (0.0, m),
                    (_, true) => (ch - m, ch),
                    ((x, _), false) if x < 1.0 => (0.0, m),
                    _ => (cw - m, cw),
                };
                // the grain: long wavering lines along the rail, darker figure among them
                let lines = (m * 2.2) as usize;
                for _ in 0..lines {
                    let across = lo + rnd() * (hi - lo);
                    let (amp, fr, ph) = (0.4 + rnd() * 1.6, 1.0 + rnd() * 3.0, rnd() * 6.3);
                    let dark = rnd() < 0.3;
                    ctx.set_stroke_style(col(if dark { 0.55 } else { 0.72 + 0.2 * rnd() }, if dark { 0.35 } else { 0.18 + 0.2 * rnd() }));
                    ctx.set_line_width(if dark { 0.8 + rnd() * 1.4 } else { 0.4 + rnd() * 0.8 });
                    ctx.begin_path();
                    let n = (len / 8.0) as usize + 2;
                    for k in 0..=n {
                        let t = k as f64 / n as f64 * len;
                        let a = across + amp * (t / len * fr * 6.28 + ph).sin();
                        let (x, y) = if *along_x { (t, a) } else { (a, t) };
                        if k == 0 {
                            ctx.move_to(x, y);
                        } else {
                            ctx.line_to(x, y);
                        }
                    }
                    ctx.stroke();
                }
                // pores, short dark flecks along the grain
                for _ in 0..(len * m / 60.0) as usize {
                    let (t, a) = (rnd() * len, lo + rnd() * (hi - lo));
                    let l = 1.5 + rnd() * 4.0;
                    ctx.set_stroke_style(col(0.45, 0.25 + 0.2 * rnd()));
                    ctx.set_line_width(0.6);
                    ctx.begin_path();
                    let (x, y, dx, dy) = if *along_x { (t, a, l, 0.0) } else { (a, t, 0.0, l) };
                    ctx.move_to(x, y);
                    ctx.line_to(x + dx, y + dy);
                    ctx.stroke();
                }
                ctx.restore();
            }
            // the mitres: each joint opened a hair, grime in it
            ctx.set_stroke_style("rgba(20,14,10,0.55)");
            ctx.set_line_width(0.9);
            for (a, b) in [((0.0, 0.0), (m, m)), ((cw, 0.0), (cw - m, m)), ((0.0, ch), (m, ch - m)), ((cw, ch), (cw - m, ch - m))] {
                ctx.begin_path();
                ctx.move_to(a.0, a.1);
                ctx.line_to(b.0, b.1);
                ctx.stroke();
            }
            // the stain rubbed thin along the outer edge and the sight edge
            ctx.set_line_width(2.4);
            for inset in [1.2, m - 1.4] {
                ctx.set_stroke_style("rgba(222,196,160,0.14)");
                ctx.stroke_rect(inset, inset, cw - 2.0 * inset, ch - 2.0 * inset);
            }
            // knocks and scratches
            for _ in 0..(cw + ch) as usize / 14 {
                let edge = rnd() < 0.7;
                let (x, y) = if rnd() < 0.5 {
                    (
                        rnd() * cw,
                        if rnd() < 0.5 {
                            if edge {
                                1.0
                            } else {
                                rnd() * m
                            }
                        } else if edge {
                            ch - 1.0
                        } else {
                            ch - rnd() * m
                        },
                    )
                } else {
                    (
                        if rnd() < 0.5 {
                            if edge {
                                1.0
                            } else {
                                rnd() * m
                            }
                        } else if edge {
                            cw - 1.0
                        } else {
                            cw - rnd() * m
                        },
                        rnd() * ch,
                    )
                };
                if rnd() < 0.5 {
                    let stops = [(0.0, "rgba(214,182,140,0.45)".to_string()), (1.0, "rgba(214,182,140,0)".into())];
                    wm_blob(ctx, x, y, 0.8 + rnd() * 2.2, 0.5 + 0.5 * rnd(), rnd() * 3.0, &stops);
                } else {
                    let a = rnd() * 6.3;
                    let l = 3.0 + rnd() * 12.0;
                    ctx.set_stroke_style(format!("rgba(200,170,130,{:.2})", 0.12 + 0.18 * rnd()));
                    ctx.set_line_width(0.5);
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(x + a.cos() * l, y + a.sin() * l);
                    ctx.stroke();
                }
            }
            // dust along the top rail, and grime where hands set it straight
            let gd = ctx.create_linear_gradient(0, 0, 0, m);
            gd.add_color_stop(0, "rgba(168,162,150,0.32)");
            gd.add_color_stop(1, "rgba(168,162,150,0.08)");
            ctx.set_fill_style(&gd);
            ctx.fill_rect(m * 0.4, 0, cw - m * 0.8, m);
            for x in [m * 0.6, cw - m * 0.6] {
                let stops = [(0.0, "rgba(28,20,14,0.22)".to_string()), (1.0, "rgba(28,20,14,0)".into())];
                wm_blob(ctx, x, ch * (0.55 + 0.2 * rnd()), m * 1.4, 0.6, PI / 2.0, &stops);
            }
        })
    })
}

/// The wall mirror: its size, its moulding's width, its glass as seen in
/// the frame (w, h) and its face's distance from the wall
/// (`Real__wallMirrorFrameWood` / `Gilt`, r13a.py), and how much more glass
/// runs under the lip (both sides)
const WM_SIZE: [f64; 2] = [0.420, 0.600];

const WM_MOULD: f64 = 0.034;

const WM_GLASS: [f64; 3] = [0.352, 0.532, 0.0090];

const WM_UNDER: f64 = 0.012;

/// The gilt frame as its front is mapped (its bounds): old gold leaf on red
/// bole over gesso, dulled, burnished on the beads and the cushion's crest
/// and rubbed through there to the bole, chipped to the white gesso along
/// its outer edge, grime in its quirks, dust on its top. Each pixel from
/// where on the moulding's profile it lies (its distance in from the outer
/// edge), no canvas call in the loop
fn wm_gilt_tex(vary: u32) -> Tex {
    let ppm = 900.0;
    canvas_tex_opt((WM_SIZE[0] * ppm).round(), (WM_SIZE[1] * ppm).round(), false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        let (n, m) = (w as usize, h as usize);
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = (j as f64 + 0.5) / ppm;
            for i in 0..n {
                let x = (i as f64 + 0.5) / ppm;
                let d = x.min(WM_SIZE[0] - x).min(y).min(WM_SIZE[1] - y);
                let bump = |c: f64, s: f64| (-((d - c) / s).powi(2)).exp();
                // the beads and the cushion's crest, the quirks between
                let crest = bump(0.0057, 0.0016).max(bump(0.0200, 0.0042)).max(bump(0.0303, 0.0016));
                let recess = bump(0.0086, 0.0009).max(bump(0.0277, 0.0009)).max(bump(0.0340, 0.0008));
                let (u, v) = (x * 900.0, y * 900.0);
                let mot = fbm(seed ^ 0x11, u * 0.05, v * 0.05, 3);
                let gold = [146.0, 116.0, 62.0];
                let mut c = [0.0; 3];
                for k in 0..3 {
                    c[k] = gold[k] * (0.82 + 0.3 * crest + 0.16 * (mot - 0.5));
                }
                // rubbed through to the red bole on the high places
                let rub = crest * sstep(0.56, 0.7, fbm(seed ^ 0x12, u * 0.09, v * 0.09, 3));
                let bole = [118.0, 62.0, 40.0];
                // chipped to the gesso along the outer edge, a knock here and there
                let chip = sstep(0.0035, 0.0015, d) * sstep(0.62, 0.72, fbm(seed ^ 0x13, u * 0.12, v * 0.12, 2))
                    + sstep(0.86, 0.9, vnoise(seed ^ 0x14, u * 0.06, v * 0.06)) * crest;
                let gesso = [204.0, 194.0, 168.0];
                // grime in the quirks and over it all in patches; dust on the top rail
                let grime = (recess * 0.75 + 0.25 * sstep(0.5, 0.8, fbm(seed ^ 0x15, u * 0.02, v * 0.02, 2))).min(0.9);
                let dust = if y < WM_MOULD { 0.22 + 0.1 * (1.0 - crest) } else { 0.04 } * (0.6 + 0.4 * mot);
                for k in 0..3 {
                    let mut t = c[k] + (bole[k] - c[k]) * rub;
                    t += (gesso[k] - t) * chip.min(1.0);
                    t *= 1.0 - 0.55 * grime;
                    t += ([150.0, 146.0, 136.0][k] - t) * dust;
                    out[i * 4 + k] = t.round().clamp(0.0, 255.0) as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        let _ = m;
        ctx.put_image_data(&img, 0, 0);
    })
}

/// The basin's glaze as its top is mapped (x across, z out from the wall:
/// the body 0.5 x 0.4 m, or the bowl's inside, 0.38 x 0.27 m from 0.105 m
/// out): off-white vitreous china gone grey, its glaze crazed, grime in
/// patches; in the bowl the rust-brown runs where the taps dripped for years
/// down to the plug, and scale round the plug; dust on the deck
fn wm_basin_tex(vary: u32, bowl: bool) -> Tex {
    let (cw, chh) = if bowl { (196.0, 140.0) } else { (256.0, 205.0) };
    canvas_tex_opt(cw, chh, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let px =
                |x: f64, z: f64| if bowl { ((x + 0.19) / 0.38 * w, (z - 0.105) / 0.27 * h) } else { ((x + 0.25) / 0.5 * w, z / 0.4 * h) };
            ctx.set_fill_style("#d6d1c4");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..14 {
                let (x, y, r) = (rnd() * w, rnd() * h, 20.0 + rnd() * 60.0);
                let stops = [(0.0, format!("rgba(120,112,96,{:.3})", 0.05 + 0.08 * rnd())), (1.0, "rgba(120,112,96,0)".into())];
                wm_blob(ctx, x, y, r, 0.6 + 0.4 * rnd(), rnd() * 3.0, &stops);
            }
            // crazing: a fine network in the glaze
            ctx.set_line_width(0.5);
            for _ in 0..60 {
                let (mut x, mut y) = (rnd() * w, rnd() * h);
                ctx.set_stroke_style(format!("rgba(90,84,74,{:.2})", 0.1 + 0.15 * rnd()));
                ctx.begin_path();
                ctx.move_to(x, y);
                let mut a = rnd() * 6.3;
                for _ in 0..4 {
                    a += (rnd() - 0.5) * 1.6;
                    x += a.cos() * (4.0 + rnd() * 8.0);
                    y += a.sin() * (4.0 + rnd() * 8.0);
                    ctx.line_to(x, y);
                }
                ctx.stroke();
            }
            if !bowl {
                let gd = ctx.create_linear_gradient(0, 0, 0, h * 0.3);
                gd.add_color_stop(0, "rgba(150,146,134,0.35)");
                gd.add_color_stop(1, "rgba(150,146,134,0)");
                ctx.set_fill_style(&gd);
                ctx.fill_rect(0, 0, w, h * 0.3);
                return;
            }
            // the brown the dripping left, pooled toward the plug; short runs under the taps
            let (pxp, pyp) = px(0.0, 0.25);
            for _ in 0..6 {
                let (x, y) = (pxp + (rnd() - 0.5) * w * 0.25, pyp + (rnd() - 0.5) * h * 0.3);
                let stops = [(0.0, format!("rgba(122,82,46,{:.3})", 0.1 + 0.08 * rnd())), (1.0, "rgba(122,82,46,0)".into())];
                wm_blob(ctx, x, y, w * (0.1 + 0.12 * rnd()), 0.7 + 0.3 * rnd(), rnd() * 3.0, &stops);
            }
            ctx.set_line_cap("round");
            for sx in [-1.0, 1.0] {
                let (x0, _) = px(sx * 0.08, 0.105);
                let g = ctx.create_linear_gradient(0, 0, 0, h * 0.3);
                g.add_color_stop(0, "rgba(118,72,38,0.32)");
                g.add_color_stop(1, "rgba(118,72,38,0)");
                ctx.set_stroke_style(&g);
                ctx.set_line_width(5.0 + 3.0 * rnd());
                ctx.begin_path();
                ctx.move_to(x0, 0);
                ctx.line_to(x0 - sx * 4.0, h * 0.3);
                ctx.stroke();
            }
            let stops =
                [(0.0, "rgba(176,164,138,0.0)".to_string()), (0.5, "rgba(176,164,138,0.45)".into()), (1.0, "rgba(176,164,138,0)".into())];
            wm_blob(ctx, pxp, pyp, 18.0, 1.0, 0.0, &stops);
        })
    })
}

/// A drip's stain dried on the floor: a soft, ragged, darker patch, its
/// tide line a little darker, clear round it (transparent)
fn wm_stain_tex(vary: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            ctx.clear_rect(0, 0, w, h);
            for _ in 0..9 {
                let (x, y) = (w * (0.35 + 0.3 * rnd()), h * (0.35 + 0.3 * rnd()));
                let r = w * (0.12 + 0.16 * rnd());
                let a = 0.12 + 0.12 * rnd();
                let stops = [
                    (0.0, format!("rgba(26,28,30,{a:.3})")),
                    (0.75, format!("rgba(26,28,30,{:.3})", a * 0.8)),
                    (1.0, "rgba(26,28,30,0)".into()),
                ];
                wm_blob(ctx, x, y, r, 0.6 + 0.4 * rnd(), rnd() * 3.0, &stops);
            }
            ctx.set_stroke_style("rgba(22,22,22,0.12)");
            ctx.set_line_width(1.6);
            ctx.begin_path();
            let r0 = w * 0.34;
            for k in 0..=40 {
                let a = k as f64 / 40.0 * PI * 2.0;
                let rr = r0 * (0.85 + 0.15 * (a * 3.0 + rnd() * 0.4).sin());
                let (x, y) = (w / 2.0 + a.cos() * rr, h / 2.0 + a.sin() * rr * 0.8);
                if k == 0 {
                    ctx.move_to(x, y);
                } else {
                    ctx.line_to(x, y);
                }
            }
            ctx.stroke();
        })
    })
}
