//! `place_standing_mirrors`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small mirror on stand w/ optional smudge (floor):
// 6% chance per level. Vanity/table mirror on hinged frame
// w/ 50% chance of visible fingerprint/lipstick smudge.
pub(crate) fn place_standing_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Frame color pool.
    let frame_colors: [u32; 5] = [0xd0a040, 0x606060, 0xf0e0d0, 0xa04060, 0x8b5a2b];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    // Mirror glass — bluish-tinted reflective look.
    let mirror_mat = basic_mat!({ color: 0xd8e8f0 });
    let back_mat = lambert!({ color: 0x302030 });
    let mir = group();
    // Base — wide flat oval disc.
    let base = mesh(pg("StandingMirrors__base", cyl_geo!(0.055, 0.065, 0.010, 16)), &frame_mat);
    base.position().y = 0.005;
    mir.add(&base);
    // Vertical stem (thin cylinder up from base).
    let stem = mesh(pg("StandingMirrors__stem", cyl_geo!(0.005, 0.005, 0.070, 6)), &frame_mat);
    // y 0.008-0.058: from inside the base up into the tilted frame's bottom
    // rim (tube bottom y 0.055, z 0.008), below the glass and back plate.
    stem.scale().y = 0.714;
    stem.position().set(0, 0.033, 0.006);
    mir.add(&stem);
    // Mirror oval frame — thin torus tilted slightly.
    let tilt = -0.15;
    let frame_outer = mesh(pg("StandingMirrors__frame", torus_geo!(0.055, 0.006, 5, 20)), &frame_mat);
    frame_outer.rotation().x = tilt;
    frame_outer.position().y = 0.115;
    mir.add(&frame_outer);
    // Mirror face — thin disc, oval-shape via Y-scale.
    let mirror = mesh(circle_geo!(0.053, 20), &mirror_mat);
    mirror.rotation().x = tilt;
    mirror.position().y = 0.115;
    mirror.position().z = 0.002;
    mir.add(&mirror);
    // Back plate — dark oval behind.
    let back = mesh(pg("StandingMirrors__back", circle_geo!(0.055, 20)), &back_mat);
    back.rotation().x = tilt;
    back.rotation().y = PI;
    back.position().y = 0.115;
    back.position().z = -0.002;
    mir.add(&back);
    // 2 hinge pegs (small dots on frame sides).
    for side in [-1.0, 1.0] {
        let peg = mesh(pg("StandingMirrors__peg", cyl_geo!(0.005, 0.005, 0.010, 5)), &frame_mat);
        peg.rotation().z = PI / 2.0;
        peg.position().set(side * 0.062, 0.115, 0);
        mir.add(&peg);
    }
    // 50% chance: smudge on mirror surface.
    if rnd() < 0.5 {
        // Smudge color pool: lipstick (red), fingerprint (dark), foundation (tan).
        let smudge_colors: [u32; 4] = [0xa03040, 0x604020, 0xc0a080, 0x805050];
        let smudge_mat = basic_mat!({ color: pick(&smudge_colors), transparent: true, opacity: 0.55 });
        // 3-6 elliptical smudge blobs scattered on mirror face.
        let blob_count = 3 + (rnd() * 4.0).floor() as i32;
        for _i in 0..blob_count {
            let rr = 0.010 + rnd() * 0.010;
            let blob = mesh(circle_geo!(rr, 6), &smudge_mat);
            let ang = rnd() * PI * 2.0;
            let dd = rnd() * 0.030;
            blob.position().set(ang.dcos() * dd, 0, ang.dsin() * dd);
            blob.rotation().x = tilt;
            // Offset slightly above mirror surface.
            let mirror_pos = v3(ang.dcos() * dd, 0.115, 0.004 + ang.dsin() * dd);
            blob.position().copy(&mirror_pos);
            blob.rotation().x = tilt;
            // Random rotation for organic look.
            blob.rotation().z = rnd() * PI * 2.0;
            mir.add(&blob);
        }
    }
    mir.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    mir.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mir);
}
