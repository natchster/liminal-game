//! `place_small_tombstones`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::pick_tile;

// Small tombstone: 6% chance per level. Gray stone slab
// with rounded top + engraved cross. Halloween family.
pub(crate) fn place_small_tombstones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let stone_colors: [u32; 4] = [0x808080, 0x707070, 0x606060, 0x585858];
    let stone_mat = lambert!({ color: pick_hex(&stone_colors) });
    let engrave_mat = lambert!({ color: 0x2a2418 });
    let stone = group();
    // Slab body (rectangular).
    let body = mesh(pg("SmallTombstones__body", box_geo!(0.12, 0.16, 0.024)), &stone_mat);
    body.position().y = 0.084;
    stone.add(&body);
    // Rounded top (half cylinder).
    let top = mesh(pg("SmallTombstones__top", cyl_geo!(0.060, 0.060, 0.024, 12, 1, false, 0, PI)), &stone_mat);
    // Turn the half-disc (local x >= 0) about its own axis first, so the
    // arch spans the slab's full width across the top instead of a D on its
    // right half.
    top.rotation().y = PI / 2.0;
    top.rotation().x = PI / 2.0;
    top.position().set(0, 0.164, 0);
    stone.add(&top);
    // Base slab.
    let base = mesh(pg("SmallTombstones__base", box_geo!(0.14, 0.014, 0.040)), &stone_mat);
    base.position().y = 0.007;
    stone.add(&base);
    // Engraved cross on front (2 bars).
    let v_bar = mesh(pg("SmallTombstones__vBar", box_geo!(0.010, 0.070, 0.003)), &engrave_mat);
    v_bar.position().set(0, 0.120, 0.013);
    stone.add(&v_bar);
    let h_bar = mesh(pg("SmallTombstones__hBar", box_geo!(0.035, 0.010, 0.003)), &engrave_mat);
    h_bar.position().set(0, 0.130, 0.013);
    stone.add(&h_bar);
    // Slight tilt for aged look.
    stone.rotation().z = (rnd() - 0.5) * 0.15;
    stone.rotation().x = (rnd() - 0.5) * 0.10;
    stone.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    stone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&stone);
}
