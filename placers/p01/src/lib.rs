//! Prop placers ported from backrooms.js (chunk p01; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

pub mod consts;
use consts::*;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeScrapPapers", place_scrap_papers));
    r.push(("placeChalkMarks", place_chalk_marks));
    r.push(("placeSmallKeys", place_small_keys));
    r.push(("placeVintageCameras", place_vintage_cameras));
    r.push(("placeBackpacks", place_backpacks));
    r.push(("placeSmallGiftBoxes", place_small_gift_boxes));
    r.push(("placeSmallPartyHorns", place_small_party_horns));
    r.push(("placeSmallScales", place_small_scales));
    r.push(("placeSmallAbacuses", place_small_abacuses));
    r.push(("placeSmallTelegraphKeys", place_small_telegraph_keys));
    r.push(("placeSmallSewingMachines", place_small_sewing_machines));
    r.push(("placeHangingBrassLanterns", place_hanging_brass_lanterns));
    r.push(("placeSmallSuitcases", place_small_suitcases));
    r.push(("placeSmallPhonographs", place_small_phonographs));
    r.push(("placeSmallRotaryPhones", place_small_rotary_phones));
    r.push(("placeSmallTypewriters", place_small_typewriters));
    r.push(("placeSmallGlobes", place_small_globes));
    r.push(("placeSmallTelescopes", place_small_telescopes));
    r.push(("placeSmallPadlockKeys", place_small_padlock_keys));
    r.push(("placeParchmentScrolls", place_parchment_scrolls));
    r.push(("placeSmallCompasses", place_small_compasses));
    r.push(("placeSmallPocketWatches", place_small_pocket_watches));
}

// ------------------------------------------------------------------ helpers
/// Every open interior tile, row by row (the `openTiles` loop every placer repeats).
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); }
        }
    }
    v
}
/// In-place Fisher-Yates exactly as the JS loops do it.
fn shuffle<T>(v: &mut [T]) {
    for i in (1..v.len()).rev() {
        let j = (rnd() * (i + 1) as f64).floor() as usize;
        v.swap(i, j);
    }
}
/// `arr[Math.floor(Math.random() * arr.length)]`
fn pick<T: Copy>(v: &[T]) -> T { v[(rnd() * v.len() as f64).floor() as usize] }

/// A wall-facing slot: open tile (x, z) with a wall at (x + dx, z + dz).
#[derive(Clone, Copy)]
struct WallCand { x: i32, z: i32, dx: i32, dz: i32 }

/// Every (open tile, wall direction) pair: `!inGrid(nx, nz) || tiles[...] === 1`.
fn wall_candidates(g: &Game) -> Vec<WallCand> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 { continue; }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if g.is_wall_tile(nx, nz) { v.push(WallCand { x, z, dx, dz }); }
            }
        }
    }
    v
}

/// `a.clone().sub(b)`
fn vsub(a: Vec3, b: Vec3) -> Vec3 { let mut v = a; v.sub(&b); v }

// ------------------------------------------------------------------ line 5798
// R246 discarded scrap papers: 1-3 small crumpled paper meshes per level.
fn place_scrap_papers(g: &mut Game, _dist: &[i32]) {
    let count = 1 + (rnd() * 3.0).floor() as usize;   // 1-3 scraps
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let paper_mat = lambert!({ color: 0xd0c8b4 });
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        // Irregular crumple shape: small box scaled non-uniformly.
        let size_x = 0.06 + rnd() * 0.05;
        let size_y = 0.04 + rnd() * 0.03;
        let size_z = 0.06 + rnd() * 0.05;
        let geo = pg("ScrapPapers__wad", box_geo!(size_x, size_y, size_z));
        let scrap = mesh(geo, &paper_mat);
        // Rest on floor with random rotation.
        let rx = (rnd() - 0.5) * 0.4;
        let ry = rnd() * PI * 2.0;
        let rz = (rnd() - 0.5) * 0.4;
        scrap.rotation().set(rx, ry, rz);
        let ox = (rnd() - 0.5) * 2.6;
        let oz = (rnd() - 0.5) * 2.6;
        scrap.position().set(g.wx(x) + ox, size_y * 0.5, g.wz(z) + oz);
        g.world.add(&scrap);
        i += 1;
    }
}

// ------------------------------------------------------------------ line 5835
// R246 chalk marks: 0-3 small white streak decals on random walls.
fn place_chalk_marks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.35 { return; }   // 35% chance of NO chalk this level
    let count = 1 + (rnd() * 3.0).floor() as usize;   // 1-3 marks
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let chalk_color = chalk_color_by_weather(g.weather_key());
    let chalk_mat = basic_mat!({ color: chalk_color, transparent: true, opacity: 0.75, depthWrite: false, side: DoubleSide });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // Short vertical tally: 0.015 wide x 0.10-0.16 tall.
        let w = 0.012 + rnd() * 0.006;
        let h = 0.10 + rnd() * 0.06;
        let mark = mesh(plane_geo!(w, h), &chalk_mat);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
        let mid_height = 0.6 + rnd() * 0.8;   // 0.6 - 1.4 m
        if c.dx.abs() == 1 {
            mark.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            mark.position().set(wall_x, mid_height, wall_z + (rnd() - 0.5) * 1.4);
        } else {
            mark.rotation().y = if c.dz > 0 { PI } else { 0.0 };
            mark.position().set(wall_x + (rnd() - 0.5) * 1.4, mid_height, wall_z);
        }
        g.world.add(&mark);
        i += 1;
    }
}

// ------------------------------------------------------------------ line 5911
// R290 small key prop: 25% chance per level, 1-3 tiny gold keys on the floor.
fn place_small_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 { return; }   // 25% chance
    let count = 1 + (rnd() * 3.0).floor() as usize;
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let key_color = key_color_by_weather(g.weather_key());
    let key_mat = lambert!({ color: key_color });
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        let key = group();
        // Ring at head of key.
        let ring = mesh(pg("SmallKeys__bow", torus_geo!(0.014, 0.003, 4, 10)), &key_mat);
        ring.rotation().x = PI / 2.0;
        ring.position().x = -0.015;
        key.add(&ring);
        // Shaft.
        let shaft = mesh(pg("SmallKeys__blade", cyl_geo!(0.003, 0.003, 0.028, 5)), &key_mat);
        shaft.rotation().z = PI / 2.0;
        shaft.position().x = 0.012;
        key.add(&shaft);
        // Tooth (tiny bump on shaft end): bitting in the key's horizontal plane.
        let tooth = mesh(pg("SmallKeys__bitting", box_geo!(0.008, 0.006, 0.003)), &key_mat);
        tooth.rotation().x = -PI / 2.0;
        tooth.position().set(0.019, 0, 0.005);
        key.add(&tooth);
        let ox = (rnd() - 0.5) * 2.6;
        let oz = (rnd() - 0.5) * 2.6;
        key.position().set(g.wx(x) + ox, 0.003, g.wz(z) + oz);
        key.rotation().y = rnd() * PI * 2.0;
        g.world.add(&key);
        i += 1;
    }
}

