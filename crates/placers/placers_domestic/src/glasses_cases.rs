//! `place_glasses_cases`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Eyeglasses case opened w/ cleaning cloth (floor): 6% chance per level.
pub(crate) fn place_glasses_cases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let case_colors = [0x101010, 0x603020, 0x2060a0, 0x804a20, 0xa02040];
    let case_mat = lambert!({ color: choice(&case_colors) });
    let lining_mat = lambert!({ color: 0xd0c0a0 });
    let cloth_colors = [0xa0d0e0, 0xf0f0f0, 0xe0c0d0, 0xd0f0a0, 0xf0e0a0];
    let cloth_mat = lambert!({ color: choice(&cloth_colors) });
    let gc = group();
    // Bottom half of case — flat with velvet lining inside.
    let (case_w, case_d, case_h) = (0.15, 0.07, 0.020);
    let bottom_half = mesh(pg("GlassesCases__bottom", box_geo!(case_w, case_h, case_d)), &case_mat);
    bottom_half.position().y = case_h / 2.0 + 0.002;
    gc.add(&bottom_half);
    // Lining inside bottom half — soft cream disc.
    let lining = mesh(pg("GlassesCases__lining", box_geo!(case_w - 0.010, 0.003, case_d - 0.010)), &lining_mat);
    lining.position().y = case_h + 0.003;
    gc.add(&lining);
    // Top half tilted open — hinged upward at back, swinging about the
    // hinge line on the base's back top edge.
    let lid_pivot = group();
    lid_pivot.position().set(0, case_h + 0.002, -case_d * 0.5);
    lid_pivot.rotation().x = -PI / 3.0;
    gc.add(&lid_pivot);
    let top_half = mesh(pg("GlassesCases__lid", box_geo!(case_w, case_h * 0.7, case_d)), &case_mat);
    top_half.position().set(0, case_h * 0.35, case_d * 0.5);
    lid_pivot.add(&top_half);
    // Interior of top half (lining visible): in the lid's hollow.
    let top_lining = mesh(pg("GlassesCases__lining", box_geo!(case_w - 0.010, 0.003, case_d - 0.010)), &lining_mat);
    top_lining.position().set(0, case_h * 0.35 + 0.0033, case_d * 0.5); // hollow ceiling at lid-local 0.0048
    lid_pivot.add(&top_lining);
    // Hinge line (small dark cylinder), on the hinge axis.
    let hinge = mesh(pg("GlassesCases__hinge", cyl_geo!(0.003, 0.003, case_w * 0.85, 5)), lambert!({ color: 0x606060 }));
    hinge.rotation().z = PI / 2.0;
    hinge.position().set(0, case_h + 0.002, -case_d * 0.5);
    gc.add(&hinge);
    // Front snap clasp (tiny dark box).
    let clasp = mesh(pg("GlassesCases__clasp", box_geo!(0.006, 0.004, 0.002)), lambert!({ color: 0xc0a060 }));
    clasp.position().set(0, case_h * 0.6, case_d * 0.5 + 0.002);
    gc.add(&clasp);
    // Cleaning cloth — small square folded lying beside/inside.
    let cloth = mesh(pg("GlassesCases__cloth", box_geo!(0.05, 0.002, 0.05)), &cloth_mat);
    cloth.position().set(case_w * 0.5 + 0.04, 0.005, case_d * 0.1);
    cloth.rotation().y = 0.3;
    cloth.rotation().z = 0.15;
    gc.add(&cloth);
    // Second folded cloth corner for wrinkled effect.
    let cloth2 = mesh(pg("GlassesCases__clothFold", box_geo!(0.03, 0.003, 0.03)), &cloth_mat);
    cloth2.position().set(case_w * 0.5 + 0.03, 0.008, case_d * 0.14);
    cloth2.rotation().y = 0.4;
    cloth2.rotation().z = -0.2;
    gc.add(&cloth2);
    // 50% chance: folded eyeglasses inside case lining.
    if rnd() < 0.5 {
        let frame_colors = [0x101010, 0x604020, 0xa02040, 0xc0a040];
        let glass_frame_mat = lambert!({ color: choice(&frame_colors) });
        let lens_mat = basic_mat!({ color: 0xf0f0f8, transparent: true, opacity: 0.4 });
        // 2 lens rings side-by-side inside case.
        for side in [-1.0f64, 1.0] {
            let ring = mesh(pg("GlassesCases__rim", torus_geo!(0.012, 0.0015, 3, 10)), &glass_frame_mat);
            ring.rotation().x = PI / 2.0;
            ring.position().set(side * 0.015, case_h + 0.007, 0);
            gc.add(&ring);
            // Lens disc.
            let lens = mesh(circle_geo!(0.011, 10), &lens_mat);
            lens.rotation().x = -PI / 2.0;
            lens.position().set(side * 0.015, case_h + 0.008, 0);
            gc.add(&lens);
        }
        // Bridge.
        let bridge = mesh(pg("GlassesCases__bridge", box_geo!(0.008, 0.002, 0.002)), &glass_frame_mat);
        bridge.position().set(0, case_h + 0.008, -0.004);
        gc.add(&bridge);
        // Folded temples: each starts at its rim's outer top corner and folds
        // inward across the rims, lying on them.
        for side in [-1.0f64, 1.0] {
            let temple = mesh(pg("GlassesCases__temple", cyl_geo!(0.001, 0.001, 0.028, 3)), &glass_frame_mat);
            temple.rotation().z = side * PI / 2.0;
            temple.rotation().y = side * 0.2;
            temple.position().set(side * (0.026 - 0.014 * 0.2f64.dcos()), case_h + 0.0095, -0.004 + 0.014 * 0.2f64.dsin());
            gc.add(&temple);
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    gc.position().set(px, 0, pz);
    gc.rotation().y = rnd() * PI * 2.0;
    g.world.add(&gc);
}
