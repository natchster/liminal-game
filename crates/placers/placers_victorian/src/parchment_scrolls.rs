//! `place_parchment_scrolls`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small parchment scroll: 8% chance per level.
pub(crate) fn place_parchment_scrolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let parchment_mat = lambert!({ color: 0xe8dcc0 });
    let shadow_mat = lambert!({ color: 0xc0b090 });
    let ribbon_colors = [0xa02040, 0x4030c0, 0x2a2418, 0x8a2040];
    let ribbon_mat = lambert!({ color: pick(&ribbon_colors), side: DoubleSide });
    let scroll = group();
    let scroll_len = 0.22;
    let scroll_r = 0.020;
    // Main roll.
    let roll = mesh(pg("ParchmentScrolls__roll", cyl_geo!(scroll_r, scroll_r, scroll_len, 12)), &parchment_mat);
    roll.rotation().z = PI / 2.0;
    roll.position().set(scroll_len / 2.0, scroll_r, 0);
    scroll.add(&roll);
    // Inner shadow discs at ends (roll depth).
    for s in [-1.0f64, 1.0] {
        let shadow_disc = mesh(circle_geo!(scroll_r * 0.75, 8), &shadow_mat);
        shadow_disc.rotation().y = PI / 2.0;
        shadow_disc.position().set(if s > 0.0 { scroll_len - 0.001 } else { 0.001 }, scroll_r, 0);
        if s < 0.0 {
            shadow_disc.rotation().y = -PI / 2.0;
        }
        scroll.add(&shadow_disc);
    }
    // Ribbon wrapped around middle, snug on the roll (radius 0.021).
    let ribbon = mesh(cyl_geo!(scroll_r + 0.003, scroll_r + 0.003, 0.012, 12, 1, true), &ribbon_mat);
    ribbon.scale().set((scroll_r + 0.001) / (scroll_r + 0.003), 1, (scroll_r + 0.001) / (scroll_r + 0.003));
    ribbon.rotation().z = PI / 2.0;
    ribbon.position().set(scroll_len / 2.0, scroll_r, 0);
    scroll.add(&ribbon);
    // 2 ribbon tail planes hanging.
    for s in [-1.0f64, 1.0] {
        let tail = mesh(plane_geo!(0.008, 0.030), &ribbon_mat);
        tail.rotation().y = PI / 2.0;
        tail.position().set(scroll_len / 2.0, scroll_r + 0.008, s * 0.024);
        tail.rotation().z = s * 0.2;
        scroll.add(&tail);
    }
    scroll.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    scroll.rotation().y = rnd() * PI * 2.0;
    g.world.add(&scroll);
}
