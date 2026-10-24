//! Smoke tests: build the Stairwell and the Loop with the game's own deps,
//! run them for many frames on wandering input, and dispose.
use engine::prelude::*;
use game::loop_mode::*;
use game::stairwell::*;
use std::f64::consts::PI;

fn capacity(o: &Obj) -> usize { match &o.r().kind { Kind::Instanced { inst, .. } => inst.matrices.len(), _ => 0 } }

#[test]
fn stairwell_smoke() {
    let scene = new_scene();
    let camera = perspective_camera(74.0, 1.0, 0.05, 90.0);
    camera.set_rotation_order(EulerOrder::YXZ);
    let mut s = Stairwell::new(StairDeps::game(None));
    scene.add(&s.group);
    let mut r = game::rng::Mulberry32::new(17);
    for flights in [2, 3, 10, 40] {
        s.enter(flights);
        assert!(s.group.visible());
        let (mut yaw, mut pitch) = (PI, -0.2);
        let mut events = 0;
        for f in 0..6000 {
            if f % 90 == 0 { yaw += (r.next() - 0.5) * 3.0; pitch = (r.next() - 0.5) * 1.2; }
            let fwd = if f % 500 < 450 { 1.0 } else { -1.0 };
            let inp = StairInput::with_camera(fwd, (r.next() - 0.5) * 0.4, [1.2, 2.1, 3.4][f % 3], yaw, pitch, &camera);
            let out = s.update(1.0 / 60.0, &inp);
            camera.position().set(out.x, out.y, out.z);
            camera.rotation().set(pitch, yaw, 0);
            events += out.events.len();
            assert!(out.seg >= -1 && out.seg <= 2 * (s.flights() - 1) + 1, "seg {} out of range", out.seg);
            assert_eq!(s.locate(out.seg, out.x, out.z, 0.3), Some(out.seg), "standing where it cannot stand");
            assert!(out.x.is_finite() && out.y.is_finite() && out.z.is_finite());
            assert!(s.landing() >= 0 && s.landing() <= s.flights());
            assert!(s.depth_m() >= -1e-9);
        }
        // the debug hooks the game uses: forceStairFace, stairPlace
        s.st.encounter_at.clear();
        if s.spawn_face(FaceKind::Below) { s.st.face.as_mut().unwrap().stare = -1e9; assert!(s.face.visible()); }
        s.st.face = None; s.face.set_visible(false);
        let seg = 2 * (s.flights() - 1) + 1;
        let lp = s.landing_point(seg, 0.5);
        s.st.seg = seg; s.st.x = 0.0; s.st.z = lp.z; s.st.y = s.seg_y(seg, 0.0, lp.z);
        let out = s.update(1.0 / 60.0, &StairInput::with_camera(0.0, 0.0, 0.0, 0.0, 0.0, &camera));
        assert!(out.near_exit, "the door at the bottom ({} flights)", flights);
        let _ = events;
    }
    // camera_world_direction is three's (−Z for a camera)
    camera.position().set(0, 0, 0); camera.rotation().set(0, 0, 0);
    let d = camera_world_direction(&camera);
    assert!((d.z + 1.0).abs() < 1e-12);
    s.dispose();
    assert!(!s.group.visible());
    assert!(s.group.parent().is_none());
}

#[test]
fn loop_smoke() {
    let tex = || Some(Tex::data_rgba(vec![200; 4], 1, 1));
    let scene = new_scene();
    let mut lp = LoopMode::new(LoopDeps::game(LoopTextures { wall: tex(), wall_dirty: tex(), carpet: tex(), ceiling: tex(), panel: tex() }));
    scene.add(&lp.group);
    lp.enter(2);
    let mut r = game::rng::Mulberry32::new(23);
    let mut yaw = lp.state().yaw;
    let mut seams = 0;
    let check = |lp: &LoopMode| {
        for (name, m) in &lp.meshes { assert!(m.instance_count() <= capacity(m), "{} over capacity", name); }
    };
    for f in 0..20000 {
        if f % 60 == 0 { yaw += (r.next() - 0.5) * 2.0; }
        if f % 2500 == 0 && f > 0 {
            let lap = [-100, -3, 0, 2, 5, 6, 9, 40, 1000][(r.next() * 9.0) as usize];
            lp.place(lap, r.next() * LOOP_STRIP, (r.next() - 0.5) * 1.0, None);
            check(&lp);
        }
        let fwd = if f % 700 < 600 { 1.0 } else { -1.0 };
        let out = lp.update(1.0 / 60.0, &LoopInput { fwd, strafe: (r.next() - 0.5) * 0.6, speed: [2.0, 3.6, 6.4][f % 3], yaw });
        seams += out.events.iter().filter(|e| matches!(e, LoopEvent::Seam { .. })).count();
        assert!(out.z >= 0.0 && out.z < LOOP_STRIP, "z {} left the strip", out.z);
        assert!(out.x > -LOOP::W / 2.0 && out.x < LOOP::W / 2.0);
        assert!(out.lap >= 0 && lp.state().lap >= LAP_MIN);
        assert!(out.light_count <= 6);
        assert!(out.env.fog > 0.0 && out.env.reverb >= 0 && out.env.reverb <= 6);
        check(&lp);
    }
    // pure functions at the extremes
    assert!(lap_look(-5).exit && lap_look(1000).bare);
    assert!(loop_drain(100).is_finite());
    assert_eq!(loop_reverb(100.0).er, 3);
    let _ = seams;
    let (ex, ez) = lp.exit_door_at();
    lp.place(0, ez, ex + 0.3, Some(PI / 2.0));
    let out = lp.update(1.0 / 60.0, &LoopInput { yaw: PI / 2.0, ..Default::default() });
    assert!(out.near_exit);
    lp.dispose();
    scene.remove(&lp.group);
    let mut n = 0;
    scene.traverse(&mut |o| if o.name().starts_with("loop") { n += 1; });
    assert_eq!(n, 0);
}
