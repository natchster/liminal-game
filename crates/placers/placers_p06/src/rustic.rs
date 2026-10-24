//! Rustic cabin props: antlers, signs, trophy fish, cuckoo clock, lantern,
//! cast iron, firewood, wagon wheel, map, bellows (R653–R660).
use crate::{face_yaw, open_tiles, pick, wall_tiles};
use game::prelude::*;

// R653 mounted deer antler pair: 3% chance per level. Wooden plaque +
// branching antler cylinders.
pub fn place_mounted_antlers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let plaque_mat = lambert!({ color: 0x604020 });
    let antler_mat = lambert!({ color: 0xd0c090 });
    let mount = group();
    let mount_y = 1.80;
    // Shield-shape wooden plaque (rounded top).
    let plaque = mesh(pg("MountedAntlers__plaque", box_geo!(0.18, 0.22, 0.020)), &plaque_mat);
    plaque.position().y = mount_y;
    mount.add(&plaque);
    // Rounded top of plaque (half-sphere).
    let plaque_top = mesh(pg("MountedAntlers__crest", sphere_geo!(0.09, 8, 5, 0, PI * 2.0, 0, PI / 2.0)), &plaque_mat);
    plaque_top.position().y = mount_y + 0.11;
    plaque_top.scale().set(1, 0.5, 0.2);
    mount.add(&plaque_top);
    // 2 antlers extending upward from base.
    for side in [-1.0f64, 1.0] {
        // Main antler trunk.
        let trunk = mesh(pg("MountedAntlers__trunk", cyl_geo!(0.006, 0.010, 0.14, 6)), &antler_mat);
        // burrs together near the middle, beams leaning outwards (a V, not a Λ)
        trunk.rotation().z = -side * 0.3;
        trunk.position().set(side * 0.035, mount_y + 0.10, 0.014);
        mount.add(&trunk);
        // 2 branching prongs per antler: tines rising from the beam (base 45 % and
        // 75 % of the way up it), pointing up and forward, the lower one a little inwards.
        let beam_dir = [side * 0.3f64.sin(), 0.3f64.cos()];
        for b in 0..2 {
            let prong = mesh(pg("MountedAntlers__prong", cyl_geo!(0.004, 0.005, 0.055, 5)), &antler_mat);
            prong.rotation().z = side * (0.25 - b as f64 * 0.15);
            prong.rotation().x = 0.4;
            let along = (0.45 + b as f64 * 0.30 - 0.5) * 0.14;
            let tz = prong.rot().z;
            prong.position().set(
                side * 0.035 + beam_dir[0] * along - tz.sin() * 0.0275,
                mount_y + 0.10 + beam_dir[1] * along + tz.cos() * 0.4f64.cos() * 0.0275,
                0.014 + tz.cos() * 0.4f64.sin() * 0.0275,
            );
            mount.add(&prong);
        }
    }
    mount.rotation().y = face_yaw(dx, dz);
    mount.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&mount);
}

// R653 fertilizer spike cluster: 8% chance per level. 3-5 short colored
// plastic sticks on floor.
pub fn place_fertilizer_spikes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let spike_colors: [u32; 4] = [0x408040, 0x2060c0, 0xc02020, 0xf0e020];
    let cluster = group();
    let count = 3 + (rnd() * 3.0).floor() as i32;
    for _i in 0..count {
        let spike_mat = lambert!({ color: pick(&spike_colors) });
        let spike = group();
        // Main stick.
        let body = mesh(pg("FertilizerSpikes__body", cyl_geo!(0.003, 0.004, 0.055, 6)), &spike_mat);
        body.rotation().z = PI / 2.0;
        body.position().y = 0.003;
        spike.add(&body);
        // Pointed cone tip.
        let tip = mesh(pg("FertilizerSpikes__tip", cone_geo!(0.004, 0.012, 5)), &spike_mat);
        tip.rotation().z = -PI / 2.0;
        tip.position().set(0.033, 0.003, 0);
        spike.add(&tip);
        spike.position().set((rnd() - 0.5) * 0.12, 0, (rnd() - 0.5) * 0.12);
        spike.rotation().y = rnd() * PI * 2.0;
        cluster.add(&spike);
    }
    cluster.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&cluster);
}

