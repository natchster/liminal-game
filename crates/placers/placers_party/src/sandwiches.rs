//! `place_sandwiches`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Half-eaten sandwich on plate (floor): 6% chance per level.
pub(crate) fn place_sandwiches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let plate_colors = [0xffffff, 0xf0e8d8, 0xe8e0c8, 0xd0d0e0];
    let plate_mat = lambert!({ color: pick(&plate_colors) });
    // Bread color pool.
    let bread_colors = [0xd0a060, 0xa06030, 0xe0c088, 0xc09858];
    let bread_mat = lambert!({ color: pick(&bread_colors) });
    // Filling color pool — cheese/ham/turkey tones.
    let filling_colors = [0xf0d040, 0xe0a080, 0xd0c0a0, 0xc06040, 0x804020];
    let filling_mat = lambert!({ color: pick(&filling_colors) });
    let s = group();
    // Plate — shallow disc + torus rim.
    let plate = mesh(pg("Sandwiches__plate", cyl_geo!(0.10, 0.092, 0.006, 16)), &plate_mat);
    plate.position().y = 0.003;
    s.add(&plate);
    let plate_rim = mesh(pg("Sandwiches__rim", torus_geo!(0.095, 0.004, 4, 16)), &plate_mat);
    plate_rim.rotation().x = PI / 2.0;
    plate_rim.position().y = 0.008;
    s.add(&plate_rim);
    // Sandwich cut diagonally — bottom bread slice (half-square).
    let (half_w, half_h) = (0.08, 0.020);
    let bottom_bread = mesh(pg("Sandwiches__bread", box_geo!(half_w, half_h, half_w)), &bread_mat);
    bottom_bread.position().y = 0.006 + half_h / 2.0;
    bottom_bread.rotation().y = PI / 4.0;
    s.add(&bottom_bread);
    // Filling layer (thinner).
    let filling = mesh(pg("Sandwiches__filling", box_geo!(half_w * 0.98, 0.008, half_w * 0.98)), &filling_mat);
    filling.position().y = 0.006 + half_h + 0.004;
    filling.rotation().y = PI / 4.0;
    s.add(&filling);
    // Optional lettuce — 70% chance.
    if rnd() < 0.7 {
        let lettuce_mat = lambert!({ color: 0x60a040 });
        // 3 small ruffled edges peeking out (thin boxes).
        for i in 0..3 {
            let ruffle = mesh(pg("Sandwiches__lettuce", box_geo!(0.024, 0.004, 0.010)), &lettuce_mat);
            let ang = (i as f64 / 3.0) * PI * 2.0 + PI / 4.0;
            ruffle.position().set(ang.cos() * 0.04, 0.006 + half_h + 0.008, ang.sin() * 0.04);
            ruffle.rotation().y = ang + PI / 2.0;
            ruffle.rotation().z = 0.2;
            s.add(&ruffle);
        }
    }
    // Top bread slice.
    let top_bread = mesh(pg("Sandwiches__bread", box_geo!(half_w, half_h, half_w)), &bread_mat);
    top_bread.position().y = 0.006 + half_h + 0.008 + half_h / 2.0;
    top_bread.rotation().y = PI / 4.0;
    s.add(&top_bread);
    // Bite gap — a slightly darker bread interior wedge.
    let bite_mat = lambert!({ color: 0xa08050 });
    let bite_inner = mesh(pg("Sandwiches__bite", box_geo!(0.030, half_h * 2.0 + 0.008, 0.030)), &bite_mat);
    // Set into the sandwich edge (3 mm proud) and standing on the plate, flush
    // with the top slice.
    bite_inner.position().set(half_w * 0.25, 0.0305, half_w * 0.25);
    bite_inner.rotation().y = PI / 4.0;
    s.add(&bite_inner);
    // Bread crumbs — 6-10 scattered around plate.
    let crumb_count = 6 + (rnd() * 5.0).floor() as i32;
    for _i in 0..crumb_count {
        let c_size = 0.003 + rnd() * 0.004;
        let crumb = mesh(pg("Sandwiches__crumb", box_geo!(c_size, c_size, c_size)), &bread_mat);
        let ang = rnd() * PI * 2.0;
        // Resting on the plate's flat, inside the rim.
        let rr = 0.05 + rnd() * 0.035;
        crumb.position().set(ang.cos() * rr, 0.006 + c_size / 2.0, ang.sin() * rr);
        crumb.rotation().y = rnd() * PI * 2.0;
        s.add(&crumb);
    }
    s.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    s.rotation().y = rnd() * PI * 2.0;
    g.world.add(&s);
}