// ------------------------------------------------------------------ line 5957
// R407 vintage camera: 10% chance per level.
fn place_vintage_cameras(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 { return; }   // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let body_mat = lambert!({ color: 0x181410 });
    let lens_mat = lambert!({ color: 0x2a2418 });
    let glass_mat = lambert!({ color: 0x1a2028, transparent: true, opacity: 0.65 });
    let cam = group();
    // Body: small box.
    let body = mesh(pg("VintageCameras__body", box_geo!(0.10, 0.06, 0.05)), &body_mat);
    body.position().y = 0.03;
    cam.add(&body);
    // Lens: cylinder protruding front.
    let lens = mesh(pg("VintageCameras__lens", cyl_geo!(0.020, 0.020, 0.03, 10)), &lens_mat);
    lens.rotation().x = PI / 2.0;
    lens.position().set(0, 0.03, 0.035);
    cam.add(&lens);
    // Glass front on lens.
    let glass = mesh(circle_geo!(0.018, 10), &glass_mat);
    glass.position().set(0, 0.03, 0.051);
    cam.add(&glass);
    // Viewfinder: small dark rectangle bump on top.
    let vf = mesh(pg("VintageCameras__finder", box_geo!(0.02, 0.015, 0.015)), &body_mat);
    vf.position().set(-0.03, 0.068, 0);
    cam.add(&vf);
    cam.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cam.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cam);
    // R408 25% chance: small film canister cylinder near camera.
    if rnd() < 0.25 {
        let can_colors = [0x8a2020, 0x282838, 0x2a2820, 0x505050];
        let can_body = lambert!({ color: pick(&can_colors) });
        let can_cap = lambert!({ color: 0x282020 });
        let canister = group();
        let can_b = mesh(pg("VintageCameras__can", cyl_geo!(0.018, 0.018, 0.06, 8)), &can_body);
        can_b.position().y = 0.030;
        canister.add(&can_b);
        let c_cap = mesh(pg("VintageCameras__cap", cyl_geo!(0.019, 0.019, 0.008, 8)), &can_cap);
        c_cap.position().y = 0.064;
        canister.add(&c_cap);
        let c_ang = rnd() * PI * 2.0;
        let c_rad = 0.08 + rnd() * 0.12;
        let cp = cam.pos();
        canister.position().set(cp.x + c_ang.cos() * c_rad, 0, cp.z + c_ang.sin() * c_rad);
        canister.rotation().y = rnd() * PI * 2.0;
        g.world.add(&canister);
    }
}

// ------------------------------------------------------------------ line 6025
// R411 small backpack: 12% chance per level.
fn place_backpacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 { return; }   // 12% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let body_colors = [0x203040, 0x2a1810, 0x4a3018, 0x484028, 0x381818];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let strap_mat = lambert!({ color: 0x181410 });
    let bp = group();
    // Body: upright box.
    let body = mesh(pg("Backpacks__body", box_geo!(0.28, 0.36, 0.16)), &body_mat);
    body.position().y = 0.18;
    bp.add(&body);
    // 2 vertical shoulder straps on back.
    for sx_off in [-0.08, 0.08] {
        let strap = mesh(pg("Backpacks__strap", plane_geo!(0.04, 0.35)), &strap_mat);
        strap.rotation().y = PI;   // face out of the back (it faced into the body)
        strap.position().set(sx_off, 0.18, -0.081);
        bp.add(&strap);
    }
    // Top handle: half-torus.
    let handle = mesh(pg("Backpacks__handle", torus_geo!(0.04, 0.006, 4, 8, PI)), &strap_mat);
    handle.rotation().z = 0.0;   // loop over the top, ends on it (was a sideways C)
    handle.position().y = 0.36;
    bp.add(&handle);
    // Small front pocket flap.
    let pocket = mesh(pg("Backpacks__pocket", plane_geo!(0.16, 0.10)), &strap_mat);
    pocket.position().set(0, 0.08, 0.081);
    bp.add(&pocket);
    bp.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    bp.rotation().y = rnd() * PI * 2.0;
    // Slight tilt for organic slump.
    bp.rotation().z = (rnd() - 0.5) * 0.10;
    g.world.add(&bp);
}

// ------------------------------------------------------------------ line 6074
// R427 small gift box: 10% chance per level.
fn place_small_gift_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 { return; }   // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let box_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xd050a0, 0xa020a0];
    let ribbon_colors = [0xf0e0d0, 0xd0d0d0, 0xd8b820, 0x2050d8];
    let box_mat = lambert!({ color: pick(&box_colors) });
    let ribbon_mat = lambert!({ color: pick(&ribbon_colors) });
    let gift = group();
    let bs = 0.10 + rnd() * 0.04;   // 10-14 cm cube
    // Body: cube.
    let body = mesh(pg("SmallGiftBoxes__body", box_geo!(bs, bs, bs)), &box_mat);
    body.position().y = bs / 2.0;
    gift.add(&body);
    // 2 ribbon strips crossing top + wrapping sides.
    let r_wide = 0.010;
    let r_top1 = mesh(pg("SmallGiftBoxes__ribbonX", box_geo!(bs + 0.002, 0.003, r_wide)), &ribbon_mat);
    r_top1.position().y = bs + 0.001;
    gift.add(&r_top1);
    let r_top2 = mesh(pg("SmallGiftBoxes__ribbonZ", box_geo!(r_wide, 0.003, bs + 0.002)), &ribbon_mat);
    r_top2.position().y = bs + 0.001;
    gift.add(&r_top2);
    // Side ribbon wraps (front + back + left + right).
    for (rw, rh, rp, px, py, pz) in [
        (bs + 0.002, bs, r_wide, 0.0, 0.0, bs / 2.0 + 0.001),
        (bs + 0.002, bs, r_wide, 0.0, 0.0, -bs / 2.0 - 0.001),
        (r_wide, bs, bs + 0.002, bs / 2.0 + 0.001, 0.0, 0.0),
        (r_wide, bs, bs + 0.002, -bs / 2.0 - 0.001, 0.0, 0.0),
    ] {
        let r_side = mesh(box_geo!(rw, rh, if rw == r_wide { rp } else { rw }), &ribbon_mat);
        // Scale the box to a 1 cm x 2 mm strip down the middle of its face,
        // continuing the top ribbon, and add it (it was never added).
        let gd = if rw == r_wide { rp } else { rw };
        r_side.scale().set(
            if px == 0.0 { r_wide / rw } else { 0.002 / rw },
            1,
            if px == 0.0 { 0.002 / gd } else { r_wide / gd },
        );
        r_side.position().set(px, bs / 2.0 + py, pz);
        gift.add(&r_side);
    }
    // Bow: 2 small triangles at ribbon crossing.
    for bow_side in [-1.0f64, 1.0] {
        let bow_geo = Geo::new();
        let bow_verts: Vec<f32> = [
            0.0, bs + 0.005, 0.0,
            bow_side * 0.025, bs + 0.020, 0.0,
            bow_side * 0.020, bs + 0.005, bow_side * 0.010,
        ].iter().map(|&v| v as f32).collect();
        bow_geo.set_position(bow_verts);
        bow_geo.set_index(vec![0, 1, 2]);
        bow_geo.compute_vertex_normals();
        let bow = mesh(bow_geo, &ribbon_mat);
        gift.add(&bow);
    }
    gift.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    gift.rotation().y = rnd() * PI * 2.0;
    g.world.add(&gift);
}

