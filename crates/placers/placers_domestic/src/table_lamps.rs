//! `place_table_lamps`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri as pick;

// Table lamp: 15% chance per level. 3-piece Group: circular
// base + narrower vertical stem + tapered shade at top. Dark/off.
pub(crate) fn place_table_lamps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let base_colors = [0x2a2418, 0x1a1a1e, 0x3a2818];
    let base_color = base_colors[pick(base_colors.len())];
    let shade_colors = [0xa89880, 0x807060, 0x504840];
    let shade_color = shade_colors[pick(shade_colors.len())];
    let base_mat = lambert!({ color: base_color });
    let shade_mat = lambert!({ color: shade_color });
    let lamp = group();
    // Base: wide short cylinder.
    let base = mesh(pg("TableLamps__base", cyl_geo!(0.10, 0.10, 0.03, 12)), &base_mat);
    base.position().y = 0.015;
    lamp.add(&base);
    // Stem: narrow tall cylinder.
    let stem = mesh(pg("TableLamps__stem", cyl_geo!(0.010, 0.010, 0.32, 6)), &base_mat);
    stem.position().y = 0.19;
    lamp.add(&stem);
    // Shade: tapered cone (wide top → narrower bottom).
    let shade = mesh(pg("TableLamps__shade", cyl_geo!(0.11, 0.08, 0.14, 8)), &shade_mat);
    shade.position().y = 0.42;
    lamp.add(&shade);
    // Slight lean for character.
    lamp.rotation().z = (rnd() - 0.5) * 0.10;
    lamp.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    lamp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&lamp);
}
