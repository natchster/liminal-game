//! `place_dice_clusters`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Dice cluster: 6% chance per level. 2-4 six-sided cubes w/ pip decals.
pub(crate) fn place_dice_clusters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let die_colors = [0xf8f4e8, 0xc02020, 0x202020, 0x40a0e0];
    let die_color = choice(&die_colors);
    let cluster = group();
    let count = 2 + (rnd() * 3.0).floor() as i32;
    for _i in 0..count {
        let die_mat = lambert!({ color: die_color });
        let s = 0.020;
        let die = mesh(pg("DiceClusters__die", box_geo!(s, s, s)), &die_mat);
        die.position().set((rnd() - 0.5) * 0.10, s / 2.0, (rnd() - 0.5) * 0.10);
        // Rest flat on a face: x flips 0/pi and z quarter turns keep the cube axis-aligned.
        die.rotation().set(if rnd() < 0.5 { 0.0 } else { PI }, rnd() * PI, (rnd() * 4.0).floor() * PI / 2.0);
        cluster.add(&die);
        // Top pip decal, square with the die's top face (its yaw, mirrored when flipped over x).
        let pip_decal = mesh(plane_geo!(s * 0.9, s * 0.9), lambert!({ map: die_face_texture(), transparent: true }));
        let dr = die.rot();
        pip_decal.rotation().set(-PI / 2.0, 0, if dr.x != 0.0 { -dr.y } else { dr.y });
        pip_decal.position().copy(&die.pos());
        pip_decal.position().y += s / 2.0 + 0.001;
        cluster.add(&pip_decal);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// `die_face_texture`: one random die face (1-6 pips).
game::lazy_static_tex!(
    die_face_texture,
    canvas_tex(64, 64, |ctx, w, h| {
        ctx.set_fill_style("#f8f4e8");
        ctx.fill_rect(0, 0, w, h);
        // Pick a random face 1-6 for display purposes.
        let pips = 1 + (rnd() * 6.0).floor() as usize;
        ctx.set_fill_style("#202020");
        let positions: [&[(f64, f64)]; 6] = [
            &[(0.5, 0.5)],
            &[(0.25, 0.25), (0.75, 0.75)],
            &[(0.25, 0.25), (0.5, 0.5), (0.75, 0.75)],
            &[(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)],
            &[(0.25, 0.25), (0.75, 0.25), (0.5, 0.5), (0.25, 0.75), (0.75, 0.75)],
            &[(0.25, 0.25), (0.75, 0.25), (0.25, 0.5), (0.75, 0.5), (0.25, 0.75), (0.75, 0.75)],
        ];
        for &(px, py) in positions[pips - 1] {
            ctx.begin_path();
            ctx.arc(w * px, h * py, 5, 0, PI * 2.0);
            ctx.fill();
        }
    })
);
