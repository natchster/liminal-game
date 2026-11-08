//! p23, first half: reliquary / stained glass / crystal skull / portrait /
//! crucifix / owl / candlestick / skulls / potion / candy corn / treat pail.
use crate::*;
use game::prelude::*;

mod t01;
pub use t01::place_ancient_portraits;
// the glass skull, the owl, the candlestick and the skulls stacked: batch
// `t09`'s (`mystic/t09.rs`)
mod t09;
pub use t09::{place_brass_candlesticks, place_crystal_skulls, place_skull_stacks, place_small_owls};
// the reliquary and the crucifix: batch `t19`'s (`mystic/t19.rs`)
mod t19;
pub use t19::{place_reliquary_boxes, place_wall_crucifixes};

// the stained glass panel: batch `t21`'s (`mystic/t21.rs`)
mod t21;
pub use t21::place_stained_glass_windows;

/// The wall-hung panel placement shared by the window / portrait / crucifix:
/// returns nothing, sets rotation.y and position (one draw for the slide).
fn hang_on_wall(g: &Game, o: &Obj, c: &WallCand, inset: f64, center_y: f64) {
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - inset);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - inset);
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        o.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        o.position().set(wall_x + (rnd() - 0.5) * 0.6, center_y, wall_z);
    }
}

// Ancient portrait texture: dark oil painting w/ figure silhouette.
game::lazy_static_tex!(
    portrait_texture,
    canvas_tex(192, 256, |ctx, w, h| {
        // Dark background wash.
        let grad = ctx.create_radial_gradient(w / 2.0, h / 3.0, 20, w / 2.0, h / 2.0, 180);
        grad.add_color_stop(0, "#4a3820");
        grad.add_color_stop(1, "#181008");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Figure silhouette: shoulders + head.
        ctx.set_fill_style("#1a1008");
        // Shoulders (trapezoid).
        ctx.begin_path();
        ctx.move_to(w * 0.15, h * 0.75);
        ctx.line_to(w * 0.85, h * 0.75);
        ctx.line_to(w * 0.75, h * 0.55);
        ctx.line_to(w * 0.25, h * 0.55);
        ctx.close_path();
        ctx.fill();
        // Head (ellipse).
        ctx.begin_path();
        ctx.ellipse(w / 2.0, h * 0.42, w * 0.15, h * 0.15, 0, 0, PI * 2.0);
        ctx.fill();
        // Faint face features.
        ctx.set_stroke_style("rgba(40,25,15,0.7)");
        ctx.set_line_width(1);
        // Eyes (hollow dark spots).
        ctx.set_fill_style("#050300");
        ctx.begin_path();
        ctx.arc(w / 2.0 - 14.0, h * 0.42, 3, 0, PI * 2.0);
        ctx.arc(w / 2.0 + 14.0, h * 0.42, 3, 0, PI * 2.0);
        ctx.fill();
        // Age craquelure.
        ctx.set_stroke_style("rgba(50,35,20,0.4)");
        for _ in 0..30 {
            ctx.begin_path();
            let sx = rnd() * w;
            let sy = rnd() * h;
            ctx.move_to(sx, sy);
            ctx.line_to(sx + rnd() * 30.0 - 15.0, sy + rnd() * 30.0 - 15.0);
            ctx.stroke();
        }
    })
);