// ------------------------------------------------------------------ line 6146
// R427 small party horn: 8% chance per level.
fn place_small_party_horns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let horn_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0];
    let horn_mat = lambert!({ color: pick(&horn_colors), side: DoubleSide });
    let mouth_mat = lambert!({ color: 0x2a2418 });
    let horn = group();
    // Body: long tapered cone laid horizontal.
    let body = mesh(cone_geo!(0.015, 0.20, 6), &horn_mat);
    body.rotation().z = -PI / 2.0;
    body.position().set(0.10, 0.015, 0);
    horn.add(&body);
    // Mouthpiece: small dark disc at wide end.
    let mouth = mesh(pg("SmallPartyHorns__mouth", cyl_geo!(0.014, 0.014, 0.005, 6)), &mouth_mat);
    mouth.rotation().z = PI / 2.0;
    mouth.position().set(0.005, 0.015, 0);
    horn.add(&mouth);
    horn.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    horn.rotation().y = rnd() * PI * 2.0;
    g.world.add(&horn);
}

// ------------------------------------------------------------------ line 6184
// R478 small vintage brass scales: 5% chance per level, gentle sway.
fn place_small_scales(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; }   // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let scales = group();
    // Base disc.
    let base = mesh(pg("SmallScales__base", cyl_geo!(0.045, 0.055, 0.010, 10)), &brass_mat);
    base.position().y = 0.005;
    scales.add(&base);
    // Central pillar.
    let pillar = mesh(pg("SmallScales__pillar", cyl_geo!(0.005, 0.005, 0.10, 6)), &brass_mat);
    pillar.position().y = 0.060;
    scales.add(&pillar);
    // Cross-beam group (rotates as a whole).
    let beam = group();
    let beam_bar = mesh(pg("SmallScales__beam", cyl_geo!(0.003, 0.003, 0.14, 4)), &brass_mat);
    beam_bar.rotation().z = PI / 2.0;
    beam.add(&beam_bar);
    // 2 chains hanging + 2 pans.
    for s in [-1.0f64, 1.0] {
        for ci in 0..3 {
            let chain = mesh(pg("SmallScales__chain", cyl_geo!(0.001, 0.001, 0.030, 3)), &brass_mat);
            // One straight run from inside the beam end down into the pan.
            chain.position().set(s * 0.065, -0.013 - ci as f64 * 0.030, 0);
            beam.add(&chain);
        }
        let pan = mesh(pg("SmallScales__pan", cyl_geo!(0.024, 0.026, 0.004, 10)), &brass_mat);
        // Hung 1.1 cm higher: the chains reach it and it clears the base.
        pan.position().set(s * 0.065, -0.087, 0);
        beam.add(&pan);
    }
    beam.position().y = 0.112;
    scales.add(&beam);
    // Very slight balance sway.
    let sway_phase = rnd() * PI * 2.0;
    let sway_period = 6.0 + rnd() * 4.0;
    let beam2 = beam.clone();
    g.animate(move |_g, _dt, t| {
        beam2.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.05;
    });
    scales.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    scales.rotation().y = rnd() * PI * 2.0;
    g.world.add(&scales);
}

// ------------------------------------------------------------------ line 6242
// R478 small vintage abacus: 6% chance per level.
fn place_small_abacuses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; }   // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let frame_mat = lambert!({ color: 0x8a5030 });
    let rod_mat = lambert!({ color: 0xc89040 });
    let bead_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030];
    let abacus = group();
    let (w, h, t) = (0.15, 0.12, 0.008);
    // Frame (4 boxes making rectangle border).
    for (fw, fh, fx, fy) in [
        (w, 0.012, 0.0, h / 2.0 - 0.006),
        (w, 0.012, 0.0, -h / 2.0 + 0.006),
        (0.012, h - 0.024, -w / 2.0 + 0.006, 0.0),
        (0.012, h - 0.024, w / 2.0 - 0.006, 0.0),
    ] {
        let bar = mesh(pg("SmallAbacuses__bar", box_geo!(fw, fh, t)), &frame_mat);
        bar.position().set(fx, 0.060 + fy, 0);
        abacus.add(&bar);
    }
    // 4 rod rows with beads.
    let n_rows = 4;
    let n_beads = 10;
    for row in 0..n_rows {
        let row_y = 0.060 - h / 2.0 + 0.020 + row as f64 * 0.020;
        // Rod.
        let rod = mesh(pg("SmallAbacuses__rod", cyl_geo!(0.0015, 0.0015, w - 0.020, 4)), &rod_mat);
        rod.rotation().z = PI / 2.0;
        rod.position().set(0, row_y, 0);
        abacus.add(&rod);
        // Beads.
        let bead_mat = lambert!({ color: bead_colors[row % bead_colors.len()] });
        for b in 0..n_beads {
            let bead = mesh(pg("SmallAbacuses__bead", cyl_geo!(0.007, 0.007, 0.010, 8)), &bead_mat);
            bead.rotation().z = PI / 2.0;
            // End beads stop 0.5 mm short of the stiles' inner faces (±0.063).
            bead.position().set(-w / 2.0 + 0.0175 + b as f64 * ((w - 0.035) / (n_beads - 1) as f64), row_y, 0);
            abacus.add(&bead);
        }
    }
    abacus.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    abacus.rotation().y = rnd() * PI * 2.0;
    g.world.add(&abacus);
}

// ------------------------------------------------------------------ line 6301
// R477 small telegraph key: 8% chance per level.
fn place_small_telegraph_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let wood_mat = lambert!({ color: 0x603820 });
    let brass_mat = lambert!({ color: 0xc89040 });
    let knob_mat = lambert!({ color: 0x181008 });
    let key = group();
    // Base wood plate.
    let base = mesh(pg("SmallTelegraphKeys__base", box_geo!(0.08, 0.010, 0.06)), &wood_mat);
    base.position().y = 0.005;
    key.add(&base);
    // Brass pivot posts.
    for s in [-1.0f64, 1.0] {
        let post = mesh(pg("SmallTelegraphKeys__post", cyl_geo!(0.004, 0.004, 0.020, 5)), &brass_mat);
        post.position().set(0, 0.020, s * 0.010);
        key.add(&post);
    }
    // Key lever arm.
    let lever = mesh(pg("SmallTelegraphKeys__lever", box_geo!(0.06, 0.004, 0.008)), &brass_mat);
    lever.position().set(0.008, 0.024, 0);
    key.add(&lever);
    // Bakelite knob on top.
    let knob = mesh(pg("SmallTelegraphKeys__knob", sphere_geo!(0.010, 8, 6)), &knob_mat);
    knob.scale().y = 0.5;
    knob.position().set(0.028, 0.028, 0);
    key.add(&knob);
    // 2 brass terminals on base.
    for s in [-1.0f64, 1.0] {
        let term = mesh(pg("SmallTelegraphKeys__terminal", cyl_geo!(0.004, 0.004, 0.008, 5)), &brass_mat);
        term.position().set(-0.030, 0.014, s * 0.020);
        key.add(&term);
    }
    // Wire trailing off: from between the terminals, over the base's end and
    // down onto the floor.
    let wire = mesh(pg("SmallTelegraphKeys__wire", cyl_geo!(0.001, 0.001, 0.10, 3)), &knob_mat);
    let wire_a = v3(-0.030, 0.012, 0);
    let wire_b = v3(-0.126, 0.001, 0.012);
    let dir = vsub(wire_b, wire_a).normalized();
    wire.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &dir);
    wire.scale().y = wire_a.distance_to(&wire_b) / 0.10;
    {
        let mut p = wire.position();
        p.copy(&wire_a).add(&wire_b).multiply_scalar(0.5);
    }
    key.add(&wire);
    key.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    key.rotation().y = rnd() * PI * 2.0;
    g.world.add(&key);
}

