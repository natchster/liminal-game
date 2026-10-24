//! Draws the app icon (a fluorescent-lit yellow corridor) into PNGs for iconutil.
//! `cargo run -p engine --example make_icon -- out_dir`
use engine::canvas::Canvas;

fn draw(size: u32) -> Vec<u8> {
    let s = size as f64;
    let mut c = Canvas::new(size, size);
    // rounded tile
    let r = s * 0.2237;
    c.begin_path();
    c.move_to(r, 0);
    c.line_to(s - r, 0);
    c.quadratic_curve_to(s, 0, s, r);
    c.line_to(s, s - r);
    c.quadratic_curve_to(s, s, s - r, s);
    c.line_to(r, s);
    c.quadratic_curve_to(0, s, 0, s - r);
    c.line_to(0, r);
    c.quadratic_curve_to(0, 0, r, 0);
    c.close_path();
    c.save();
    c.clip();
    let g = c.create_linear_gradient(0, 0, 0, s);
    g.add_color_stop(0.0, "#d8cf98");
    g.add_color_stop(0.45, "#c9b45a");
    g.add_color_stop(1.0, "#6e5a22");
    c.set_fill_style(&g);
    c.fill_rect(0, 0, s, s);
    // wallpaper stripes
    c.set_fill_style("rgba(120,96,30,0.18)");
    let mut x = 0.0;
    while x < s {
        c.fill_rect(x, s * 0.22, s * 0.018, s * 0.56);
        x += s * 0.06;
    }
    // corridor: a dark doorway in perspective
    c.set_fill_style("rgba(30,24,10,0.9)");
    c.begin_path();
    c.move_to(s * 0.41, s * 0.36);
    c.line_to(s * 0.59, s * 0.36);
    c.line_to(s * 0.59, s * 0.74);
    c.line_to(s * 0.41, s * 0.74);
    c.close_path();
    c.fill();
    // ceiling panels
    for (i, w) in [(0, 0.46), (1, 0.30), (2, 0.18)] {
        let y = s * (0.08 + 0.07 * i as f64);
        c.set_fill_style("#fffbe2");
        c.set_shadow_color("rgba(255,250,200,0.9)");
        c.set_shadow_blur(s * 0.03);
        c.fill_rect(s * (0.5 - w / 2.0), y, s * w, s * 0.035);
    }
    c.set_shadow_blur(0);
    // carpet
    let fg = c.create_linear_gradient(0, s * 0.74, 0, s);
    fg.add_color_stop(0.0, "#8a7a3e");
    fg.add_color_stop(1.0, "#4a3f18");
    c.set_fill_style(&fg);
    c.fill_rect(0, s * 0.74, s, s * 0.26);
    // vignette
    let v = c.create_radial_gradient(s / 2.0, s / 2.0, s * 0.25, s / 2.0, s / 2.0, s * 0.75);
    v.add_color_stop(0.0, "rgba(0,0,0,0)");
    v.add_color_stop(1.0, "rgba(0,0,0,0.55)");
    c.set_fill_style(&v);
    c.fill_rect(0, 0, s, s);
    c.restore();
    c.to_image().data
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or("AppIcon.iconset".into());
    std::fs::create_dir_all(&out).unwrap();
    for (name, px) in [
        ("icon_16x16", 16),
        ("icon_16x16@2x", 32),
        ("icon_32x32", 32),
        ("icon_32x32@2x", 64),
        ("icon_128x128", 128),
        ("icon_128x128@2x", 256),
        ("icon_256x256", 256),
        ("icon_256x256@2x", 512),
        ("icon_512x512", 512),
        ("icon_512x512@2x", 1024),
    ] {
        engine::png::write_rgba(&format!("{out}/{name}.png"), px, px, &draw(px)).unwrap();
    }
    println!("wrote {out}");
}
