//! `place_silver_trays`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Silver serving tray with optional teacups + biscuits + sugar bowl (floor): 4%.
pub(crate) fn place_silver_trays(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // Tray metal 3-color pool: silver / pewter / brass.
    let tray_mats = [
        lambert!({ color: 0xd8d8e0, emissive: 0x161818 }), // polished silver
        lambert!({ color: 0x909098 }),                     // pewter
        lambert!({ color: 0xc0a060, emissive: 0x201810 }), // brass
    ];
    let tray_mat = tray_mats[ri(tray_mats.len())].clone();
    let st = group();
    // Main tray body.
    let tray_r = 0.14;
    let tray = mesh(pg("SilverTrays__tray", cyl_geo!(tray_r, tray_r, 0.008, 20)), &tray_mat);
    tray.position().y = 0.008;
    st.add(&tray);
    // Raised rim.
    let rim = mesh(pg("SilverTrays__rim", torus_geo!(tray_r, 0.006, 5, 20)), &tray_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.014;
    st.add(&rim);
    // 2 side handles.
    for side in [-1.0f64, 1.0] {
        let handle = mesh(pg("SilverTrays__handle", torus_geo!(0.020, 0.004, 4, 8, PI)), &tray_mat);
        handle.rotation().x = PI / 2.0;
        handle.rotation().z = -PI / 2.0;
        handle.rotation().y = if side > 0.0 { 0.0 } else { PI };
        handle.position().set(side * (tray_r + 0.002), 0.015, 0);
        st.add(&handle);
    }
    // Engraved center ring (decorative).
    let center_ring = mesh(pg("SilverTrays__engraving", torus_geo!(tray_r * 0.45, 0.001, 4, 20)), lambert!({ color: 0xa0a0a8 }));
    center_ring.rotation().x = PI / 2.0;
    center_ring.position().y = 0.0114;
    st.add(&center_ring);
    // Item pool.
    let has_tea = rnd() < 0.60;
    let tea_colors: [u32; 4] = [0xf0e8d8, 0xe0d8c0, 0xf0d0d0, 0xd0e0f0];
    if has_tea {
        let teacup_count = 1 + (rnd() * 2.0).floor() as i32;
        for i in 0..teacup_count {
            let fi = i as f64;
            let cup_mat = lambert!({ color: tea_colors[ri(tea_colors.len())] });
            let cup = mesh(pg("SilverTrays__cup", cyl_geo!(0.024, 0.020, 0.032, 12)), &cup_mat);
            cup.position().set((fi - 0.5) * 0.08, 0.0292, -0.03);
            st.add(&cup);
            let saucer = mesh(pg("SilverTrays__saucer", cyl_geo!(0.038, 0.038, 0.003, 14)), &cup_mat);
            saucer.position().set((fi - 0.5) * 0.08, 0.0131, -0.03);
            st.add(&saucer);
            let cup_handle = mesh(pg("SilverTrays__cupHandle", torus_geo!(0.008, 0.002, 4, 8)), &cup_mat);
            cup_handle.position().set((fi - 0.5) * 0.08 + 0.0295, 0.0282, -0.03);
            st.add(&cup_handle);
            // Tea inside — dark liquid disc.
            let tea_mat = lambert!({ color: 0x604020, transparent: true, opacity: 0.9 });
            let tea_surf = mesh(circle_geo!(0.020, 12), &tea_mat);
            tea_surf.rotation().x = -PI / 2.0;
            tea_surf.position().set((fi - 0.5) * 0.08, 0.0422, -0.03);
            st.add(&tea_surf);
            // Steam wisp animator (60% per cup).
            if rnd() < 0.6 {
                let tea_steam_mat = basic_mat!({ color: 0xf0f0f0, transparent: true, opacity: 0.16 });
                struct Wisp {
                    mesh: Obj,
                    phase: f64,
                    base_y: f64,
                    cup_cx: f64,
                }
                let mut tea_wisps: Vec<Wisp> = vec![];
                let cup_cx = (fi - 0.5) * 0.08;
                for j in 0..2 {
                    let wisp = mesh(sphere_geo!(0.008, 6, 5), &tea_steam_mat);
                    wisp.scale().y = 2.0;
                    let px = cup_cx + (rnd() - 0.5) * 0.008;
                    let py = 0.055 + j as f64 * 0.020;
                    let pz = -0.03 + (rnd() - 0.5) * 0.008;
                    wisp.position().set(px, py, pz);
                    let phase = rnd() * PI * 2.0;
                    tea_wisps.push(Wisp { mesh: wisp.clone(), phase, base_y: wisp.pos().y, cup_cx });
                    st.add(&wisp);
                }
                g.animate(move |_g, _dt, t| {
                    for w in &tea_wisps {
                        {
                            let mut p = w.mesh.position();
                            p.y = w.base_y + (t * 0.7 + w.phase).dsin() * 0.010;
                            p.x = w.cup_cx + (t * 0.5 + w.phase).dsin() * 0.010;
                            p.z = -0.03 + (t * 0.6 + w.phase * 1.3).dcos() * 0.010;
                        }
                        let s = 0.85 + (t * 0.9 + w.phase).dsin() * 0.18;
                        w.mesh.scale().set(s, 2.0 * s, s);
                    }
                });
            }
            // 40% chance small spoon leaning against saucer.
            if rnd() < 0.4 {
                let spoon_mat = lambert!({ color: 0xd8d8e0 });
                let spoon = mesh(pg("SilverTrays__spoon", cyl_geo!(0.0015, 0.0015, 0.030, 4)), &spoon_mat);
                spoon.position().set((fi - 0.5) * 0.08 + 0.030, 0.027, -0.045);
                spoon.rotation().z = 0.6;
                st.add(&spoon);
                let spoon_bowl = mesh(pg("SilverTrays__spoonBowl", sphere_geo!(0.004, 6, 4)), &spoon_mat);
                spoon_bowl.scale().set(1.4, 0.4, 1.0);
                spoon_bowl.position().set((fi - 0.5) * 0.08 + 0.043, 0.0132, -0.045);
                st.add(&spoon_bowl);
            }
        }
    }
    // 50% chance biscuits on small plate.
    if rnd() < 0.5 {
        let plate_mat = lambert!({ color: 0xf0e8d8 });
        let plate = mesh(pg("SilverTrays__plate", cyl_geo!(0.040, 0.040, 0.003, 14)), &plate_mat);
        plate.position().set(0.04, 0.0131, 0.06);
        st.add(&plate);
        let biscuit_count = 3 + (rnd() * 2.0).floor() as i32;
        let biscuit_colors: [u32; 3] = [0xc09060, 0xa07040, 0xd0a080];
        for i in 0..biscuit_count {
            let biscuit_mat = lambert!({ color: biscuit_colors[ri(biscuit_colors.len())] });
            let biscuit = mesh(pg("SilverTrays__biscuit", cyl_geo!(0.011, 0.011, 0.004, 8)), &biscuit_mat);
            let stacked = i < 2;
            if stacked {
                biscuit.position().set(0.04, 0.0151 + i as f64 * 0.004, 0.06);
            } else {
                let px = 0.04 + (rnd() - 0.5) * 0.045;
                let pz = 0.06 + (rnd() - 0.5) * 0.045;
                biscuit.position().set(px, 0.0156, pz);
            }
            st.add(&biscuit);
        }
    }
    // 40% chance sugar bowl (small covered pot).
    if rnd() < 0.4 {
        let sugar_mat = lambert!({ color: tea_colors[ri(tea_colors.len())] });
        let sugar = mesh(pg("SilverTrays__sugar", cyl_geo!(0.018, 0.020, 0.024, 10)), &sugar_mat);
        sugar.position().set(-0.07, 0.0236, 0.03);
        st.add(&sugar);
        let lid = mesh(pg("SilverTrays__lid", sphere_geo!(0.020, 8, 5, 0, PI * 2.0, 0, PI * 0.4)), &sugar_mat);
        lid.position().set(-0.07, 0.0291, 0.03);
        st.add(&lid);
        let knob = mesh(pg("SilverTrays__knob", sphere_geo!(0.004, 5, 4)), &sugar_mat);
        knob.position().set(-0.07, 0.0527, 0.03);
        st.add(&knob);
    }
    // 25% chance small folded cloth napkin on tray.
    if rnd() < 0.25 {
        // 0xc0a08040 is a 32-bit value; Color::hex keeps only the low 24 bits (0xa08040), deliberately.
        let nap_colors: [u32; 3] = [0xf0f0e8, 0xe0e0d8, 0xc0a08040];
        let nap_mat = lambert!({ color: nap_colors[ri(nap_colors.len())] });
        let napkin = mesh(pg("SilverTrays__napkin", box_geo!(0.045, 0.005, 0.030)), &nap_mat);
        napkin.position().set(0.0, 0.0141, -0.095);
        napkin.rotation().y = (rnd() - 0.5) * 0.4;
        st.add(&napkin);
    }
    st.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    st.rotation().y = rnd() * PI * 2.0;
    g.world.add(&st);
}