// ------------------------------------------------------------------ line 6360
// R477 small vintage sewing machine: 4% chance per level.
fn place_small_sewing_machines(g: &mut Game, _dist: &[i32]) {
    g.pp.sewing_positions.clear();   // R481 reset per level
    if rnd() < 0.96 { return; }   // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let iron_mat = lambert!({ color: 0x181008 });
    let gold_mat = lambert!({ color: 0xd8b040 });
    let needle_mat = lambert!({ color: 0xc0c0c0 });
    let sewing = group();
    // Base plate.
    let base = mesh(pg("SmallSewingMachines__bed", box_geo!(0.20, 0.014, 0.08)), &iron_mat);
    base.position().y = 0.007;
    sewing.add(&base);
    // Body: L-shape (main mass + tall column).
    let body = mesh(pg("SmallSewingMachines__pillar", box_geo!(0.06, 0.10, 0.06)), &iron_mat);
    body.position().set(-0.06, 0.064, 0);
    sewing.add(&body);
    // Top arm (extends right from body).
    let arm = mesh(pg("SmallSewingMachines__arm", box_geo!(0.16, 0.03, 0.05)), &iron_mat);
    arm.position().set(0.02, 0.128, 0);
    sewing.add(&arm);
    // Needle head (below arm right end).
    let needle_head = mesh(pg("SmallSewingMachines__head", box_geo!(0.020, 0.025, 0.035)), &iron_mat);
    needle_head.position().set(0.086, 0.104, 0);
    sewing.add(&needle_head);
    // Needle.
    let needle = mesh(pg("SmallSewingMachines__needle", cyl_geo!(0.001, 0.001, 0.020, 3)), &needle_mat);
    needle.position().set(0.086, 0.082, 0);
    sewing.add(&needle);
    // Hand wheel on right side of body.
    let wheel = mesh(pg("SmallSewingMachines__wheel", cyl_geo!(0.030, 0.030, 0.006, 12)), &iron_mat);
    wheel.rotation().z = PI / 2.0;
    wheel.set_rotation_order(EulerOrder::ZYX);   // the animator's rotation.y then spins it on its own axle
    wheel.position().set(-0.093, 0.100, 0);   // on the column's outer end face (x -0.09)
    sewing.add(&wheel);
    // Wheel spokes (thin lines - use small boxes crossing).
    for i in 0..4 {
        let spoke = mesh(box_geo!(0.001, 0.056, 0.002), &gold_mat);
        spoke.rotation().x = (i as f64 / 4.0) * PI;
        spoke.position().set(-0.0965, 0.100, 0);
        sewing.add(&spoke);
    }
    // Gold decorative curves (small dots on body).
    for i in 0..3 {
        let decor = mesh(circle_geo!(0.005, 6), &gold_mat);
        decor.position().set(-0.06, 0.030 + i as f64 * 0.020, 0.031);
        sewing.add(&decor);
    }
    // Wheel slow rotation (the JS spoke loop is an empty no-op).
    let rot_rate = 0.3 + rnd() * 0.3;
    let wheel2 = wheel.clone();
    g.animate(move |_g, _dt, t| {
        wheel2.rotation().y = t * rot_rate;
    });
    sewing.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    sewing.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sewing);
    // R481 track for whirr audio.
    let sp = sewing.pos();
    g.pp.sewing_positions.push(v3(sp.x, 0.10, sp.z));
}

// ------------------------------------------------------------------ line 6435
// R476 hanging brass lantern: 4% chance per level, hanging from the ceiling.
fn place_hanging_brass_lanterns(g: &mut Game, _dist: &[i32]) {
    g.pp.brass_lantern_positions.clear();   // R480 reset per level
    if rnd() < 0.96 { return; }   // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let glass_mat = basic_mat!({ color: 0xffe0a0, transparent: true, opacity: 0.35 });
    let chain_mat = lambert!({ color: 0x8a7050 });
    let lantern = group();
    let chain_len = 0.5 + rnd() * 0.4;
    let lantern_y = WALL_H - chain_len - 0.10;
    // Chain from ceiling: stretched 2.2 cm so it runs from the ceiling down
    // into the top cap's boss.
    let chain = mesh(pg("HangingBrassLanterns__chain", cyl_geo!(0.003, 0.003, chain_len, 4)), &chain_mat);
    chain.scale().y = (chain_len + 0.022) / chain_len;
    chain.position().y = WALL_H + 0.002 - (chain_len + 0.022) / 2.0;
    lantern.add(&chain);
    // Top cap.
    let top_cap = mesh(pg("HangingBrassLanterns__topCap", cyl_geo!(0.030, 0.040, 0.015, 6)), &brass_mat);
    top_cap.position().y = lantern_y + 0.085;
    lantern.add(&top_cap);
    // 4 corner posts (thin cylinders).
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0 + PI / 4.0;
        let post = mesh(pg("HangingBrassLanterns__post", cyl_geo!(0.003, 0.003, 0.14, 4)), &brass_mat);
        post.position().set(a.cos() * 0.028, lantern_y, a.sin() * 0.028);
        lantern.add(&post);
    }
    // 4 glass panels between posts (sides of the square whose corners are the posts).
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0;
        let panel = mesh(plane_geo!(0.038, 0.14), &glass_mat);
        panel.position().set(a.cos() * 0.028 * std::f64::consts::FRAC_1_SQRT_2, lantern_y, a.sin() * 0.028 * std::f64::consts::FRAC_1_SQRT_2);
        panel.rotation().y = -a + PI / 2.0;
        lantern.add(&panel);
    }
    // Bottom cap.
    let bot_cap = mesh(pg("HangingBrassLanterns__botCap", cyl_geo!(0.040, 0.030, 0.015, 6)), &brass_mat);
    bot_cap.position().y = lantern_y - 0.085;
    lantern.add(&bot_cap);
    // Inner light + candle flame.
    let flame_mat = basic_mat!({ color: 0xffd070, transparent: true, opacity: 0.9 });
    let flame = mesh(cone_geo!(0.010, 0.030, 6), &flame_mat);
    flame.position().y = lantern_y;
    lantern.add(&flame);
    let glow = point_light!(0xffb060, 0.7, 1.5, 2);
    glow.position().y = lantern_y;
    lantern.add(&glow);
    // R484 warm halo sphere inside lantern glass.
    let halo_mat = basic_mat!({ color: 0xffc070, transparent: true, opacity: 0.35, depthWrite: false });
    let halo = mesh(sphere_geo!(0.036, 8, 6), &halo_mat);
    halo.position().y = lantern_y;
    lantern.add(&halo);
    // Flicker.
    let flicker_phase = rnd() * PI * 2.0;
    let flicker_rate = 5.0 + rnd() * 3.0;
    {
        let (glow, flame, halo) = (glow.clone(), flame.clone(), halo.clone());
        g.animate(move |_g, _dt, t| {
            let p = t * flicker_rate + flicker_phase;
            let wobble = p.sin() * 0.5 + (p * 1.7).sin() * 0.3;
            glow.set_intensity(0.7 + wobble * 0.15);
            flame.scale().y = 1.0 + wobble * 0.10;
            halo.scale().set_scalar(1.0 + wobble * 0.10);
        });
    }
    lantern.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    lantern.rotation().y = rnd() * PI * 2.0;
    // Slow sway.
    let is_swaying = rnd() < 0.60;
    if is_swaying {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 8.0 + rnd() * 6.0;
        let lantern2 = lantern.clone();
        g.animate(move |_g, _dt, t| {
            lantern2.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.03;
        });
    }
    g.world.add(&lantern);
    // R480 track swaying lanterns for creak audio.
    if is_swaying {
        let lp = lantern.pos();
        g.pp.brass_lantern_positions.push(v3(lp.x, lantern_y, lp.z));
    }
}