// Small skull: 5% chance per level. Bone skull with 2 eye
// sockets + nose gap + teeth. Halloween family.
pub fn place_small_skulls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let bone_mat = lambert!({ color: 0xe8dcc0 });
    let socket_mat = basic_mat!({ color: 0x080604 });
    let skull = group();
    // Cranium (main head sphere).
    let cranium = mesh(pg("SmallSkulls__cranium", sphere_geo!(0.045, 12, 10)), &bone_mat);
    cranium.scale().set(1, 1, 1.1);
    cranium.position().y = 0.045;
    skull.add(&cranium);
    // Jaw (smaller flatter cylinder attached below cranium).
    let jaw = mesh(pg("SmallSkulls__jaw", box_geo!(0.055, 0.014, 0.040)), &bone_mat);
    jaw.position().set(0, 0.014, 0.010);
    skull.add(&jaw);
    // 2 eye sockets (dark discs on front).
    for s in [-1.0, 1.0] {
        let socket = mesh(circle_geo!(0.010, 8), &socket_mat);
        socket.position().set(s * 0.014, 0.052, 0.040);
        skull.add(&socket);
    }
    // Nose triangle (small dark).
    let nose_geo = tri_geo([-0.006, 0.038, 0.045, 0.006, 0.038, 0.045, 0.0, 0.024, 0.045]);
    let nose = mesh(nose_geo, &socket_mat);
    skull.add(&nose);
    // Teeth: small bump grid (5 bumps).
    // Set on the jaw's top edge (0.021), following its horseshoe front
    // (z 0.0242 at the chin curving back to ~0.019 at x ±0.014), each tooth
    // turned to face out of the curve.
    for i in 0..5 {
        let tooth = mesh(pg("SmallSkulls__tooth", box_geo!(0.005, 0.006, 0.002)), &bone_mat);
        let tx = -0.014 + i as f64 * 0.007;
        tooth.position().set(tx, 0.0225, 0.0242 - 0.00545 * (tx / 0.0148).powi(2));
        tooth.rotation().y = (2.0 * 0.00545 * tx / (0.0148 * 0.0148)).atan();
        skull.add(&tooth);
    }
    skull.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    skull.rotation().y = rnd() * PI * 2.0;
    // Tilted to the side sometimes.
    if rnd() < 0.30 {
        let sgn = if rnd() < 0.5 { 1.0 } else { -1.0 };
        let tilt = sgn * (PI / 6.0 + rnd() * PI / 4.0);
        skull.rotation().z = tilt;
    }
    g.world.add(&skull);
}

// Small potion bottle: 8% chance per level. Glass vial
// with colored liquid + cork stopper. Halloween/mystical family.
pub fn place_small_potions(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let glass_mat = basic_mat!({ color: 0xc0e0d0, transparent: true, opacity: 0.35 });
    let potion_colors: [u32; 5] = [0x30d040, 0xa020d0, 0xd82040, 0x2050d8, 0xd8b040];
    let potion_color = pick_u32(&potion_colors);
    let potion_mat = basic_mat!({ color: potion_color, transparent: true, opacity: 0.85 });
    let cork_mat = lambert!({ color: 0xa87050 });
    let potion = group();
    // Body: round-shouldered bottle.
    let body = mesh(sphere_geo!(0.024, 10, 8, 0, PI * 2.0, 0.4, PI - 0.4), &glass_mat);
    body.position().y = 0.028;
    potion.add(&body);
    // Neck.
    let neck = mesh(cyl_geo!(0.008, 0.008, 0.020, 6), &glass_mat);
    neck.position().y = 0.056;
    potion.add(&neck);
    // Cork.
    let cork = mesh(pg("SmallPotions__cork", cyl_geo!(0.007, 0.007, 0.010, 6)), &cork_mat);
    cork.position().y = 0.070;
    potion.add(&cork);
    // Colored liquid inside (smaller sphere).
    let liquid = mesh(sphere_geo!(0.018, 8, 6), &potion_mat);
    liquid.position().y = 0.026;
    liquid.scale().y = 0.7;
    potion.add(&liquid);
    // 15% chance tipped.
    if rnd() < 0.15 {
        potion.rotation().z = PI / 2.0;
        potion.position().y = 0.018;
    }
    potion.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    potion.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    potion.rotation().y = rnd() * PI * 2.0;
    g.world.add(&potion);
}

// Candy corn scatter: batch `t22`'s (`mystic/t22.rs`)
mod t22;
pub use t22::place_candy_corn_scatters;

// Small trick-or-treat pail: 6% chance per level. Orange
// plastic pumpkin-shaped candy pail. Halloween family.
pub fn place_small_treat_pails(g: &mut Game, _dist: &[i32]) {
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
