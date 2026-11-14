//! `place_small_pool_balls`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open_tile;
use placers_kit::place::scatter_xz;

// Small pool ball: 8% chance per level. Colored sphere
// with white numbered circle stripe. Games family.
pub(crate) fn place_small_pool_balls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let ball_colors = [0xf0e050, 0x2050d8, 0xd82040, 0xa020a0, 0xe07020, 0x30a030, 0x8a2020, 0x201a10, 0xf0ece0];
    let ball_color = pick_color(&ball_colors);
    let ball_mat = lambert!({ color: ball_color });
    let white_mat = lambert!({ color: 0xf0ece0 });
    let num_mat = lambert!({ color: 0x101008 });
    let is_striped = rnd() < 0.40;
    let ball = group();
    let r = 0.028;
    let body = mesh(pg("SmallPoolBalls__ball", sphere_geo!(r, 12, 10)), &ball_mat);
    body.position().y = r;
    ball.add(&body);
    if is_striped {
        // White band across equator for striped ball.
        let stripe = mesh(pg("SmallPoolBalls__stripe", cyl_geo!(r * 1.001, r * 1.001, r * 0.6, 12, 1, true)), &white_mat);
        stripe.position().y = r;
        ball.add(&stripe);
    }
    // White circle patch on top with number spot.
    let patch = mesh(circle_geo!(r * 0.35, 12), &white_mat);
    patch.rotation().x = -PI / 2.0;
    patch.position().y = r * 2.0 + 0.0005;
    ball.add(&patch);
    let num = mesh(circle_geo!(r * 0.10, 8), &num_mat);
    num.rotation().x = -PI / 2.0;
    num.position().y = r * 2.0 + 0.0010;
    ball.add(&num);
    scatter_xz(g, &ball, x, z, 2.4);
    ball.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ball);
}