// ------------------------------------------------------------------ line 6538
// R476 small vintage suitcase: 8% chance per level.
fn place_small_suitcases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let leather_colors = [0x603820, 0x502010, 0x8a5030, 0x201a10];
    let leather_mat = lambert!({ color: pick(&leather_colors) });
    let brass_mat = lambert!({ color: 0xc89040 });
    let suitcase = group();
    // Body.
    let body = mesh(pg("SmallSuitcases__body", box_geo!(0.22, 0.10, 0.14)), &leather_mat);
    body.position().y = 0.05;
    suitcase.add(&body);
    // Middle divider line (open-close seam).
    let seam = mesh(pg("SmallSuitcases__seam", box_geo!(0.222, 0.003, 0.142)), &brass_mat);
    seam.position().y = 0.05;
    suitcase.add(&seam);
    // Handle (torus arc on top).
    let handle = mesh(pg("SmallSuitcases__handle", torus_geo!(0.024, 0.005, 4, 12, PI)), &leather_mat);
    handle.rotation().x = PI / 2.0;
    handle.position().y = 0.104;
    suitcase.add(&handle);
    // 2 brass buckles on front.
    for s in [-1.0f64, 1.0] {
        let buckle = mesh(pg("SmallSuitcases__buckle", box_geo!(0.014, 0.014, 0.004)), &brass_mat);
        buckle.position().set(s * 0.05, 0.048, 0.072);
        suitcase.add(&buckle);
        // Buckle latch (small strip).
        let latch = mesh(pg("SmallSuitcases__latch", box_geo!(0.008, 0.006, 0.002)), &brass_mat);
        latch.position().set(s * 0.05, 0.048, 0.075);
        suitcase.add(&latch);
    }
    // Corner feet at bottom.
    for (cx, cz) in [(-1.0f64, -1.0f64), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        let corner = mesh(pg("SmallSuitcases__foot", cyl_geo!(0.008, 0.008, 0.008, 5)), &brass_mat);
        corner.position().set(cx * 0.095, 0.004, cz * 0.055);
        suitcase.add(&corner);
    }
    suitcase.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    suitcase.rotation().y = rnd() * PI * 2.0;
    g.world.add(&suitcase);
}

// ------------------------------------------------------------------ line 6594
// R475 small vintage phonograph: 4% chance per level.
fn place_small_phonographs(g: &mut Game, _dist: &[i32]) {
    g.pp.phonograph_positions.clear();   // R480 reset per level
    if rnd() < 0.96 { return; }   // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let wood_mat = lambert!({ color: 0x6a3820 });
    let brass_mat = lambert!({ color: 0xc89040 });
    let record_mat = lambert!({ color: 0x201a10 });
    let label_mat = lambert!({ color: 0xa02040 });
    let phono = group();
    // Wood base cube.
    let base = mesh(pg("SmallPhonographs__base", box_geo!(0.16, 0.06, 0.14)), &wood_mat);
    base.position().y = 0.03;
    phono.add(&base);
    // Turntable disc.
    let turntable = mesh(pg("SmallPhonographs__turntable", cyl_geo!(0.055, 0.055, 0.006, 12)), &record_mat);
    turntable.position().y = 0.063;
    phono.add(&turntable);
    // Record on turntable (slightly smaller).
    let record = mesh(pg("SmallPhonographs__record", cyl_geo!(0.052, 0.052, 0.002, 14)), &record_mat);
    record.position().y = 0.067;
    phono.add(&record);
    // Record label.
    let label = mesh(circle_geo!(0.015, 8), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().y = 0.0685;
    phono.add(&label);
    // Tone arm: from its pivot (collar end, -Y) on the cabinet's back-right
    // corner to the sound box (+Y end) on the record.
    let arm_pivot = v3(0.076, 0.064, -0.068);
    let arm_tip = v3(0.032, 0.0745, 0);
    let arm = mesh(pg("SmallPhonographs__arm", cyl_geo!(0.003, 0.003, 0.08, 4)), &brass_mat);
    arm.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &vsub(arm_tip, arm_pivot).normalized());
    arm.scale().y = arm_tip.distance_to(&arm_pivot) / 0.08;
    {
        let mut p = arm.position();
        p.copy(&arm_pivot).add(&arm_tip).multiply_scalar(0.5);
    }
    phono.add(&arm);
    let arm_head = mesh(pg("SmallPhonographs__head", box_geo!(0.010, 0.006, 0.008)), &brass_mat);
    arm_head.position().set(0.032, 0.070, 0);
    phono.add(&arm_head);
    // Large brass horn: throat (apex, +Y) at the arm pivot, the flared mouth up and back.
    let horn = mesh(pg("SmallPhonographs__horn", cone_geo!(0.08, 0.20, 12, 1, true)), &brass_mat);
    let horn_dir = v3(-0.30, 0.90, -0.30).normalized();
    horn.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &(-horn_dir));
    {
        let mut p = horn.position();
        p.copy(&arm_pivot).add_scaled_vector(&horn_dir, 0.10);
    }
    phono.add(&horn);
    // Turntable slow rotation animator.
    let rot_rate = 0.5 + rnd() * 0.5;
    {
        let (turntable, record, label) = (turntable.clone(), record.clone(), label.clone());
        g.animate(move |_g, _dt, t| {
            turntable.rotation().y = t * rot_rate;
            record.rotation().y = t * rot_rate;
            label.rotation().z = t * rot_rate;
        });
    }
    phono.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    phono.rotation().y = rnd() * PI * 2.0;
    g.world.add(&phono);
    // R480 track for vinyl crackle audio.
    let pp = phono.pos();
    g.pp.phonograph_positions.push(v3(pp.x, 0.10, pp.z));
}

