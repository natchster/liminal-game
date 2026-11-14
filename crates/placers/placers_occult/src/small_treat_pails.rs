//! `place_small_treat_pails`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_tile;

// Small trick-or-treat pail: 6% chance per level. Orange
// plastic pumpkin-shaped candy pail. Halloween family.
pub(crate) fn place_small_treat_pails(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let pump_mat = lambert!({ color: 0xd87020 });
    let stem_mat = lambert!({ color: 0x306018 });
    let handle_mat = lambert!({ color: 0x101010 });
    let face_mat = basic_mat!({ color: 0x181008 });
    let pail = group();
    // Body: flattened sphere (pumpkin shape).
    let body = mesh(pg("SmallTreatPails__body", sphere_geo!(0.070, 14, 10)), &pump_mat);
    body.scale().set(1, 0.75, 1);
    body.position().y = 0.055;
    pail.add(&body);
    // Rim (dark cylinder at top for candy opening).
    let rim = mesh(pg("SmallTreatPails__rim", cyl_geo!(0.048, 0.048, 0.015, 12)), &pump_mat);
    rim.position().y = 0.108;
    pail.add(&rim);
    // Small stem.
    let stem = mesh(pg("SmallPumpkins__stem", cyl_geo!(0.008, 0.010, 0.014, 6)), &stem_mat);
    stem.position().y = 0.122;
    pail.add(&stem);
    // Handle arc (black plastic).
    let handle = mesh(pg("SmallTreatPails__handle", torus_geo!(0.055, 0.004, 4, 12, PI)), &handle_mat);
    // Upright arch over the opening; narrowed so its pivot ends meet the
    // rim's sides (r 0.048) at the rim's mid-height.
    handle.scale().x = 0.052 / 0.055;
    handle.position().y = 0.108;
    pail.add(&handle);
    // Face on body: each decal 1 mm outside the (0.070, 0.0525, 0.070)
    // ellipsoid (the lobed body stays inside it), facing along its normal.
    let on_body = |m: &Obj, px: f64, py: f64| {
        let zs = 0.070 * (1.0 - (px / 0.070).powi(2) - ((py - 0.055) / 0.0525).powi(2)).sqrt();
        let mut n = v3(px / 0.0049, (py - 0.055) / 0.00275625, zs / 0.0049);
        n.normalize();
        m.position().set(px, py, zs).add_scaled_vector(&n, 0.001);
        let mut tgt = m.pos();
        tgt.add(&n);
        m.look_at(tgt.x, tgt.y, tgt.z);
    };
    let eye1 = mesh(circle_geo!(0.010, 3), &face_mat);
    on_body(&eye1, -0.023, 0.072);
    pail.add(&eye1);
    let eye2 = mesh(circle_geo!(0.010, 3), &face_mat);
    on_body(&eye2, 0.023, 0.072);
    pail.add(&eye2);
    let mouth = mesh(plane_geo!(0.040, 0.014), &face_mat);
    on_body(&mouth, 0.0, 0.040);
    pail.add(&mouth);
    pail.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pail.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pail);
}
