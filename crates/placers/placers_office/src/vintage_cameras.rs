//! `place_vintage_cameras`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Vintage camera: 10% chance per level.
pub(crate) fn place_vintage_cameras(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
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
    // 25% chance: small film canister cylinder near camera.
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
        canister.position().set(cp.x + c_ang.dcos() * c_rad, 0, cp.z + c_ang.dsin() * c_rad);
        canister.rotation().y = rnd() * PI * 2.0;
        g.world.add(&canister);
    }
}
