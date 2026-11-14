//! `place_framed_newspapers`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::wall_tiles;

pub(crate) fn place_framed_newspapers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0x8b5a2b, 0x202020, 0xc0a060];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    // Reuse `newspaper_texture` for the front page.
    let news_mat = lambert!({ map: newspaper_texture() });
    let fnw = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.22, 0.28);
    let frame = mesh(pg("FramedNewspapers__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    fnw.add(&frame);
    // Gold-mat inner border (2-layer framing).
    let mat_border_mat = lambert!({ color: 0xd0c090 });
    let mat_border = mesh(pg("FramedNewspapers__mat", box_geo!(frame_w - 0.010, frame_h - 0.010, 0.002)), &mat_border_mat);
    mat_border.position().set(0, mount_y, 0.018);
    fnw.add(&mat_border);
    let news = mesh(plane_geo!(frame_w - 0.030, frame_h - 0.030), &news_mat);
    news.position().set(0, mount_y, 0.020);
    fnw.add(&news);
    // Small brass plaque below frame.
    let plaque_mat = lambert!({ color: 0xc0a060 });
    let plaque = mesh(pg("FramedNewspapers__plaque", box_geo!(frame_w * 0.6, 0.014, 0.004)), &plaque_mat);
    plaque.position().set(0, mount_y - frame_h * 0.5 - 0.015, 0.014);
    fnw.add(&plaque);
    // Engraved text line on plaque (thin dark rectangle).
    let text_line_mat = lambert!({ color: 0x2a1810 });
    let text_line = mesh(box_geo!(frame_w * 0.45, 0.001, 0.001), &text_line_mat);
    text_line.position().set(0, mount_y - frame_h * 0.5 - 0.015, 0.017);
    fnw.add(&text_line);
    fnw.rotation().y = face_wall(dx, dz);
    fnw.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    fnw.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&fnw);
}