// ------------------------------------------------------------------ line 6667
// R475 small rotary phone: 6% chance per level.
fn place_small_rotary_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; }   // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let body_colors = [0x181008, 0x502818, 0xa02040];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let dial_mat = lambert!({ color: 0xc0c0c0 });
    let cord_mat = lambert!({ color: 0x201a10 });
    let phone = group();
    // Base body.
    let base = mesh(pg("SmallRotaryPhones__base", box_geo!(0.13, 0.05, 0.10)), &body_mat);
    base.position().y = 0.025;
    phone.add(&base);
    // Rotary dial (disc on top).
    let dial = mesh(pg("SmallRotaryPhones__dial", cyl_geo!(0.028, 0.028, 0.006, 14)), &dial_mat);
    dial.position().set(0, 0.053, 0.008);
    phone.add(&dial);
    // 10 dial finger holes.
    for i in 0..10 {
        let a = (i as f64 / 10.0) * PI * 2.0;
        let hole = mesh(pg("SmallRotaryPhones__hole", cyl_geo!(0.003, 0.003, 0.008, 5)), &body_mat);
        hole.position().set(a.cos() * 0.020, 0.055, 0.008 + a.sin() * 0.020);
        phone.add(&hole);
    }
    // 2 handset cradle posts.
    for s in [-1.0f64, 1.0] {
        let post = mesh(pg("SmallRotaryPhones__post", cyl_geo!(0.008, 0.010, 0.014, 6)), &body_mat);
        post.position().set(s * 0.045, 0.056, -0.028);
        phone.add(&post);
    }
    // Handset sitting on cradle.
    let handset = mesh(pg("SmallRotaryPhones__handset", cyl_geo!(0.010, 0.010, 0.11, 8)), &body_mat);
    handset.rotation().z = PI / 2.0;
    handset.position().set(0, 0.070, -0.028);
    phone.add(&handset);
    // Ear/mouth cups.
    for s in [-1.0f64, 1.0] {
        let cup = mesh(pg("SmallRotaryPhones__cup", cyl_geo!(0.014, 0.012, 0.010, 8)), &body_mat);
        cup.rotation().z = PI / 2.0;
        cup.position().set(s * 0.048, 0.070, -0.028);
        phone.add(&cup);
    }
    // Curly cord (torus loop).
    let cord = mesh(pg("SmallRotaryPhones__cord", torus_geo!(0.020, 0.002, 4, 12, PI * 1.5)), &cord_mat);
    cord.rotation().x = PI / 2.0;
    cord.rotation().z = PI / 2.0;
    cord.position().set(0.070, 0.030, -0.028);
    phone.add(&cord);
    phone.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    phone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&phone);
}

// ------------------------------------------------------------------ line 6734
// R474 small vintage typewriter: 5% chance per level.
fn place_small_typewriters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; }   // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let body_mat = lambert!({ color: 0x201a10 });
    let key_mat = lambert!({ color: 0xf0ece0 });
    let paper_mat = lambert!({ color: 0xe8dcc0 });
    let roll_mat = lambert!({ color: 0x2a2418 });
    let tw = group();
    // Base body.
    let base = mesh(pg("SmallTypewriters__base", box_geo!(0.16, 0.02, 0.14)), &body_mat);
    base.position().y = 0.010;
    tw.add(&base);
    // Angled key deck (top-front slope).
    let deck = mesh(pg("SmallTypewriters__deck", box_geo!(0.16, 0.010, 0.08)), &body_mat);
    deck.rotation().x = 0.35;   // low at the typist (+z), rising toward the roll
    deck.position().set(0, 0.036, 0.03);
    tw.add(&deck);
    // 4x10 key grid, sitting on the deck's recessed floor (row 0 at the front).
    for ky in 0..4 {
        for kx in 0..10 {
            let key = mesh(pg("VintageTypewriters__key", cyl_geo!(0.005, 0.005, 0.004, 5)), &key_mat);
            let kkx = -0.070 + kx as f64 * 0.0155;
            let kkz = 0.0567 - ky as f64 * 0.0169;
            key.rotation().x = 0.35;
            key.position().set(kkx, 0.0305 + ky as f64 * 0.0062, kkz);
            tw.add(&key);
        }
    }
    // Back platen (paper roll cylinder), resting on the base.
    let roll = mesh(pg("SmallTypewriters__roll", cyl_geo!(0.020, 0.020, 0.13, 8)), &roll_mat);
    roll.rotation().z = PI / 2.0;
    roll.position().set(0, 0.040, -0.045);
    tw.add(&roll);
    // Paper coming out top.
    let paper = mesh(plane_geo!(0.12, 0.07), &paper_mat);
    paper.position().set(0, 0.078, -0.045);
    paper.rotation().x = -0.15;
    tw.add(&paper);
    // Return lever on side: foot on the roll's left end, leaning up and out.
    let lever = mesh(pg("VintageTypewriters__lever", cyl_geo!(0.003, 0.003, 0.030, 4)), &roll_mat);
    lever.rotation().z = 0.5;
    lever.position().set(-0.073, 0.058, -0.045);
    tw.add(&lever);
    tw.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    tw.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tw);
}

// ------------------------------------------------------------------ line 6795
// R474 small vintage globe texture.
game::lazy_static_tex!(globe_texture, canvas_tex(256, 128, |ctx, w, h| {
    // Ocean blue.
    ctx.set_fill_style("#4080b0");
    ctx.fill_rect(0, 0, w, h);
    // Continent blobs (5 land shapes).
    ctx.set_fill_style("#508050");
    let conts: [(f64, f64, f64, f64); 5] = [
        (40.0, 60.0, 30.0, 22.0),
        (100.0, 50.0, 25.0, 30.0),
        (160.0, 40.0, 20.0, 18.0),
        (200.0, 70.0, 26.0, 24.0),
        (130.0, 90.0, 22.0, 16.0),
    ];
    for &(cx, cy, rx, ry) in conts.iter() {
        ctx.begin_path();
        ctx.ellipse(cx, cy, rx, ry, rnd() * 0.5, 0, PI * 2.0);
        ctx.fill();
        // Extra bumps for irregular shape.
        for _ in 0..4 {
            let a = rnd() * PI * 2.0;
            let r = rx.min(ry) * 0.4;
            ctx.begin_path();
            ctx.arc(cx + a.cos() * r * 1.4, cy + a.sin() * r * 1.4, r * 0.8, 0, PI * 2.0);
            ctx.fill();
        }
    }
    // Equator + meridian faint lines.
    ctx.set_stroke_style("rgba(80,40,20,0.4)");
    ctx.set_line_width(1);
    ctx.begin_path();
    ctx.move_to(0, h / 2.0);
    ctx.line_to(w, h / 2.0);
    ctx.stroke();
    for i in 1..6 {
        let fi = i as f64;
        ctx.begin_path();
        ctx.move_to(fi * w / 6.0, 0);
        ctx.line_to(fi * w / 6.0, h);
        ctx.stroke();
    }
}));

