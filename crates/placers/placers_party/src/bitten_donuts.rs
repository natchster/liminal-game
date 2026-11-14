//! `place_bitten_donuts`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Donut w/ single bite missing (floor): 5% chance per level.
// Twin bimodal variant of the donut — partial ring w/ bite gap +
// crumbs. Uses the torus arc for the gap.
pub(crate) fn place_bitten_donuts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let napkin_colors: [u32; 5] = [0xf8f8f0, 0xf0e8d8, 0xf8e0e0, 0xe0f0e8, 0xf0e0f0];
    let napkin_mat = lambert!({ color: pick(&napkin_colors) });
    let dough_mat = lambert!({ color: 0xc89050 });
    let glaze_colors: [u32; 5] = [0xf0e8c0, 0x603020, 0xd0a0a8, 0xa0c090, 0xa07030];
    let glaze_mat = lambert!({ color: pick(&glaze_colors) });
    let bitten_donut = group();
    // Napkin square (like the donut's, more crumpled).
    let napkin = mesh(pg("BittenDonuts__napkin", box_geo!(0.20, 0.003, 0.20)), &napkin_mat);
    napkin.position().y = 0.0015;
    napkin.rotation().y = (rnd() - 0.5) * 0.4;
    bitten_donut.add(&napkin);
    // Multiple folded corners — 2 wrinkles for "used" look.
    for _i in 0..2 {
        let fold = mesh(pg("BittenDonuts__wrinkle", box_geo!(0.05, 0.005, 0.05)), &napkin_mat);
        fold.position().set((rnd() - 0.5) * 0.15, 0.005, (rnd() - 0.5) * 0.15);
        // Lies flat on the napkin; the modelled ridge is the wrinkle, and the
        // old 0.3-0.6 rad tilt draw now sets its height (the tilt sank one side
        // up to 1 cm into the floor).
        let pinch = 0.3 + rnd() * 0.3;
        fold.scale().y = 0.55 + pinch;
        let sy = fold.scl().y;
        fold.position().y = 0.0028 + 0.0025 * sy;
        let nry = napkin.rot().y;
        fold.rotation().y = nry + (rnd() - 0.5) * 0.4;
        bitten_donut.add(&fold);
    }
    // Donut dough — partial torus (bite gap = ~PI/3.5 = ~50°).
    let bite_arc = PI * 2.0 - PI / 3.5;
    let dough = mesh(pg("BittenDonuts__dough", torus_geo!(0.045, 0.020, 6, 14, bite_arc)), &dough_mat);
    dough.rotation().x = PI / 2.0;
    dough.rotation().z = rnd() * PI * 2.0;
    dough.position().y = 0.023;
    bitten_donut.add(&dough);
    // Glaze — matching partial torus above.
    let glaze = mesh(pg("BittenDonuts__glaze", torus_geo!(0.045, 0.018, 6, 14, bite_arc)), &glaze_mat);
    glaze.rotation().x = PI / 2.0;
    glaze.rotation().z = dough.rot().z;
    glaze.position().y = 0.033;
    bitten_donut.add(&glaze);
    // Bite-mark inner surface — small darker semi-sphere at torn end.
    let bite_inner_mat = lambert!({ color: 0xa07040 });
    // Compute bite gap position.
    let bite_ang = dough.rot().z + bite_arc + PI / 7.0;
    let bite_inner = mesh(pg("BittenDonuts__bite", sphere_geo!(0.014, 6, 4)), &bite_inner_mat);
    bite_inner.position().set(bite_ang.dcos() * 0.045, 0.028, bite_ang.dsin() * 0.045);
    bitten_donut.add(&bite_inner);
    // Scattered crumbs — 5-8 tiny boxes on napkin.
    let crumb_count = 5 + (rnd() * 4.0).floor() as i32;
    for _i in 0..crumb_count {
        let crumb_size = 0.004 + rnd() * 0.005;
        let crumb = mesh(pg("BittenDonuts__crumb", box_geo!(crumb_size, crumb_size, crumb_size)), &dough_mat);
        crumb.position().set((rnd() - 0.5) * 0.15, 0.004 + rnd() * 0.002, (rnd() - 0.5) * 0.15);
        crumb.rotation().y = rnd() * PI * 2.0;
        bitten_donut.add(&crumb);
    }
    // Maybe 1-2 sprinkle crumbs too.
    if rnd() < 0.5 {
        let sprinkle_colors: [u32; 6] = [0xff4040, 0x40a0ff, 0xffe040, 0x40e080, 0xff80c0, 0xffffff];
        for _i in 0..3 {
            let s_mat = lambert!({ color: pick(&sprinkle_colors) });
            let s = mesh(pg("Donuts__sprinkle", box_geo!(0.008, 0.002, 0.002)), &s_mat);
            s.position().set((rnd() - 0.5) * 0.15, 0.004, (rnd() - 0.5) * 0.15);
            s.rotation().y = rnd() * PI * 2.0;
            bitten_donut.add(&s);
        }
    }
    bitten_donut.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    bitten_donut.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bitten_donut);
}