// R654 rustic wooden wall sign: 4% chance per level. Wood planks +
// burned/carved text.
game::lazy_static_tex!(
    rustic_sign_texture,
    canvas_tex(160, 80, |ctx, w, h| {
        // Warm wood base.
        ctx.set_fill_style("#8b5a2b");
        ctx.fill_rect(0, 0, w, h);
        // Horizontal plank divider lines (darker).
        ctx.set_stroke_style("#4a2f1a");
        ctx.set_line_width(2);
        for i in 1..3 {
            ctx.begin_path();
            ctx.move_to(0, i as f64 * h / 3.0);
            ctx.line_to(w, i as f64 * h / 3.0);
            ctx.stroke();
        }
        // Wood grain streaks.
        ctx.set_stroke_style("rgba(60, 40, 20, 0.4)");
        ctx.set_line_width(1);
        for _i in 0..8 {
            ctx.begin_path();
            let y0 = rnd() * h;
            ctx.move_to(0, y0);
            let y1 = rnd() * h;
            let y2 = rnd() * h;
            let y3 = rnd() * h;
            ctx.bezier_curve_to(w * 0.3, y1, w * 0.7, y2, w, y3);
            ctx.stroke();
        }
        // Burned "text" placeholder — 2 lines of dark bars.
        ctx.set_fill_style("#201810");
        ctx.fill_rect(24, 20, w - 48.0, 6);
        ctx.fill_rect(32, 42, w - 64.0, 5);
    })
);

pub fn place_rustic_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let wood_mat = lambert!({ map: rustic_sign_texture() });
    let rope_mat = lambert!({ color: 0x604020 });
    let sign = group();
    let mount_y = 1.75;
    // Main wood panel.
    let panel = mesh(pg("RusticSigns__panel", box_geo!(0.30, 0.15, 0.015)), &wood_mat);
    panel.position().y = mount_y;
    sign.add(&panel);
    // 2 hanging rope strands from top corners.
    for side in [-1.0, 1.0] {
        let rope = mesh(pg("RusticSigns__rope", cyl_geo!(0.002, 0.002, 0.10, 4)), &rope_mat);
        rope.rotation().z = side * 0.15;
        rope.position().set(side * 0.10, mount_y + 0.12, 0);
        sign.add(&rope);
    }
    sign.rotation().z = (rnd() - 0.5) * 0.06;
    sign.rotation().y = face_yaw(dx, dz);
    sign.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    g.world.add(&sign);
}

// R654 mounted trophy fish: 3% chance per level. Wood plaque + elongated fish
// body w/ tail + fins.
pub fn place_mounted_fish(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let plaque_mat = lambert!({ color: 0x604020 });
    let fish_colors: [u32; 4] = [0x408090, 0x60a080, 0x808060, 0x80a0c0];
    let fish_mat = lambert!({ color: pick(&fish_colors) });
    let eye_mat = lambert!({ color: 0x101010 });
    let mount = group();
    let mount_y = 1.65;
    // Wooden plaque (horizontal oval).
    let plaque = mesh(pg("MountedFish__plaque", box_geo!(0.35, 0.15, 0.018)), &plaque_mat);
    plaque.position().y = mount_y;
    mount.add(&plaque);
    // Fish body (elongated ellipsoid).
    let body = mesh(pg("MountedFish__body", sphere_geo!(0.045, 10, 8)), &fish_mat);
    body.scale().set(2.5, 1, 0.6);
    body.position().set(-0.02, mount_y, 0.020);
    mount.add(&body);
    // Tail (triangular fin).
    let tail = mesh(pg("MountedFish__tail", cone_geo!(0.028, 0.040, 3)), &fish_mat);
    tail.rotation().z = -PI / 2.0;
    tail.rotation().x = PI / 2.0;
    tail.position().set(0.115, mount_y, 0.020);
    mount.add(&tail);
    // Dorsal fin (triangle on top).
    let dorsal = mesh(pg("MountedFish__dorsal", cone_geo!(0.020, 0.030, 3)), &fish_mat);
    dorsal.rotation().x = PI / 2.0;
    // fin's foot on the back (body top +0.045): the crest stands clear, the root stays in the body
    dorsal.position().set(-0.010, mount_y + 0.043, 0.020);
    mount.add(&dorsal);
    // Eye.
    let eye = mesh(pg("MountedFish__eye", sphere_geo!(0.006, 5, 4)), &eye_mat);
    eye.position().set(-0.090, mount_y + 0.005, 0.038);
    mount.add(&eye);
    mount.rotation().y = face_yaw(dx, dz);
    mount.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.017), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.017));
    g.world.add(&mount);
}

