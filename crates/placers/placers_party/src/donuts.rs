//! `place_donuts`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Donut on napkin: 8% chance per level. Glazed torus
// donut resting on a folded paper napkin square.
pub(crate) fn place_donuts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Napkin — pastel color pool.
    let napkin_colors: [u32; 5] = [0xf8f8f0, 0xf0e8d8, 0xf8e0e0, 0xe0f0e8, 0xf0e0f0];
    let napkin_mat = lambert!({ color: pick(&napkin_colors) });
    // Donut base — golden fried.
    let dough_mat = lambert!({ color: 0xc89050 });
    // Glaze — 5-color pool (glazed, chocolate, strawberry, matcha, maple).
    let glaze_colors: [u32; 5] = [0xf0e8c0, 0x603020, 0xd0a0a8, 0xa0c090, 0xa07030];
    let glaze_mat = lambert!({ color: pick(&glaze_colors) });
    let donut = group();
    // Napkin square (thin box).
    let napkin = mesh(pg("Donuts__napkin", box_geo!(0.20, 0.003, 0.20)), &napkin_mat);
    napkin.position().y = 0.0015;
    napkin.rotation().y = (rnd() - 0.5) * 0.3;
    donut.add(&napkin);
    // Folded corner detail (small triangle box tilted).
    let fold_geom = pg("Donuts__fold", box_geo!(0.06, 0.005, 0.06));
    let fold = mesh(fold_geom, &napkin_mat);
    // Lies on the napkin with its +X+Z tip on the napkin's corner, turned
    // with the napkin; the modelled corner curls up by itself (the old 0.4 rad
    // tilt sank one side 7 mm into the floor and it overhung the corner 1 cm).
    let nry = napkin.rot().y;
    let (nc, ns) = (nry.dcos(), nry.dsin());
    fold.position().set(0.07 * (nc + ns), 0.0053, 0.07 * (nc - ns));
    fold.rotation().y = nry;
    donut.add(&fold);
    // Donut dough (torus).
    let dough = mesh(pg("Donuts__dough", torus_geo!(0.045, 0.020, 6, 14)), &dough_mat);
    dough.rotation().x = PI / 2.0;
    dough.position().y = 0.023;
    donut.add(&dough);
    // Glaze top (slightly larger torus, thinner, glaze color).
    let glaze = mesh(pg("Donuts__glaze", torus_geo!(0.045, 0.018, 6, 14)), &glaze_mat);
    glaze.rotation().x = PI / 2.0;
    glaze.position().y = 0.033;
    donut.add(&glaze);
    // Optional sprinkles — 55% chance: 6-10 tiny colored boxes.
    if rnd() < 0.55 {
        let sprinkle_colors: [u32; 6] = [0xff4040, 0x40a0ff, 0xffe040, 0x40e080, 0xff80c0, 0xffffff];
        let count = 6 + (rnd() * 5.0).floor() as i32;
        for _i in 0..count {
            let s_mat = lambert!({ color: pick(&sprinkle_colors) });
            let s = mesh(pg("Donuts__sprinkle", box_geo!(0.008, 0.002, 0.002)), &s_mat);
            let ang = rnd() * PI * 2.0;
            let rr = 0.045 + (rnd() - 0.5) * 0.010;
            // On the glaze's top surface at this radius (at 0.041 they were 8-10 mm
            // inside the icing).
            s.position().set(ang.dcos() * rr, 0.0334 + (0.018 * 0.018 - (rr - 0.045).dpowi(2)).sqrt(), ang.dsin() * rr);
            s.rotation().y = rnd() * PI * 2.0;
            donut.add(&s);
        }
    }
    donut.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    donut.rotation().y = rnd() * PI * 2.0;
    g.world.add(&donut);
}
