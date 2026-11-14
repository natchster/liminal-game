//! `place_small_globes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small vintage globe: 5% chance per level, slow spin.
pub(crate) fn place_small_globes(g: &mut Game, _dist: &[i32]) {
    g.pp.globe_positions.clear(); // reset per level
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
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
    // Track for wood creak audio.
    let gp = globe.pos();
    g.pp.globe_positions.push(v3(gp.x, 0.12, gp.z));
}

// Small vintage globe texture.
game::lazy_static_tex!(
    globe_texture,
    canvas_tex(256, 128, |ctx, w, h| {
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
                ctx.arc(cx + a.dcos() * r * 1.4, cy + a.dsin() * r * 1.4, r * 0.8, 0, PI * 2.0);
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
    })
);