// R655 cuckoo clock: 3% chance per level. Chalet body w/ peaked roof + face +
// pinecone weights.
pub fn place_cuckoo_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.cuckoo_clock_positions.clear(); // R664 reset per level
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let chalet_mat = lambert!({ color: 0x8b5a2b });
    let roof_mat = lambert!({ color: 0x604020 });
    let face_mat = lambert!({ color: 0xf0e8c8 });
    let hand_mat = lambert!({ color: 0x101010 });
    let cone_mat = lambert!({ color: 0x504020 });
    let cuckoo = group();
    let mount_y = 1.75;
    // Chalet body (main rectangle).
    let body = mesh(pg("CuckooClocks__body", box_geo!(0.22, 0.20, 0.06)), &chalet_mat);
    body.position().y = mount_y;
    cuckoo.add(&body);
    // Peaked roof (triangular prism via cone with 3 sides).
    let roof = mesh(pg("CuckooClocks__roof", cone_geo!(0.14, 0.10, 3)), &roof_mat);
    roof.rotation().y = PI / 2.0;
    roof.position().y = mount_y + 0.158; // gable wall's foot (part y −0.058) on the case top (+0.10)
    cuckoo.add(&roof);
    // Round clock face.
    let face = mesh(circle_geo!(0.055, 12), &face_mat);
    face.position().set(0, mount_y + 0.020, 0.032);
    cuckoo.add(&face);
    // Face rim.
    let rim = mesh(pg("CuckooClocks__rim", torus_geo!(0.055, 0.004, 4, 12)), &roof_mat);
    rim.rotation().x = 0.0;
    rim.position().set(0, mount_y + 0.020, 0.033);
    cuckoo.add(&rim);
    // Hour + minute hands.
    let hour_h = mesh(pg_own("CuckooClocks__hour", box_geo!(0.004, 0.030, 0.003)), &hand_mat);
    hour_h.position().set(0, mount_y + 0.035, 0.036);
    hour_h.geo().translate(0, -0.005, 0);
    hour_h.rotation().z = rnd() * PI * 2.0;
    cuckoo.add(&hour_h);
    let minute_h = mesh(pg_own("CuckooClocks__minute", box_geo!(0.003, 0.040, 0.003)), &hand_mat);
    minute_h.position().set(0, mount_y + 0.035, 0.037);
    minute_h.geo().translate(0, -0.010, 0);
    minute_h.rotation().z = rnd() * PI * 2.0;
    cuckoo.add(&minute_h);
    // 2 pinecone weights hanging below body (chain + cone).
    for side in [-1.0, 1.0] {
        let chain = mesh(pg("CuckooClocks__chain", cyl_geo!(0.001, 0.001, 0.12, 3)), &roof_mat);
        chain.position().set(side * 0.070, mount_y - 0.16, 0.020);
        cuckoo.add(&chain);
        let cone = mesh(pg("CuckooClocks__cone", cone_geo!(0.010, 0.024, 6)), &cone_mat);
        cone.rotation().x = PI;
        cone.position().set(side * 0.070, mount_y - 0.231, 0.020); // eye (top) on the chain's end (−0.22)
        cuckoo.add(&cone);
    }
    cuckoo.rotation().y = face_yaw(dx, dz);
    cuckoo.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.030), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.030));
    g.world.add(&cuckoo);
    let p = cuckoo.pos();
    g.pp.cuckoo_clock_positions.push(v3(p.x, 0, p.z)); // R664 track for cuckoo call
}

