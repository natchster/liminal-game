//! Headless render of a small test scene to PNG (shader / pipeline check).
use engine::prelude::*;
use engine::render::*;
use objc2_metal::MTLCommandBuffer;

fn main() {
    let device = objc2_metal::MTLCreateSystemDefaultDevice().expect("metal");
    let mut r = Renderer::new(device);
    let scene = new_scene();
    let camera = perspective_camera(74.0, 16.0 / 9.0, 0.05, 90.0);
    camera.set_pos(0, 1.6, 5);
    camera.look_at(0, 1.0, 0);
    // a canvas-textured floor
    let mut c = Canvas::new(256, 256);
    c.set_fill_style("#c8b070"); c.fill_rect(0, 0, 256, 256);
    c.set_fill_style("rgba(80,60,20,0.5)");
    for i in 0..8 { c.begin_path(); c.arc(32.0 * i as f64 + 16.0, 128, 14, 0, 6.3); c.fill(); }
    c.set_fill_style("#201008"); c.set_font("bold 28px serif"); c.set_text_align("center"); c.fill_text("BACKROOMS", 128, 60);
    let tex = c.to_tex(); tex.t().color_space = ColorSpace::Srgb; tex.set_repeat_wrapping(); tex.t().repeat.set(4, 4);
    let floor = mesh(plane_geo!(20, 20), lambert!({ map: &tex }));
    floor.rotation().x = -std::f64::consts::PI / 2.0;
    scene.add(&floor);
    let b = mesh(box_geo!(1, 1, 1), lambert!({ color: 0xd9b03a, emissive: 0x2a1f08 }));
    b.set_pos(-1.5, 0.5, 0); scene.add(&b);
    let s = mesh(sphere_geo!(0.6, 32, 16), phong!({ color: 0x8080ff, specular: 0xffffff, shininess: 60 }));
    s.set_pos(1.5, 0.6, 0); scene.add(&s);
    let t = mesh(torus_geo!(0.4, 0.12, 12, 48), basic_mat!({ color: 0xff4020, transparent: true, opacity: 0.6 }));
    t.set_pos(0, 1.5, 0); scene.add(&t);
    scene.add(&ambient_light(0xa89a70, 0.78));
    scene.add(&hemisphere_light(0xd0c48c, 0xaaa176, 1.05));
    let pl = point_light!(0xfff2cf, 6.0, 8.0, 1.6); pl.set_pos(0, 2.5, 1.5); scene.add(&pl);
    let sl = spot_light!(0xffeec4, 8.0, 10.0, std::f64::consts::PI / 2.0, 1.0, 1.6); sl.set_pos(-1.5, 3.0, 0); scene.add(&sl);
    let tg = sl.target().unwrap(); tg.set_pos(-1.5, 0, 0); scene.add(&tg);
    let dust = points(Geo::from_points(&[v3(0, 1, 2), v3(0.5, 1.2, 2), v3(-0.5, 0.8, 2)]), &points_mat!({ size: 0.2, color: 0xffffff }));
    scene.add(&dust);
    let globals = RenderGlobals { background: Color::hex(0x070604), fog: Some((Color::hex(0x070604), 0.052)), pixel_ratio: 1.0, ..Default::default() };
    let (w, h) = (960usize, 540usize);
    let target = r.offscreen_target(w, h);
    let ops: Vec<OverlayOp> = if std::env::var("NO_OVERLAY").is_ok() { vec![] } else { vec![
        OverlayOp::Radial { center: [0.5, 0.5], radius: [0.7071, 0.7071], stops: vec![(0.55, [0.0, 0.0, 0.0, 0.0]), (1.0, [0.0, 0.0, 0.0, 0.85])], blend: OBlend::Normal, opacity: 1.0 },
    ] };
    let cb = r.render(&scene, &camera, &globals, &ops, &target, w, h);
    cb.commit();
    cb.waitUntilCompleted();
    let (tw, th, px) = r.read_texture(&target);
    let out = std::env::args().nth(1).unwrap_or("/tmp/rendertest.png".into());
    engine::png::write_rgba(&out, tw as u32, th as u32, &px).unwrap();
    println!("wrote {out} {tw}x{th} calls={} tris={} programs={}", r.info.calls, r.info.triangles, r.info.programs);
}
