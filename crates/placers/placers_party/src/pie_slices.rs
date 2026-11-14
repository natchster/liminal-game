//! `place_pie_slices`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Pie slice on plate: 7% chance per level. Triangular
// wedge of pie (crust + filling) resting on ceramic plate.
pub(crate) fn place_pie_slices(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Filling color pool = variety of pie types.
    let filling_colors: [u32; 6] = [
        0xa02020, // cherry
        0x6a3a10, // pecan
        0xd0a040, // lemon meringue
        0x502010, // chocolate
        0x8040a0, // blueberry
        0xf0a060, // pumpkin
    ];
    let plate_colors: [u32; 3] = [0xffffff, 0xf0e8d8, 0xe8e0c8];
    let plate_mat = lambert!({ color: pick(&plate_colors) });
    let crust_mat = lambert!({ color: 0xc89060 });
    let filling_mat = lambert!({ color: pick(&filling_colors) });
    let pie = group();
    // Plate (shallow disc w/ raised rim).
    let plate = mesh(pg("PieSlices__plate", cyl_geo!(0.090, 0.082, 0.006, 16)), &plate_mat);
    plate.position().y = 0.003;
    pie.add(&plate);
    let plate_rim = mesh(pg("PieSlices__rim", torus_geo!(0.086, 0.004, 4, 16)), &plate_mat);
    plate_rim.rotation().x = PI / 2.0;
    plate_rim.position().y = 0.008;
    pie.add(&plate_rim);
    // Pie slice = partial cylinder with a 60° theta length.
    let slice_h = 0.028;
    let slice_r = 0.075;
    let slice_geom = pg("PieSlices__filling", cyl_geo!(slice_r, slice_r, slice_h, 10, 1, false, 0, PI / 3.0));
    let filling = mesh(slice_geom, &filling_mat);
    filling.position().y = 0.006 + slice_h / 2.0;
    filling.rotation().y = rnd() * PI * 2.0;
    pie.add(&filling);
    // Top crust cap (thin) on the filling — slightly lighter crust color.
    let top_crust = mesh(pg("PieSlices__crust", cyl_geo!(slice_r, slice_r, 0.004, 10, 1, false, 0, PI / 3.0)), &crust_mat);
    top_crust.position().y = 0.006 + slice_h - 0.002;
    top_crust.rotation().y = filling.rot().y;
    pie.add(&top_crust);
    // Optional dollop of cream: 40% chance.
    if rnd() < 0.4 {
        let cream = mesh(pg("PieSlices__cream", sphere_geo!(0.012, 6, 5)), &plate_mat);
        // Position roughly near the pointed apex of the slice.
        // The wedge's centre line is at (sin, cos) of its angle (the cylinder's theta
        // convention), 3 cm out where the slice is wide enough for the dollop.
        let ang = filling.rot().y + PI / 6.0;
        cream.position().set(ang.dsin() * 0.030, 0.006 + slice_h + 0.004, ang.dcos() * 0.030);
        cream.scale().y = 0.7;
        pie.add(&cream);
    }
    pie.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    pie.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pie);
}
