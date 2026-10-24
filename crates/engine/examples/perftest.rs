//! Renderer stress: many meshes / materials / lights, timed offscreen frames.
use engine::prelude::*;
use engine::render::*;
use objc2_metal::MTLCommandBuffer;

fn main() {
    let device = objc2_metal::MTLCreateSystemDefaultDevice().expect("metal");
    let mut r = Renderer::new(device);
    let scene = new_scene();
    let camera = perspective_camera(74.0, 16.0 / 9.0, 0.05, 90.0);
    camera.set_pos(0, 1.6, 0);
    let mats: Vec<Mat> = (0..300).map(|i| lambert!({ color: 0x404040 + (i * 7919) as u32 % 0xbfbfbf })).collect();
    let geos = [box_geo!(0.3, 0.3, 0.3), cyl_geo!(0.1, 0.1, 0.4, 12), sphere_geo!(0.15, 12, 8)];
    let mut k = 0usize;
    for x in -30..30 {
        for z in -50..50 {
            let m = mesh(&geos[k % 3], &mats[k % 300]);
            m.set_pos(x as f64 * 0.8, 0.3 + (k % 5) as f64 * 0.2, z as f64 * 0.8);
            scene.add(&m);
            k += 1;
        }
    }
    let floor = mesh(plane_geo!(200, 200), lambert!({ color: 0xc8b070 }));
    floor.rotation().x = -std::f64::consts::PI / 2.0;
    scene.add(&floor);
    scene.add(&ambient_light(0xa89a70, 0.78));
    scene.add(&hemisphere_light(0xd0c48c, 0xaaa176, 1.05));
    for i in 0..60 {
        let l = if i < 18 {
            let l = spot_light!(0xffeec4, 9.0, 10.0, std::f64::consts::PI / 2.0, 1.0, 1.6);
            let t = l.target().unwrap();
            t.set_pos((i % 6) as f64 * 4.0 - 10.0, 0, (i / 6) as f64 * -4.0);
            scene.add(&t);
            l
        } else {
            point_light!(0xff8040, 0.5, 2.0, 2.0)
        };
        l.set_pos((i % 6) as f64 * 4.0 - 10.0, 3.0, (i / 6) as f64 * -4.0);
        scene.add(&l);
    }
    let globals =
        RenderGlobals { background: Color::hex(0x070604), fog: Some((Color::hex(0x070604), 0.03)), pixel_ratio: 1.0, ..Default::default() };
    let (w, h) = (2560usize, 1440usize);
    let target = r.offscreen_target(w, h);
    let mut times = vec![];
    for f in 0..60 {
        camera.rotation().y = f as f64 * 0.05;
        let t0 = std::time::Instant::now();
        let cb = r.render(&scene, &camera, &globals, &[], &target, w, h);
        cb.commit();
        cb.waitUntilCompleted();
        times.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "{} meshes: median {:.2} ms, p95 {:.2} ms (cpu encode {:.2} ms) calls {} tris {} lights {}",
        k, times[30], times[57], r.info.frame_ms, r.info.calls, r.info.triangles, r.info.lights
    );
}