// ------------------------------------------------------------------ line 6835
// R474 small vintage globe: 5% chance per level, slow spin.
fn place_small_globes(g: &mut Game, _dist: &[i32]) {
    g.pp.globe_positions.clear();   // R482 reset per level
    if rnd() < 0.95 { return; }   // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let wood_mat = lambert!({ color: 0x8a5030 });
    let brass_mat = lambert!({ color: 0xc89040 });
    let globe_mat = lambert!({ map: globe_texture() });
    let globe = group();
    // Base disc.
    let base = mesh(pg("SmallGlobes__base", cyl_geo!(0.045, 0.055, 0.010, 10)), &wood_mat);
    base.position().y = 0.005;
    globe.add(&base);
    // Column.
    let col = mesh(pg("SmallGlobes__column", cyl_geo!(0.006, 0.008, 0.050, 6)), &wood_mat);
    col.position().y = 0.035;
    globe.add(&col);
    // Meridian ring (torus).
    let meridian = mesh(pg("SmallGlobes__meridian", torus_geo!(0.055, 0.003, 4, 20)), &brass_mat);
    meridian.rotation().z = 0.4;
    meridian.position().y = 0.115;
    globe.add(&meridian);
    // Sphere with map; its axis is tilted with the meridian so its poles meet
    // the meridian's pivot pins, and the spin turns it about that axis.
    let sphere = mesh(pg("SmallGlobes__sphere", sphere_geo!(0.050, 16, 12)), &globe_mat);
    let globe_axis = group();
    globe_axis.rotation().z = 0.4;
    globe_axis.position().y = 0.115;
    globe_axis.add(&sphere);
    globe.add(&globe_axis);
    // Slow rotation animator.
    let spin_rate = 0.15 + rnd() * 0.15;
    let sphere2 = sphere.clone();
    g.animate(move |_g, _dt, t| {
        sphere2.rotation().y = t * spin_rate;
    });
    globe.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    globe.rotation().y = rnd() * PI * 2.0;
    g.world.add(&globe);
    // R482 track for wood creak audio.
    let gp = globe.pos();
    g.pp.globe_positions.push(v3(gp.x, 0.12, gp.z));
}

// ------------------------------------------------------------------ line 6891
// R473 small brass telescope: 6% chance per level.
fn place_small_telescopes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; }   // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let leather_mat = lambert!({ color: 0x6a3820 });
    let lens_mat = basic_mat!({ color: 0x304050, transparent: true, opacity: 0.75 });
    let scope = group();
    // 3 sections (largest to smallest, telescoped): (len, r, x).
    let sections: [(f64, f64, f64); 3] = [(0.10, 0.020, 0.05), (0.09, 0.015, 0.145), (0.08, 0.011, 0.230)];
    for &(len, r, sx) in sections.iter() {
        let body = mesh(pg("SmallTelescopes__tube", cyl_geo!(r, r, len, 10)), &brass_mat);
        body.rotation().z = PI / 2.0;
        body.position().set(sx, 0.020, 0);
        scope.add(&body);
    }
    // Leather grip on largest section.
    let grip = mesh(pg("SmallTelescopes__grip", cyl_geo!(0.021, 0.021, 0.020, 10)), &leather_mat);
    grip.rotation().z = PI / 2.0;
    grip.position().set(0.028, 0.020, 0);
    scope.add(&grip);
    // Eyepiece at large end.
    let eye = mesh(circle_geo!(0.018, 8), &lens_mat);
    eye.rotation().y = -PI / 2.0;
    eye.position().set(-0.001, 0.020, 0);
    scope.add(&eye);
    // Lens at small end.
    let lens = mesh(circle_geo!(0.010, 8), &lens_mat);
    lens.rotation().y = PI / 2.0;
    lens.position().set(0.271, 0.020, 0);
    scope.add(&lens);
    scope.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    scope.rotation().y = rnd() * PI * 2.0;
    g.world.add(&scope);
}

// ------------------------------------------------------------------ line 6945
// R473 small padlock + key: 10% chance per level.
fn place_small_padlock_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 { return; }   // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let dark_mat = lambert!({ color: 0x2a1810 });
    let grp = group();
    // === PADLOCK ===
    let lock = group();
    // Body: rounded box.
    let body = mesh(pg("SmallPadlockKeys__body", box_geo!(0.040, 0.048, 0.020)), &brass_mat);
    body.position().y = 0.028;
    lock.add(&body);
    // Keyhole dark circle on front.
    let keyhole = mesh(circle_geo!(0.005, 6), &dark_mat);
    keyhole.position().set(0, 0.024, 0.011);
    lock.add(&keyhole);
    // Shackle (U-shape arch): an inverted U standing on the two posts.
    let shackle = mesh(pg("SmallPadlockKeys__shackle", torus_geo!(0.014, 0.004, 5, 12, PI)), &brass_mat);
    shackle.position().y = 0.056;
    lock.add(&shackle);
    // 2 shackle posts.
    for s in [-1.0f64, 1.0] {
        let post = mesh(pg("SmallPadlockKeys__post", cyl_geo!(0.004, 0.004, 0.020, 5)), &brass_mat);
        post.position().set(s * 0.014, 0.048, 0);
        lock.add(&post);
    }
    lock.position().set(-0.06, -0.004, 0);   // body bottom (0.004) on the floor
    grp.add(&lock);
    // === KEY === (lying flat: shaft, bow and bit all on the floor)
    let key = group();
    // Shaft.
    let shaft = mesh(pg("SmallPadlockKeys__shaft", cyl_geo!(0.002, 0.002, 0.080, 4)), &brass_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().set(0.040, 0.002, 0);
    key.add(&shaft);
    // Bow (ring at end), flat in the shaft's plane.
    let bow = mesh(pg("SmallPadlockKeys__bow", torus_geo!(0.012, 0.003, 4, 10)), &brass_mat);
    bow.rotation().x = PI / 2.0;
    bow.position().set(-0.005, 0.003, 0);
    key.add(&bow);
    // 2 teeth on other end, lying flat off the shaft's side.
    for i in 0..2 {
        let tooth = mesh(pg("SmallPadlockKeys__tooth", box_geo!(0.006, 0.008, 0.002)), &brass_mat);
        tooth.rotation().x = -PI / 2.0;
        tooth.position().set(0.070 + i as f64 * 0.008, 0.002, 0.005);
        key.add(&tooth);
    }
    key.position().set(0.02, 0, 0.05);
    key.rotation().y = -PI / 6.0;
    grp.add(&key);
    grp.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

// ------------------------------------------------------------------ line 7013
// R472 small parchment scroll: 8% chance per level.
fn place_parchment_scrolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let parchment_mat = lambert!({ color: 0xe8dcc0 });
    let shadow_mat = lambert!({ color: 0xc0b090 });
    let ribbon_colors = [0xa02040, 0x4030c0, 0x2a2418, 0x8a2040];
    let ribbon_mat = lambert!({ color: pick(&ribbon_colors), side: DoubleSide });
    let scroll = group();
    let scroll_len = 0.22;
    let scroll_r = 0.020;
    // Main roll.
    let roll = mesh(pg("ParchmentScrolls__roll", cyl_geo!(scroll_r, scroll_r, scroll_len, 12)), &parchment_mat);
    roll.rotation().z = PI / 2.0;
    roll.position().set(scroll_len / 2.0, scroll_r, 0);
    scroll.add(&roll);
    // Inner shadow discs at ends (roll depth).
    for s in [-1.0f64, 1.0] {
        let shadow_disc = mesh(circle_geo!(scroll_r * 0.75, 8), &shadow_mat);
        shadow_disc.rotation().y = PI / 2.0;
        shadow_disc.position().set(if s > 0.0 { scroll_len - 0.001 } else { 0.001 }, scroll_r, 0);
        if s < 0.0 { shadow_disc.rotation().y = -PI / 2.0; }
        scroll.add(&shadow_disc);
    }
    // Ribbon wrapped around middle, snug on the roll (radius 0.021).
    let ribbon = mesh(cyl_geo!(scroll_r + 0.003, scroll_r + 0.003, 0.012, 12, 1, true), &ribbon_mat);
    ribbon.scale().set((scroll_r + 0.001) / (scroll_r + 0.003), 1, (scroll_r + 0.001) / (scroll_r + 0.003));
    ribbon.rotation().z = PI / 2.0;
    ribbon.position().set(scroll_len / 2.0, scroll_r, 0);
    scroll.add(&ribbon);
    // 2 ribbon tail planes hanging.
    for s in [-1.0f64, 1.0] {
        let tail = mesh(plane_geo!(0.008, 0.030), &ribbon_mat);
        tail.rotation().y = PI / 2.0;
        tail.position().set(scroll_len / 2.0, scroll_r + 0.008, s * 0.024);
        tail.rotation().z = s * 0.2;
        scroll.add(&tail);
    }
    scroll.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    scroll.rotation().y = rnd() * PI * 2.0;
    g.world.add(&scroll);
}