// R655 metal candle lantern: 4% chance per level. Frame + glass panels +
// candle + handle.
pub fn place_candle_lanterns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let frame_mat = lambert!({ color: 0x303030 });
    let glass_mat = basic_mat!({ color: 0xe8e0c0, transparent: true, opacity: 0.35 });
    let candle_mat = lambert!({ color: 0xf0e8d0 });
    let is_lit = rnd() < 0.4;
    let flame_mat = basic_mat!({ color: 0xffa040 });
    let lantern = group();
    // Base disc.
    let base = mesh(pg("CandleLanterns__base", cyl_geo!(0.045, 0.045, 0.010, 6)), &frame_mat);
    base.position().y = 0.005;
    lantern.add(&base);
    // 4 corner posts.
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0 + PI / 4.0;
        let post = mesh(pg("CandleLanterns__post", cyl_geo!(0.003, 0.003, 0.14, 4)), &frame_mat);
        post.position().set(a.cos() * 0.032, 0.075, a.sin() * 0.032);
        lantern.add(&post);
    }
    // 4 glass panels (planes between posts).
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0;
        let panel = mesh(plane_geo!(0.045, 0.12), &glass_mat);
        // on the line between two diagonal posts (0.032 · cos 45° from the centre)
        panel.position().set(a.cos() * 0.032 * std::f64::consts::FRAC_1_SQRT_2, 0.075, a.sin() * 0.032 * std::f64::consts::FRAC_1_SQRT_2);
        panel.rotation().y = -a + PI / 2.0;
        lantern.add(&panel);
    }
    // Top cap.
    let top = mesh(pg("CandleLanterns__top", cyl_geo!(0.045, 0.045, 0.010, 6)), &frame_mat);
    top.position().y = 0.145;
    lantern.add(&top);
    // Candle inside.
    let candle = mesh(pg("CandleLanterns__candle", cyl_geo!(0.012, 0.012, 0.060, 8)), &candle_mat);
    candle.position().y = 0.045;
    candle.scale().y = 0.070 / 0.060; // from the plinth top (0.010) up to the flame (0.081)
    lantern.add(&candle);
    if is_lit {
        // Wick + flame.
        let flame = mesh(cone_geo!(0.005, 0.014, 5), &flame_mat);
        flame.position().y = 0.088;
        lantern.add(&flame);
        // Light source.
        let light = point_light!(0xffa040, 0.18, 1.5);
        light.position().y = 0.088;
        lantern.add(&light);
        // R656 flame flicker animator (rapid noise-like intensity + scale wobble).
        let phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            // Rapid flicker via combined sines (like turbulent flame).
            let f = 0.7 + 0.15 * (t * 7.0 + phase).sin() + 0.15 * (t * 11.0 + phase * 1.7).sin();
            flame.scale().set_scalar(f);
            light.set_intensity(0.18 * f);
        });
    }
    // Handle loop on top (torus arch).
    let handle = mesh(pg("CandleLanterns__handle", torus_geo!(0.020, 0.002, 3, 8, PI)), &frame_mat);
    handle.rotation().y = PI / 2.0;
    handle.position().y = 0.149; // hooked ends in the cap's ears (y 0.145–0.150), not 5 mm above
    lantern.add(&handle);
    lantern.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    lantern.rotation().y = rnd() * PI * 2.0;
    g.world.add(&lantern);
}

