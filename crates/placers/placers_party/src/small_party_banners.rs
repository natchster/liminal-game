//! `place_small_party_banners`: Small party banner: 6% chance per level. Small horizontal paper banner strung between two ceiling anchor points with a gentle catenary sag.
use game::prelude::*;
use placers_kit::geometry::buf_geo;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small party banner: 6% chance per level. Small horizontal
/// paper banner strung between two ceiling anchor points with a
/// gentle catenary sag.
pub(crate) fn place_small_party_banners(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let ban_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0];
    let _ban_mat = lambert!({ color: pick(&ban_colors), side: DoubleSide });
    let string_mat = lambert!({ color: 0xf0e8d8 });
    let banner = group();
    let span_len = 1.6 + rnd() * 0.8;
    let sag = 0.18;
    let anchor_y = WALL_H - 0.05;
    // 5 triangular pennants along the string.
    let n_flags = 5;
    for i in 0..n_flags {
        let t_frac = (i as f64 + 0.5) / n_flags as f64;
        let x_pos = (t_frac - 0.5) * span_len;
        // Catenary sag approximation.
        let sag_y = anchor_y - sag * (1.0 - ((t_frac - 0.5) * 2.0).powi(2));
        let fs = 0.055;
        let flag_geo = buf_geo(&[-fs / 2.0, 0.0, 0.0, fs / 2.0, 0.0, 0.0, 0.0, -fs * 1.4, 0.0], &[0, 1, 2]);
        let flag_mat = lambert!({ color: pick(&ban_colors), side: DoubleSide });
        let flag = mesh(flag_geo, &flag_mat);
        flag.position().set(x_pos, sag_y, 0);
        banner.add(&flag);
    }
    // String segments between flag tops.
    for i in 0..n_flags {
        let t0 = (i as f64 + 0.5) / n_flags as f64;
        let t1 = (i as f64 + 1.5) / n_flags as f64;
        if i == n_flags - 1 {
            break;
        }
        let x0 = (t0 - 0.5) * span_len;
        let x1 = (t1 - 0.5) * span_len;
        let y0 = anchor_y - sag * (1.0 - ((t0 - 0.5) * 2.0).powi(2));
        let y1 = anchor_y - sag * (1.0 - ((t1 - 0.5) * 2.0).powi(2));
        let seg_len = hypot2(x1 - x0, y1 - y0);
        let seg = mesh(pg("SmallPartyBanners__string", cyl_geo!(0.002, 0.002, seg_len, 3)), &string_mat);
        seg.position().set((x0 + x1) / 2.0, (y0 + y1) / 2.0, 0);
        seg.rotation().z = -(x1 - x0).atan2(y1 - y0);
        banner.add(&seg);
    }
    // End string segments to anchor points: tied into the ceiling, each runs
    // to the outer pennant's top.
    let end_anchors = [(-span_len / 2.0, WALL_H), (span_len / 2.0, WALL_H)];
    for &(ex, ey) in end_anchors.iter() {
        let nearest =
            if ex < 0.0 { -span_len / 2.0 + span_len / n_flags as f64 / 2.0 } else { span_len / 2.0 - span_len / n_flags as f64 / 2.0 };
        let nearest_y = anchor_y - sag * (1.0 - ((0.5 / n_flags as f64 - 0.5) * 2.0).powi(2));
        let dx = nearest - ex;
        let dy = nearest_y - ey;
        let seg_len = hypot2(dx, dy);
        if seg_len < 0.01 {
            continue;
        }
        let seg = mesh(pg("SmallPartyBanners__string", cyl_geo!(0.002, 0.002, seg_len, 3)), &string_mat);
        seg.position().set((nearest + ex) / 2.0, (nearest_y + ey) / 2.0, 0);
        seg.rotation().z = -dx.atan2(dy);
        banner.add(&seg);
    }
    banner.position().set(g.wx(x) + (rnd() - 0.5) * 1.5, 0, g.wz(z) + (rnd() - 0.5) * 1.5);
    banner.rotation().y = rnd() * PI * 2.0;
    g.world.add(&banner);
}
