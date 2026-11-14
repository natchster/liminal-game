//! `place_sewing_baskets`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Sewing basket: 4% chance per level. Wicker basket
// w/ folded fabric + spool of thread + optional needles.
pub(crate) fn place_sewing_baskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Wicker color pool — earth tones.
    let wicker_colors: [u32; 4] = [0xa07040, 0x8b6030, 0xc09060, 0x704a20];
    let wicker_mat = lambert!({ color: pick(&wicker_colors) });
    let basket = group();
    // Basket body (open cylinder, tapered).
    let body = mesh(pg("SewingBaskets__body", cyl_geo!(0.10, 0.075, 0.070, 12, 1, true)), &wicker_mat);
    body.position().y = 0.035;
    basket.add(&body);
    // Basket bottom disc.
    let bottom = mesh(circle_geo!(0.075, 12), &wicker_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.001;
    basket.add(&bottom);
    // Rim (torus at top).
    let rim = mesh(pg("SewingBaskets__rim", torus_geo!(0.10, 0.006, 5, 14)), &wicker_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.070;
    basket.add(&rim);
    // Handle (partial torus arching over top).
    let handle = mesh(pg("SewingBaskets__handle", torus_geo!(0.095, 0.005, 4, 10, PI)), &wicker_mat);
    // Upright half hoop (XY plane) arching over the basket, ends on the rim
    // (the old x/z turns laid it flat inside the rim).
    handle.position().y = 0.070;
    basket.add(&handle);
    // Folded fabric inside (flat box — random pastel).
    let fabric_colors: [u32; 6] = [0xf0c0c0, 0xc0d0f0, 0xd0f0c0, 0xf0e8a0, 0xe0c0f0, 0xa0d0e0];
    let fabric_mat = lambert!({ color: pick(&fabric_colors) });
    let fabric = mesh(pg("SewingBaskets__fabric", box_geo!(0.12, 0.024, 0.09)), &fabric_mat);
    fabric.position().set(0, 0.055, 0);
    fabric.rotation().y = (rnd() - 0.5) * 0.4;
    basket.add(&fabric);
    // Spool of thread (small cylinder w/ different color).
    let thread_colors: [u32; 5] = [0x202020, 0xa02020, 0x2060a0, 0xf0f0f0, 0x408060];
    let spool_mat = lambert!({ color: 0xd8b878 });
    let thread_mat = lambert!({ color: pick(&thread_colors) });
    let spool_base = mesh(pg("SewingBaskets__spoolend", cyl_geo!(0.014, 0.014, 0.004, 8)), &spool_mat);
    spool_base.position().set(0.045, 0.070, 0.020);
    basket.add(&spool_base);
    let spool_thread = mesh(pg("SewingBaskets__thread", cyl_geo!(0.012, 0.012, 0.020, 8)), &thread_mat);
    spool_thread.position().set(0.045, 0.082, 0.020);
    basket.add(&spool_thread);
    let spool_top = mesh(pg("SewingBaskets__spoolend", cyl_geo!(0.014, 0.014, 0.004, 8)), &spool_mat);
    spool_top.position().set(0.045, 0.094, 0.020);
    basket.add(&spool_top);
    // Optional pincushion — 60% chance: tiny red hemisphere w/ pin marks.
    if rnd() < 0.6 {
        let cushion_mat = lambert!({ color: 0xa02020 });
        let cushion = mesh(pg("SewingBaskets__cushion", sphere_geo!(0.020, 6, 5, 0, PI * 2.0, 0, PI / 2.0)), &cushion_mat);
        cushion.position().set(-0.040, 0.067, 0.025); // on the fabric top (0.067)
        basket.add(&cushion);
        // 3 tiny pin heads (small spheres, white).
        let pin_head_mat = lambert!({ color: 0xf8f8f8 });
        let mut pins: Vec<(Obj, f64)> = vec![];
        for i in 0..3 {
            let ang = (i as f64 / 3.0) * PI * 2.0;
            let pin = mesh(sphere_geo!(0.003, 4, 3), &pin_head_mat);
            pin.position().set(-0.040 + ang.dcos() * 0.010, 0.087, 0.025 + ang.dsin() * 0.010);
            basket.add(&pin);
            let by = pin.pos().y;
            pins.push((pin, by));
        }
        // Subtle pincushion bob animator — 40% chance.
        // Tiny Y-bob on cushion + pins together (as if soft fabric
        // is settling from being pressed).
        if rnd() < 0.4 {
            let bob_phase = rnd() * PI * 2.0;
            let cushion_base_y = cushion.pos().y;
            g.animate(move |_g, _dt, t| {
                let bob = (t * 1.4 + bob_phase).dsin() * 0.0006;
                cushion.position().y = cushion_base_y + bob;
                for (m, base_y) in &pins {
                    m.position().y = *base_y + bob;
                }
            });
        }
    }
    basket.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    basket.rotation().y = rnd() * PI * 2.0;
    g.world.add(&basket);
}