// R656 hanging horseshoe: 5% chance per level. Wall-mounted U-shape iron shoe
// w/ nail holes.
pub fn place_horseshoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let iron_mat = lambert!({ color: 0x504040 });
    let nail_mat = lambert!({ color: 0x707070 });
    let shoe = group();
    let mount_y = 1.75;
    // Main U-shape via 3/4 torus.
    let arch = mesh(pg("Horseshoes__arch", torus_geo!(0.055, 0.010, 6, 14, PI * 1.3)), &iron_mat);
    arch.rotation().z = -PI * 0.15;
    arch.position().set(0, mount_y, 0.005);
    shoe.add(&arch);
    // 6 nail holes (small dark dots around inside of arch).
    for i in 0..6 {
        // along the web's fuller groove (r ≈ 0.058), clear of the heels and the toe clip;
        // 0.6 mm proud of the web face (z 0.0122). The group is already turned to the room.
        let a = -PI * 0.03 + (i as f64 / 5.0) * PI * 1.06;
        let nail = mesh(circle_geo!(0.004, 5), &nail_mat);
        nail.position().set(a.cos() * 0.058, mount_y + a.sin() * 0.058, 0.0128);
        shoe.add(&nail);
    }
    // Small nail-hanger at very top.
    let hanger = mesh(pg("Horseshoes__hanger", cyl_geo!(0.002, 0.002, 0.010, 4)), &nail_mat);
    // driven into the wall (z -0.008), head (+Y) towards the room; the toe's inner
    // edge (r 0.045) rests on its shank
    hanger.rotation().x = PI / 2.0;
    hanger.position().set(0, mount_y + 0.043, -0.004);
    shoe.add(&hanger);
    shoe.rotation().y = face_yaw(dx, dz);
    shoe.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.008), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.008));
    g.world.add(&shoe);
}

// R657 hanging cast iron pan: 4% chance per level. Round skillet body + long
// handle + hanger loop.
pub fn place_cast_iron_pans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let iron_mat = lambert!({ color: 0x201818 });
    let pan = group();
    let mount_y = 1.80;
    // Round pan body (short cylinder viewed side-on).
    let body = mesh(pg("CastIronPans__body", cyl_geo!(0.09, 0.09, 0.030, 14)), &iron_mat);
    body.rotation().x = PI / 2.0;
    body.position().set(-0.03, mount_y, 0.015);
    pan.add(&body);
    // Pan interior recess (slightly smaller darker disc).
    let inner = mesh(pg("CastIronPans__inner", cyl_geo!(0.076, 0.076, 0.015, 14)), lambert!({ color: 0x101010 }));
    inner.rotation().x = PI / 2.0;
    inner.position().set(-0.03, mount_y, 0.021); // top 1.5 mm below the rim (0.030), not proud of it
    pan.add(&inner);
    // Long handle extending to right.
    let handle = mesh(pg("CastIronPans__handle", box_geo!(0.20, 0.014, 0.014)), &iron_mat);
    handle.position().set(0.115, mount_y, 0.015);
    pan.add(&handle);
    // Hanger hole loop at handle end.
    let lp = mesh(pg("CastIronPans__loop", torus_geo!(0.006, 0.002, 3, 8)), &iron_mat);
    // Upright ring (XY plane, facing the room) hooked through the handle tip (x 0.215).
    lp.position().set(0.220, mount_y, 0.015);
    pan.add(&lp);
    pan.rotation().y = face_yaw(dx, dz);
    pan.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&pan);
}

