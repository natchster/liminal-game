//! `place_pencil_eyeliners`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Pencil eyeliner: 9% chance per level. Slim tapered pencil.
pub(crate) fn place_pencil_eyeliners(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors: [u32; 4] = [0x202020, 0x604060, 0x804020, 0x2060a0];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let tip_mat = lambert!({ color: 0x101010 });
    let pen = group();
    let body = mesh(pg("PencilEyeliners__body", cyl_geo!(0.004, 0.004, 0.075, 6)), &body_mat);
    body.rotation().z = PI / 2.0;
    body.position().y = 0.004;
    pen.add(&body);
    let tip = mesh(pg("PencilEyeliners__tip", cone_geo!(0.004, 0.014, 5)), &tip_mat);
    tip.rotation().z = -PI / 2.0;
    tip.position().set(0.044, 0.004, 0);
    pen.add(&tip);
    let cap = mesh(pg("PencilEyeliners__cap", cyl_geo!(0.005, 0.005, 0.020, 6)), &body_mat);
    cap.rotation().z = PI / 2.0;
    cap.position().set(-0.046, 0.005, 0); // r 5 mm: rests on the floor
    pen.add(&cap);
    pen.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pen.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pen);
}
