//! `place_water_fountains`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Water fountains: 15% chance per level. Wall-mounted steel unit.
pub(crate) fn place_water_fountains(g: &mut Game, _dist: &[i32]) {
    g.pp.fountain_positions.clear(); // reset per level
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    let steel_mat = lambert!({ color: 0x9a9890 });
    let basin_mat = lambert!({ color: 0x606058 });
    let button_mat = lambert!({ color: 0x2a2822 });
    let spout_mat = lambert!({ color: 0xa0a094 });
    let fountain = group();
    // Main body: 0.28 wide × 0.38 tall × 0.20 deep box protruding from wall.
    let body = mesh(pg("WaterFountains__body", box_geo!(0.28, 0.38, 0.20)), &steel_mat);
    body.position().y = 0.75; // top edge at ~0.94 m
    fountain.add(&body);
    // Basin cavity — small darker box slightly recessed.
    let basin = mesh(pg("WaterFountains__basin", box_geo!(0.22, 0.03, 0.14)), &basin_mat);
    basin.position().set(0, 0.95, 0.03);
    fountain.add(&basin);
    // Spout: small angled cylinder near back of basin.
    let spout = mesh(pg("WaterFountains__spout", cyl_geo!(0.008, 0.012, 0.06, 6)), &spout_mat);
    spout.rotation().x = -PI / 6.0; // pointing up and slightly forward
    spout.position().set(0, 0.98, -0.02);
    fountain.add(&spout);
    // Button on front lower.
    let button = mesh(pg("WaterFountains__button", cyl_geo!(0.02, 0.02, 0.008, 8)), &button_mat);
    button.rotation().x = PI / 2.0;
    button.position().set(0.07, 0.66, 0.104);
    fountain.add(&button);
    // Grille below basin (thin dark rectangle).
    let grille = mesh(plane_geo!(0.20, 0.03), &basin_mat);
    grille.position().set(0, 0.93, 0.101);
    fountain.add(&grille);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.10);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.10);
    if c.dx.abs() == 1 {
        fountain.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        fountain.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 1.0);
    } else {
        fountain.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        fountain.position().set(wall_x + (rnd() - 0.5) * 1.0, 0, wall_z);
    }
    g.world.add(&fountain);
    // Track for drip-audio events.
    let fp = fountain.pos();
    g.pp.fountain_positions.push(v3(fp.x, 0.94, fp.z));
    // Fountain leak stain: 40% chance a wet stain on the floor below.
    if rnd() < 0.40 {
        let r = 0.35 + rnd() * 0.20;
        let stain_geo = circle_geo!(r, 12);
        // Slight stretch for organic irregular pool.
        let sx = 1.0 + rnd() * 0.3;
        let sy = 1.0 - rnd() * 0.2;
        stain_geo.scale(sx, sy, 1);
        let stain_mat = basic_mat!({ color: 0x0e1a20, transparent: true, opacity: 0.55, depthWrite: false });
        let stain = mesh(stain_geo, &stain_mat);
        stain.rotation().x = -PI / 2.0;
        stain.rotation().z = rnd() * PI * 2.0;
        // Position: slightly in front of fountain along the wall-normal.
        let fry = fountain.rot().y;
        let fx = -fry.sin(); // fountain-front direction
        let fz = -fry.cos();
        stain.position().set(fp.x + fx * 0.25, 0.004, fp.z + fz * 0.25);
        g.world.add(&stain);
    }
}