// R657 wooden staved bucket: 4% chance per level. Barrel-style bucket w/
// vertical staves + metal bands.
pub fn place_wooden_buckets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let band_mat = lambert!({ color: 0x505050 });
    let bucket = group();
    // Main body (slightly tapered cylinder — barrel shape).
    let body = mesh(pg("WoodenBuckets__body", cyl_geo!(0.080, 0.070, 0.14, 12)), &wood_mat);
    body.position().y = 0.070;
    bucket.add(&body);
    // 12 vertical stave lines (darker thin cylinders on outside).
    for i in 0..12 {
        let a = (i as f64 / 12.0) * PI * 2.0;
        let stave = mesh(cyl_geo!(0.0015, 0.0015, 0.14, 3), lambert!({ color: 0x603818 }));
        // follow the body's taper (r ≈ 0.072 at the foot, 0.081 at the rim)
        stave.rotation().y = -a;
        stave.rotation().z = -0.0655;
        stave.position().set(a.cos() * 0.0770, 0.070, a.sin() * 0.0770);
        bucket.add(&stave);
    }
    // 2 metal bands (top + bottom).
    for i in 0..2 {
        let band_y = 0.020 + i as f64 * 0.100;
        let band = mesh(pg("WoodenBuckets__band", torus_geo!(0.079 - i as f64 * 0.005, 0.004, 4, 16)), &band_mat);
        // sized to the body at this height (it widens upward): the bottom band
        // floated off it and the top one was buried
        let band_s = (0.070 + 0.010 * band_y / 0.14 + 0.0015) / (0.079 - i as f64 * 0.005);
        band.scale().set(band_s, band_s, 1);
        band.rotation().x = PI / 2.0;
        band.position().y = band_y;
        bucket.add(&band);
    }
    // Curved wire handle.
    let handle = mesh(pg("WoodenBuckets__handle", torus_geo!(0.085, 0.003, 4, 12, PI)), &band_mat);
    handle.rotation().y = PI / 2.0;
    handle.position().y = 0.138;
    bucket.add(&handle);
    bucket.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    bucket.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bucket);
}

// R658 hanging cast iron kettle: 3% chance per level. Wall-hung tea kettle w/
// spout + handle.
pub fn place_cast_iron_kettles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let iron_mat = lambert!({ color: 0x181818 });
    let kettle = group();
    let mount_y = 1.75;
    // Bulbous body (sphere).
    let body = mesh(pg("CastIronKettles__body", sphere_geo!(0.060, 12, 10)), &iron_mat);
    body.scale().y = 0.85;
    body.position().set(0, mount_y, 0.055);
    kettle.add(&body);
    // Angled spout extending up-forward.
    let spout = mesh(pg("CastIronKettles__spout", cyl_geo!(0.010, 0.014, 0.070, 6)), &iron_mat);
    spout.rotation().z = PI / 3.0; // narrow mouth up and out at -X, wide root in the body
    spout.position().set(-0.055, mount_y + 0.028, 0.060);
    kettle.add(&spout);
    // Lid on top (short disc).
    let lid = mesh(pg("CastIronKettles__lid", cyl_geo!(0.020, 0.024, 0.008, 8)), &iron_mat);
    lid.position().set(0, mount_y + 0.056, 0.055);
    kettle.add(&lid);
    // Lid knob.
    let knob = mesh(pg("CastIronKettles__knob", sphere_geo!(0.005, 5, 4)), &iron_mat);
    knob.position().set(0, mount_y + 0.064, 0.055);
    kettle.add(&knob);
    // Curved handle arch over top.
    let handle = mesh(pg("CastIronKettles__handle", torus_geo!(0.055, 0.004, 4, 10, PI)), &iron_mat);
    handle.rotation().z = 0.0;
    handle.rotation().y = PI / 2.0;
    // ends on the body's shoulders (surface y ≈ +0.020 at 5.5 cm out), arch clears the knob
    handle.position().set(0, mount_y + 0.018, 0.055);
    kettle.add(&handle);
    kettle.rotation().y = face_yaw(dx, dz);
    kettle.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.060), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.060));
    g.world.add(&kettle);
}

