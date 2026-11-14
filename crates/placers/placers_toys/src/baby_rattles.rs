//! `place_baby_rattles`: Baby rattle: 6% chance per level. A hard plastic baby rattle of the 1960s, 12 cm: a fluted pastel ball moulded in two halves (the beads loose in it), a cream handle with the ring a small hand holds, a cap over the mould's gate; lying on the ball and the ring. Its pastel faded, a few printed stars rubbed half away, the cream gone yellow, grimed.
use crate::shared::wear_at;
use crate::shared::wear_tex;
use crate::shared::Wear;
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put_world;
use placers_kit::noise::fbmp;
use placers_kit::noise::mix;
use placers_kit::noise::sstep as ss;
use placers_kit::paint::paint_tile;
use placers_kit::paint::rgb;
use placers_kit::place::index_of;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_at;

/// Baby rattle: 6% chance per level. A hard plastic baby rattle of the
/// 1960s, 12 cm: a fluted pastel ball moulded in two halves (the beads loose
/// in it), a cream handle with the ring a small hand holds, a cap over the
/// mould's gate; lying on the ball and the ring. Its pastel faded, a few
/// printed stars rubbed half away, the cream gone yellow, grimed.
pub(crate) fn place_baby_rattles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old pastels, faded
    let rattle_colors: [u32; 4] = [0xf0c0c0, 0xc0c0f0, 0xf0e0a0, 0xa0e0c0];
    let k = index_of(&rattle_colors, pick(&rattle_colors));
    let pastel = rgb([0xc9a29c, 0x9fa4c2, 0xcab98c, 0x8fb8a2][k]);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt_at(px, pz);
    let rattle = group();
    let ball = Wear { base: pastel, alt: mix(pastel, [0.92, 0.9, 0.86], 0.4), rub: 0.0, grime: 0.4, scuff: 0.45, mottle: 0.07, scales: 0 };
    let ball_tex = paint_tile(128, move |u, v| {
        let mut c = wear_at(salt ^ 11, u, v, &ball);
        // the printed stars, rubbed half away (two to a tile, 9 mm across)
        for k in 0..2 {
            let (cx, cy) = (0.25 + 0.5 * f64::from(k), 0.3 + 0.4 * f64::from(k));
            let ink = star(u - cx, v - cy, 0.075, 0.4 * f64::from(k)) * ss(0.3, 0.42, fbmp(salt ^ 13, u, v, 6, 3));
            c = mix(c, [0.93, 0.91, 0.86], ink * 0.85);
        }
        c
    });
    put_world(&rattle, "Real__babyRattleBall", &phong!({ map: &ball_tex, specular: 0x262626, shininess: 34 }), 0.06);
    let cream = rgb(0xd2c7aa);
    let handle = Wear { base: cream, alt: mix(cream, [0.6, 0.5, 0.3], 0.3), rub: 0.0, grime: 0.5, scuff: 0.4, mottle: 0.08, scales: 0 };
    put_world(
        &rattle,
        "Real__babyRattleHandle",
        &phong!({ map: &wear_tex(salt ^ 12, 128, handle), specular: 0x222222, shininess: 30 }),
        0.05,
    );
    rattle.position().set(px, 0, pz);
    rattle.rotation().y = turn;
    g.world.add(&rattle);
}

/// How much of a five-pointed star (`r` to its points, turned `turn`) covers
/// the point (x, y) from its centre: 1 inside, 0 outside, soft at its edge
fn star(x: f64, y: f64, r: f64, turn: f64) -> f64 {
    let sector = PI * 2.0 / 5.0;
    let d = (x * x + y * y).sqrt();
    let mut phi = (y.atan2(x) + turn).rem_euclid(sector);
    if phi > sector / 2.0 {
        phi = sector - phi;
    }
    // the edge from a point (r, 0) to the notch beside it (0.4 r at 36 degrees)
    let (p0x, p0y) = (r, 0.0);
    let (p1x, p1y) = (0.4 * r * (PI / 5.0).cos(), 0.4 * r * (PI / 5.0).sin());
    let (px, py) = (d * phi.cos(), d * phi.sin());
    let (ex, ey) = (p1x - p0x, p1y - p0y);
    let side = (ex * (py - p0y) - ey * (px - p0x)) / (ex * ex + ey * ey).sqrt();
    ss(-0.004, 0.004, side)
}
