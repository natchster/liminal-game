//! `place_mission_patches`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::wall_tiles;

pub(crate) fn place_mission_patches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x8b5a2b, 0x604020, 0xc0a060, 0x2a2a2a];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let patch_mat = lambert!({ map: mission_patch_texture() });
    let mp = group();
    let mount_y = 1.60;
    let frame_size = 0.22;
    let frame = mesh(pg("MissionPatches__frame", box_geo!(frame_size, frame_size, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    mp.add(&frame);
    let patch = mesh(plane_geo!(frame_size - 0.018, frame_size - 0.018), &patch_mat);
    patch.position().set(0, mount_y, 0.019);
    mp.add(&patch);
    mp.rotation().y = face_wall(dx, dz);
    mp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    mp.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&mp);
}

game::lazy_static_tex!(
    mission_patch_texture,
    canvas_tex(220, 220, |ctx, w, h| {
        // Background — dark navy square.
        ctx.set_fill_style("#101830");
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        let patch_r = 85.0;
        // (name, year, bg, accent, craft)
        let missions: [(&str, &str, &str, &str, &str); 4] = [
            ("APOLLO 11", "1969", "#a02020", "#f0e040", "lunar"),
            ("GEMINI IV", "1965", "#404080", "#e0d090", "capsule"),
            ("MERCURY 7", "1962", "#603020", "#f0c040", "capsule"),
            ("SKYLAB", "1973", "#202070", "#e0e0e0", "station"),
        ];
        let m = missions[(rnd() * missions.len() as f64).floor() as usize];
        // Patch body.
        ctx.set_fill_style(m.2);
        ctx.begin_path();
        ctx.arc(cx, cy, patch_r, 0, PI * 2.0);
        ctx.fill();
        // Outer rim.
        ctx.set_stroke_style(m.3);
        ctx.set_line_width(4);
        ctx.begin_path();
        ctx.arc(cx, cy, patch_r, 0, PI * 2.0);
        ctx.stroke();
        // Central spacecraft icon.
        ctx.set_fill_style(m.3);
        if m.4 == "lunar" {
            ctx.fill_rect(cx - 10.0, cy - 10.0, 20, 20);
            ctx.set_stroke_style(m.3);
            ctx.set_line_width(2);
            for i in 0..4 {
                let ang = (i as f64 / 4.0) * PI * 2.0 + PI / 4.0;
                ctx.begin_path();
                ctx.move_to(cx + ang.dcos() * 8.0, cy + ang.dsin() * 8.0);
                ctx.line_to(cx + ang.dcos() * 25.0, cy + ang.dsin() * 25.0);
                ctx.stroke();
            }
        } else if m.4 == "capsule" {
            ctx.begin_path();
            ctx.move_to(cx - 15.0, cy + 15.0);
            ctx.line_to(cx - 20.0, cy + 5.0);
            ctx.line_to(cx - 8.0, cy - 20.0);
            ctx.line_to(cx + 8.0, cy - 20.0);
            ctx.line_to(cx + 20.0, cy + 5.0);
            ctx.line_to(cx + 15.0, cy + 15.0);
            ctx.close_path();
            ctx.fill();
        } else {
            // Space station — long cylinder w/ solar panels.
            ctx.fill_rect(cx - 25.0, cy - 4.0, 50, 8);
            ctx.fill_rect(cx - 45.0, cy - 2.0, 15, 4);
            ctx.fill_rect(cx + 30.0, cy - 2.0, 15, 4);
        }
        // Star field in patch background (small white dots).
        ctx.set_fill_style("#f0f0f0");
        for _ in 0..15 {
            let sang = rnd() * PI * 2.0;
            let sr = 30.0 + rnd() * 40.0;
            let sx = cx + sang.dcos() * sr;
            let sy = cy + sang.dsin() * sr;
            ctx.begin_path();
            ctx.arc(sx, sy, 0.8 + rnd(), 0, PI * 2.0);
            ctx.fill();
        }
        // Mission name.
        ctx.set_fill_style(m.3);
        ctx.set_font("bold 12px serif");
        ctx.set_text_align("center");
        ctx.fill_text(m.0, cx, cy - patch_r + 20.0);
        // Year at bottom.
        ctx.set_font("bold 10px serif");
        ctx.fill_text(m.1, cx, cy + patch_r - 12.0);
        // 3 astronaut names.
        ctx.set_font("bold 7px sans-serif");
        ctx.set_text_align("left");
        let initials: [[&str; 3]; 4] = [
            ["A. SHEPARD", "V. GRISSOM", "J. GLENN"],
            ["N. ARMSTRONG", "B. ALDRIN", "M. COLLINS"],
            ["J. YOUNG", "M. GRISSOM", "E. WHITE"],
            ["C. CONRAD", "J. LOUSMA", "O. GARRIOTT"],
        ];
        let crew = initials[(rnd() * initials.len() as f64).floor() as usize];
        for i in 0..3 {
            ctx.fill_text(crew[i], cx - patch_r + 10.0, cy - 20.0 + i as f64 * 14.0);
        }
        // "NASA" text below patch.
        ctx.set_fill_style("#f0f0f0");
        ctx.set_font("italic bold 10px serif");
        ctx.set_text_align("center");
        ctx.fill_text("NASA", cx, h - 20.0);
    })
);