// R659 firewood stack: 5% chance per level. 3-5 short logs piled on floor.
pub fn place_firewood_stacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bark_colors: [u32; 4] = [0x6b3f1f, 0x805030, 0x503020, 0x9a6f4b];
    let bark_color = pick(&bark_colors);
    let bark_mat = lambert!({ color: bark_color });
    let core_mat = lambert!({ color: 0xd0a870 });
    let stack = group();
    let count = 3 + (rnd() * 3.0).floor() as i32;
    // Build pyramid stack: bottom row (count logs), then decreasing.
    let layers = 2i32.min((count as f64 / 2.0).floor() as i32);
    // bottom row leaves logs for the rows above (3 → 2+1, 4 → 3+1, 5 → 3+2);
    // it used to take all `count` logs, so no stack ever formed
    let bottom_row = ((count + 1) as f64 / 2.0).ceil() as i32;
    let mut idx = 0;
    for layer in 0..=layers {
        let row_count = bottom_row - layer;
        if row_count <= 0 {
            break;
        }
        for i in 0..row_count {
            // Log body + end grain cap.
            let lg = group();
            let body = mesh(pg("FirewoodStacks__log", cyl_geo!(0.030, 0.030, 0.15, 8)), &bark_mat);
            body.rotation().z = PI / 2.0;
            body.position().y = 0.030;
            lg.add(&body);
            // End cap w/ light-colored core (visible one end).
            let cap = mesh(circle_geo!(0.030, 8), &core_mat);
            cap.rotation().y = PI / 2.0;
            cap.position().set(0.076, 0.030, 0);
            lg.add(&cap);
            // rows centred, 1 mm apart; upper logs sit in the grooves below
            lg.position().set(0.006 * layer as f64, layer as f64 * 0.0517, -0.061 * (row_count - 1) as f64 / 2.0 + i as f64 * 0.061);
            lg.rotation().y = (rnd() - 0.5) * 0.1;
            stack.add(&lg);
            idx += 1;
            if idx >= count {
                break;
            }
        }
        if idx >= count {
            break;
        }
    }
    stack.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    stack.rotation().y = rnd() * PI * 2.0;
    g.world.add(&stack);
}

// R659 wagon wheel (wall): 3% chance per level. Round rim + spokes + hub,
// hanging on wall.
pub fn place_wagon_wheels(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let iron_mat = lambert!({ color: 0x303030 });
    let wheel = group();
    let mount_y = 1.75;
    // Outer iron rim.
    let rim = mesh(pg("WagonWheels__rim", torus_geo!(0.14, 0.010, 5, 20)), &iron_mat);
    rim.position().set(0, mount_y, 0.014);
    wheel.add(&rim);
    // Wooden outer band inside rim.
    let band = mesh(pg("WagonWheels__band", torus_geo!(0.128, 0.014, 4, 20)), &wood_mat);
    band.position().set(0, mount_y, 0.014);
    wheel.add(&band);
    // Central hub.
    let hub = mesh(pg("WagonWheels__hub", cyl_geo!(0.025, 0.025, 0.030, 10)), &wood_mat);
    hub.rotation().x = PI / 2.0;
    hub.position().set(0, mount_y, 0.020);
    wheel.add(&hub);
    // 8 wooden spokes.
    for i in 0..8 {
        let a = (i as f64 / 8.0) * PI * 2.0;
        let spoke = mesh(pg("WagonWheels__spoke", box_geo!(0.008, 0.115, 0.008)), &wood_mat);
        spoke.rotation().z = a - PI / 2.0; // local +Y along the radius (cos a, sin a)
        spoke.position().set(a.cos() * 0.068, mount_y + a.sin() * 0.068, 0.014);
        wheel.add(&spoke);
    }
    wheel.rotation().y = face_yaw(dx, dz);
    wheel.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.018), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.018));
    g.world.add(&wheel);
}