// ------------------------------------------------------------------ line 7072
// R472 compass face texture.
game::lazy_static_tex!(compass_face_texture, canvas_tex_opt(128, 128, false, |ctx, w, h| {
    ctx.set_fill_style("#f0ece0");
    ctx.begin_path();
    ctx.arc(w / 2.0, h / 2.0, w / 2.0 - 4.0, 0, PI * 2.0);
    ctx.fill();
    ctx.set_stroke_style("#181008");
    ctx.set_line_width(2);
    ctx.stroke();
    // Cardinal N/E/S/W.
    ctx.set_fill_style("#181008");
    ctx.set_font("bold 16px serif");
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    ctx.fill_text("N", w / 2.0, 14);
    ctx.fill_text("E", w - 14.0, h / 2.0);
    ctx.fill_text("S", w / 2.0, h - 14.0);
    ctx.fill_text("W", 14, h / 2.0);
    // Cardinal marks (short lines).
    ctx.set_line_width(3);
    for i in 0..16 {
        let a = -PI / 2.0 + (i as f64 / 16.0) * PI * 2.0;
        ctx.begin_path();
        ctx.move_to(w / 2.0 + a.cos() * 40.0, h / 2.0 + a.sin() * 40.0);
        ctx.line_to(w / 2.0 + a.cos() * 48.0, h / 2.0 + a.sin() * 48.0);
        ctx.stroke();
    }
    // Needle (red-tip pointing north).
    ctx.set_fill_style("#d82040");
    ctx.begin_path();
    ctx.move_to(w / 2.0, h / 2.0 - 30.0);
    ctx.line_to(w / 2.0 - 4.0, h / 2.0);
    ctx.line_to(w / 2.0 + 4.0, h / 2.0);
    ctx.close_path();
    ctx.fill();
    ctx.set_fill_style("#404040");
    ctx.begin_path();
    ctx.move_to(w / 2.0, h / 2.0 + 30.0);
    ctx.line_to(w / 2.0 - 4.0, h / 2.0);
    ctx.line_to(w / 2.0 + 4.0, h / 2.0);
    ctx.close_path();
    ctx.fill();
    // Center pin.
    ctx.set_fill_style("#181008");
    ctx.begin_path();
    ctx.arc(w / 2.0, h / 2.0, 4, 0, PI * 2.0);
    ctx.fill();
}));

// ------------------------------------------------------------------ line 7119
// R472 small vintage compass: 8% chance per level.
fn place_small_compasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let face_mat = lambert!({ map: compass_face_texture(), side: DoubleSide });
    let compass = group();
    // Case (disc).
    let case_ = mesh(pg("SmallCompasses__case", cyl_geo!(0.036, 0.036, 0.012, 14)), &brass_mat);
    case_.position().y = 0.006;
    compass.add(&case_);
    // Face on top.
    let face = mesh(circle_geo!(0.030, 14), &face_mat);
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0125;
    compass.add(&face);
    // Hinge: barrel along Z (the lid's pivot axis) at the case's top edge.
    let hinge = mesh(pg("SmallCompasses__hinge", cyl_geo!(0.005, 0.005, 0.014, 6)), &brass_mat);
    hinge.rotation().x = PI / 2.0;
    hinge.position().set(0.039, 0.011, 0);
    compass.add(&hinge);
    // Cover leaning open: the closed lid swung 120 deg about the hinge.
    let cover = mesh(pg("SmallCompasses__cover", cyl_geo!(0.036, 0.036, 0.006, 14)), &brass_mat);
    cover.position().set(0.039 + 0.0230, 0.011 + 0.0318, 0);
    cover.rotation().z = -2.0 * PI / 3.0;
    compass.add(&cover);
    compass.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    compass.rotation().y = rnd() * PI * 2.0;
    g.world.add(&compass);
}

// ------------------------------------------------------------------ line 7201
// R481 small pocket watch: 8% chance per level.
fn place_small_pocket_watches(g: &mut Game, _dist: &[i32]) {
    g.pp.pocket_watch_positions.clear();   // R481 reset per level
    if rnd() < 0.92 { return; }   // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() { return; }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let face_mat = lambert!({ map: watch_face_texture(), side: DoubleSide });
    let watch = group();
    // Case (disc).
    let case_ = mesh(pg("SmallPocketWatches__case", cyl_geo!(0.028, 0.028, 0.010, 14)), &brass_mat);
    case_.position().y = 0.005;
    watch.add(&case_);
    // Face on top.
    let face = mesh(circle_geo!(0.024, 14), &face_mat);
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0105;
    watch.add(&face);
    // Crown on side.
    let crown = mesh(pg("SmallPocketWatches__crown", cyl_geo!(0.004, 0.004, 0.006, 6)), &brass_mat);
    crown.rotation().z = PI / 2.0;
    crown.position().set(0.031, 0.005, 0);
    watch.add(&crown);
    // Chain of tiny torus links.
    for i in 0..6 {
        let fi = i as f64;
        let link = mesh(pg("SmallPocketWatches__link", torus_geo!(0.005, 0.001, 4, 8)), &brass_mat);
        link.position().set(0.036 + fi * 0.008, 0.005 + (fi * 0.4).sin() * 0.003, 0);
        link.rotation().x = if i % 2 == 0 { PI / 2.0 } else { 0.0 };
        watch.add(&link);
    }
    watch.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    watch.rotation().y = rnd() * PI * 2.0;
    g.world.add(&watch);
    // R481 track for tick audio.
    let wp = watch.pos();
    g.pp.pocket_watch_positions.push(v3(wp.x, 0.01, wp.z));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::testkit::smoke_placers(&list);
    }
}
