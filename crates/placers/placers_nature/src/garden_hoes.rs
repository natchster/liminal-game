//! `place_garden_hoes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Garden hoe: 4% chance per level. Long wooden handle + metal flat blade.
pub(crate) fn place_garden_hoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0xa07040 });
    let head_mat = lambert!({ color: 0x505050 });
    let hoe = group();
    // Long handle (laid flat).
    let handle = mesh(pg("GardenHoes__handle", cyl_geo!(0.012, 0.012, 0.55, 8)), &wood_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().y = 0.012;
    hoe.add(&handle);
    // Metal collar at head end.
    let collar = mesh(pg("GardenHoes__collar", cyl_geo!(0.014, 0.014, 0.020, 8)), &head_mat);
    collar.rotation().z = PI / 2.0;
    collar.position().set(-0.265, 0.012, 0);
    hoe.add(&collar);
    // Flat blade head (perpendicular to handle).
    let head = mesh(pg("GardenHoes__head", box_geo!(0.016, 0.006, 0.070)), &head_mat);
    // Socket side 1 mm into the collar's end (-0.275), lying on the floor.
    head.position().set(-0.282, 0.003, 0);
    hoe.add(&head);
    // Small end cap on handle other end.
    let cap = mesh(pg("GardenHoes__cap", sphere_geo!(0.014, 6, 5)), &wood_mat);
    cap.position().set(0.278, 0.012, 0);
    hoe.add(&cap);
    hoe.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    hoe.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hoe);
}