// R660 framed vintage map: 3% chance per level. Antique paper map w/
// continent shapes + compass rose.
game::lazy_static_tex!(
    vintage_map_texture,
    canvas_tex(128, 96, |ctx, w, h| {
        // Aged paper.
        ctx.set_fill_style("#e8d8a8");
        ctx.fill_rect(0, 0, w, h);
        // Sea (bottom half, faint blue).
        ctx.set_fill_style("rgba(100, 140, 180, 0.3)");
        ctx.fill_rect(0, h * 0.5, w, h * 0.5);
        // 3 land-mass blob outlines (irregular).
        ctx.set_fill_style("#c0a870");
        ctx.set_stroke_style("#604020");
        ctx.set_line_width(1);
        for i in 0..3 {
            ctx.begin_path();
            let cx = 20.0 + i as f64 * 40.0;
            let cy = h * 0.3 + rnd() * h * 0.4;
            ctx.move_to(cx, cy);
            for a in 0..8 {
                let angle = (a as f64 / 8.0) * PI * 2.0;
                let r = 15.0 + rnd() * 10.0;
                ctx.line_to(cx + angle.cos() * r, cy + angle.sin() * r);
            }
            ctx.close_path();
            ctx.fill();
            ctx.stroke();
        }
        // Compass rose (top-right corner).
        ctx.set_stroke_style("#403020");
        ctx.set_line_width(1.5);
        let (rx, ry) = (w - 20.0, 16.0);
        ctx.begin_path();
        ctx.move_to(rx, ry - 8.0);
        ctx.line_to(rx, ry + 8.0);
        ctx.move_to(rx - 8.0, ry);
        ctx.line_to(rx + 8.0, ry);
        ctx.move_to(rx - 5.0, ry - 5.0);
        ctx.line_to(rx + 5.0, ry + 5.0);
        ctx.move_to(rx + 5.0, ry - 5.0);
        ctx.line_to(rx - 5.0, ry + 5.0);
        ctx.stroke();
        // Grid latitude lines.
        ctx.set_stroke_style("rgba(80, 60, 30, 0.2)");
        ctx.set_line_width(0.5);
        for i in 1..5 {
            ctx.begin_path();
            ctx.move_to(0, i as f64 * h / 5.0);
            ctx.line_to(w, i as f64 * h / 5.0);
            ctx.move_to(i as f64 * w / 6.0, 0);
            ctx.line_to(i as f64 * w / 6.0, h);
            ctx.stroke();
        }
    })
);

pub fn place_vintage_maps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let frame_mat = lambert!({ color: 0x604020 });
    let paper_mat = lambert!({ map: vintage_map_texture() });
    let map = group();
    let mount_y = 1.65;
    let frame = mesh(pg("VintageMaps__frame", box_geo!(0.30, 0.22, 0.014)), &frame_mat);
    frame.position().y = mount_y;
    map.add(&frame);
    let paper = mesh(plane_geo!(0.26, 0.18), &paper_mat);
    paper.position().set(0, mount_y, 0.009);
    map.add(&paper);
    map.rotation().z = (rnd() - 0.5) * 0.05;
    map.rotation().y = face_yaw(dx, dz);
    map.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    g.world.add(&map);
}

// R660 hearth bellows: 4% chance per level. Accordion pleated body + 2 wooden
// handles.
pub fn place_bellows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0x604020 });
    let leather_mat = lambert!({ color: 0x4a2f1a });
    let brass_mat = lambert!({ color: 0xa08040 });
    let bel = group();
    // 2 wooden paddles (top + bottom).
    for i in 0..2 {
        let paddle = mesh(pg("Bellows__paddle", box_geo!(0.10, 0.008, 0.06)), &wood_mat);
        paddle.position().set(0, 0.015 + i as f64 * 0.030, 0);
        bel.add(&paddle);
    }
    // 4 pleated leather bellows layers between paddles.
    for i in 0..4 {
        let pleat = mesh(pg("Bellows__pleat", box_geo!(0.095, 0.006, 0.055)), &leather_mat);
        pleat.position().set(0, 0.022 + i as f64 * 0.006, 0);
        bel.add(&pleat);
    }
    // 2 wooden handle extensions (rear).
    for i in 0..2 {
        let handle = mesh(pg("Bellows__handle", cyl_geo!(0.008, 0.008, 0.10, 6)), &wood_mat);
        handle.rotation().z = PI / 2.0;
        handle.position().set(0.100, 0.015 + i as f64 * 0.030, 0);
        bel.add(&handle);
    }
    // Brass nozzle at front tip.
    let nozzle = mesh(pg("Bellows__nozzle", cyl_geo!(0.006, 0.010, 0.024, 6)), &brass_mat);
    nozzle.rotation().z = PI / 2.0;
    nozzle.position().set(-0.062, 0.030, 0);
    bel.add(&nozzle);
    bel.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    bel.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bel);
}
