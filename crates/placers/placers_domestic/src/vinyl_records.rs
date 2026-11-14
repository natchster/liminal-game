//! `place_vinyl_records`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Vinyl record: 12% chance per level. Round black disc with
// color label at center.
pub(crate) fn place_vinyl_records(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let disc_mat = lambert!({ color: 0x0a0806 });
    let label_colors = [0xa02020, 0x2040a0, 0xa08020, 0x208020, 0x8020a0];
    let label_mat = lambert!({ color: label_colors[ri(label_colors.len())] });
    let record = group();
    // Main disc: 12-inch LP ≈ 0.15 m radius.
    let disc = mesh(pg("VinylRecords__disc", cyl_geo!(0.15, 0.15, 0.003, 20)), &disc_mat);
    disc.position().y = 0.0015;
    record.add(&disc);
    // Center label: small colored disc.
    let label = mesh(pg("VinylRecords__label", cyl_geo!(0.04, 0.04, 0.0032, 12)), &label_mat);
    label.position().y = 0.003;
    record.add(&label);
    // Tiny spindle hole (near-black disc in label center).
    let hole_mat = basic_mat!({ color: 0x040404 });
    let hole = mesh(cyl_geo!(0.005, 0.005, 0.004, 8), &hole_mat);
    hole.position().y = 0.0034;
    record.add(&hole);
    // 20% chance: dark crack line across disc surface.
    if rnd() < 0.20 {
        let crack_mat = basic_mat!({ color: 0x1a1410, transparent: true, opacity: 0.9, depthWrite: false });
        let crack = mesh(plane_geo!(0.003, 0.24), &crack_mat);
        crack.rotation().x = -PI / 2.0;
        crack.rotation().z = rnd() * PI;
        crack.position().y = 0.0033;
        record.add(&crack);
    }
    record.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    record.rotation().y = rnd() * PI * 2.0;
    g.world.add(&record);
    // 25% chance: album sleeve (square card) 0.15-0.30 m from record.
    if rnd() < 0.25 {
        let sleeve_colors = [0x2a2020, 0x1a2028, 0x282018, 0x1a1a2a];
        let sleeve_mat = lambert!({ color: sleeve_colors[ri(sleeve_colors.len())] });
        let sleeve = mesh(pg("VinylRecords__sleeve", box_geo!(0.32, 0.005, 0.32)), &sleeve_mat);
        let s_ang = rnd() * PI * 2.0;
        let s_rad = 0.15 + rnd() * 0.15;
        let rp = record.pos();
        sleeve.position().set(rp.x + s_ang.dcos() * s_rad, 0.0025, rp.z + s_ang.dsin() * s_rad);
        sleeve.rotation().y = rnd() * PI * 2.0;
        sleeve.rotation().x = (rnd() - 0.5) * 0.08;
        g.world.add(&sleeve);
    }
}
