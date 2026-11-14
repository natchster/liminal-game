//! `game::modes::stairwell` against recorded reference data (fixtures in `tests/fixtures/`).
#[path = "json_common/mod.rs"]
mod common;
use common::*;
use engine::prelude::*;
use game::base::numeric::*;
use game::modes::stairwell::*;
use std::cell::RefCell;
use std::rc::Rc;

const TOL: f64 = 1e-9;

type TexLog = Rc<RefCell<Vec<(f64, f64, bool)>>>;
/// the game's `canvas_tex`, logging (w, h, repeat) to compare with the recorded calls
fn logging_canvas_tex() -> (CanvasTexFn, TexLog) {
    let log: TexLog = Rc::new(RefCell::new(vec![]));
    let l2 = log.clone();
    let inner = game_canvas_tex();
    let f: CanvasTexFn = Rc::new(move |w: f64, h: f64, rep: bool, draw: &mut dyn FnMut(&mut Canvas, f64, f64)| {
        l2.borrow_mut().push((w, h, rep));
        (*inner)(w, h, rep, draw)
    });
    (f, log)
}
fn make(seed: u32, face_geometry: Option<Geo>) -> (Stairwell, Rc<std::cell::Cell<u64>>, TexLog) {
    let (rng, n) = counted_rng(seed);
    let (ct, log) = logging_canvas_tex();
    (Stairwell::new(StairDeps { canvas_tex: ct, rng, face_geometry }), n, log)
}

fn geo_params(g: &Geo) -> Vec<f64> {
    match g.params() {
        GeoParams::Box { width, height, depth, .. } => vec![width, height, depth],
        GeoParams::Cylinder { radius_top, radius_bottom, height, radial, height_seg, open_ended, theta_start, theta_length } => {
            vec![radius_top, radius_bottom, height, radial as f64, height_seg as f64, open_ended as i32 as f64, theta_start, theta_length]
        }
        GeoParams::Plane { width, height, .. } => vec![width, height],
        GeoParams::Sphere { radius, ws, hs, .. } => vec![radius, ws as f64, hs as f64],
        GeoParams::Circle { radius, segments, .. } => vec![radius, segments as f64],
        GeoParams::Torus { radius, tube, radial, tubular, arc } => vec![radius, tube, radial as f64, tubular as f64, arc],
        _ => vec![],
    }
}
fn geo_kind(g: &Geo) -> &'static str {
    match g.params() {
        GeoParams::Box { .. } => "box",
        GeoParams::Cylinder { .. } => "cyl",
        GeoParams::Plane { .. } => "plane",
        GeoParams::Sphere { .. } => "sphere",
        GeoParams::Circle { .. } => "circle",
        GeoParams::Torus { .. } => "torus",
        _ => "buffer",
    }
}
fn instance_matrices(o: &Obj) -> Vec<Mat4> {
    match &o.r().kind {
        Kind::Instanced { inst, .. } => inst.matrices[..inst.count].to_vec(),
        _ => vec![],
    }
}
/// compare a scene tree with its recorded dump
fn assert_tree(o: &Obj, rec: &Json, path: &str) {
    let kind = if o.is_instanced() {
        "inst"
    } else if o.is_mesh() {
        "mesh"
    } else {
        "group"
    };
    assert_eq!(kind, rec.get("k").s(), "{path}: kind");
    assert_eq!(o.visible(), rec.get("v").b(), "{path}: visible");
    let p = o.pos();
    assert_vec_close(&[p.x, p.y, p.z], &rec.get("p").fs(), TOL, &format!("{path}: position"));
    let r = o.rot();
    assert_vec_close(&[r.x, r.y, r.z], &rec.get("r").fs(), TOL, &format!("{path}: rotation"));
    if o.is_mesh() {
        let g = o.geo();
        let jg = rec.get("g");
        assert_eq!(geo_kind(&g), jg.at(0).s(), "{path}: geometry type");
        let want: Vec<f64> = jg.arr()[1..].iter().map(|v| v.f()).collect();
        assert_vec_close(&geo_params(&g), &want, TOL, &format!("{path}: geometry params"));
        assert_eq!(g.vertex_count() as i64, rec.get("vc").i(), "{path}: vertex count");
        let ic = g.r().index.as_ref().map(|i| i.len() as i64);
        assert_eq!(ic, if rec.get("ic").is_null() { None } else { Some(rec.get("ic").i()) }, "{path}: index count");
        assert_mat(&o.mat(), rec.get("m"), &format!("{path}: material"));
    }
    if o.is_instanced() {
        assert_eq!(o.instance_count() as i64, rec.get("count").i(), "{path}: instance count");
        let ms = instance_matrices(o);
        for (i, m) in ms.iter().enumerate() {
            let want = rec.get("mats").at(i).fs();
            let got: Vec<f64> = m.e.iter().map(|&v| f32r(v)).collect();
            assert_vec_close(&got, &want, 1e-7, &format!("{path}: instance {i}"));
        }
    }
    let kids = o.children();
    let jk = if rec.has("c") { rec.get("c").arr().to_vec() } else { vec![] };
    assert_eq!(kids.len(), jk.len(), "{path}: child count");
    for (i, (k, j)) in kids.iter().zip(jk.iter()).enumerate() {
        assert_tree(k, j, &format!("{path}/{i}"));
    }
}

#[test]
fn stairwell_constants() {
    let fx = Json::load("stairwell.json");
    let c = fx.get("consts");
    assert_eq!(STAIR_FL, c.get("FL").f());
    assert_eq!(STAIR_FLIGHT_DROP, c.get("DROP").f());
    assert_eq!(STAIR_FLIGHT_DROP, c.get("FLIGHT_DROP").f());
    assert_eq!(STAIR::STEPS as f64, c.get("STEPS").f());
    assert_eq!(STAIR::RISE, c.get("RISE").f());
    assert_eq!(STAIR::RUN, c.get("RUN").f());
    assert_eq!(STAIR::HALF, c.get("HALF").f());
    assert_eq!(STAIR::SPINE, c.get("SPINE").f());
    assert_eq!(STAIR::R, c.get("R").f());
    assert_eq!(STAIR::LIGHT_STEPS as f64, c.get("LIGHT_STEPS").f());
}

#[test]
fn stairwell_build_matches_reference() {
    let fx = Json::load("stairwell.json");
    let b = fx.get("build");
    let (s, n, log) = make(1234, None);
    assert_eq!(n.get() as i64, b.get("draws").i(), "rng draws while building (canvas draws)");
    let texs: Vec<(f64, f64, bool)> = b.get("texs").arr().iter().map(|t| (t.at(0).f(), t.at(1).f(), t.at(2).b())).collect();
    assert_eq!(*log.borrow(), texs, "canvas_tex calls");
    assert_tree(&s.group, b.get("tree"), "group");
    assert_geo(&s.face.geo(), b.get("face"), "face geometry");
    assert_mat(&s.face.mat(), b.get("faceMat"), "face material");
    assert_eq!(s.face.mat().r().patch, Patch::None, "the procedural face has no patch");
    // the wall texture repeats 2 × 3
    let wall = s.chunks[0].group.child(4).unwrap().mat();
    let t = wall.map_v().unwrap();
    assert_eq!((t.r().repeat.x, t.r().repeat.y), (2.0, 3.0));
    assert_eq!(t.r().wrap_s, Wrap::Repeat);
}

#[test]
fn stairwell_face_geometry_material() {
    let fx = Json::load("stairwell.json");
    let fg = fx.get("faceGeometry");
    let painted = box_geo!(0.2, 0.3, 0.2);
    {
        let n = painted.vertex_count();
        painted.set_color(vec![0.5; n * 3], 3);
    }
    let (s2, _, _) = make(5, Some(painted.clone()));
    assert_mat(&s2.face.mat(), fg.get("painted"), "painted face");
    assert_eq!(s2.face.mat().r().patch, Patch::Stairwell { kind: 0 });
    assert!(s2.face.geo().ptr_eq(&painted));
    let (s3, _, _) = make(5, Some(box_geo!(0.2, 0.3, 0.2)));
    assert_mat(&s3.face.mat(), fg.get("plain"), "unpainted face");
    assert_eq!(s3.face.mat().r().patch, Patch::Stairwell { kind: 0 });
}

#[test]
fn stairwell_locate_and_seg_y() {
    let fx = Json::load("stairwell.json");
    let (mut s, n, _) = make(1234, None);
    s.enter(10);
    assert_eq!(n.get() as i64, fx.get("enter10Draws").i());
    let p = fx.get("probes");
    let (xs, zs, segs, rs) = (p.get("xs").fs(), p.get("zs").fs(), p.get("segs").fs(), p.get("rs").fs());
    let loc = p.get("locate").arr();
    let mut k = 0;
    let mut checked = 0;
    for &r in &rs {
        for &seg in &segs {
            for &x in &xs {
                for &z in &zs {
                    let got = s.locate(seg as i32, x, z, r).map_or(-99, |v| v as i64);
                    assert_eq!(got, loc[k].i(), "locate(seg {seg}, x {x}, z {z}, r {r})");
                    k += 1;
                    if got != -99 {
                        checked += 1;
                    }
                }
            }
        }
    }
    assert!(checked > 1000, "too few open probes");
    let sy = p.get("segY").arr();
    let mut k = 0;
    for &seg in &segs {
        for &z in &zs {
            assert_close(s.seg_y(seg as i32, 0.0, z), sy[k].f(), TOL, &format!("seg_y({seg}, {z})"));
            k += 1;
        }
    }
}

#[test]
fn stairwell_enter_matches_reference() {
    let fx = Json::load("stairwell.json");
    let (mut s, n, _) = make(1234, None);
    s.enter(10); // the probes' enter, to keep the rng stream in step
    for e in fx.get("enters").arr() {
        let flights = engine::math::to_i32(e.get("flights").f());
        s.enter(flights);
        let w = format!("enter({})", e.get("flights").f());
        assert_eq!(n.get() as i64, e.get("draws").i(), "{w}: rng draws");
        assert_eq!(s.flights() as i64, e.get("flightsOut").i(), "{w}: flights");
        let enc: Vec<i64> = e.get("encounterAt").arr().iter().map(|v| v.i()).collect();
        assert_eq!(s.st.encounter_at.iter().map(|&v| v as i64).collect::<Vec<_>>(), enc, "{w}: encounter_at");
        for (i, l) in e.get("leds").arr().iter().enumerate() {
            let m = &s.leds[i];
            assert_eq!(m.visible(), l.get("v").b(), "{w}: led {i} visible");
            let p = m.pos();
            assert_vec_close(&[p.x, p.y, p.z], &l.get("p").fs(), TOL, &format!("{w}: led {i} position"));
            assert_eq!(mat_type(&m.mat()) == "basic", l.get("on").b(), "{w}: led {i} material");
            assert_eq!(s.led_blink[i], l.get("blink").b(), "{w}: led {i} blink");
        }
        for (i, (o, j)) in [&s.puddle, &s.gouge, &s.hole, &s.hole_light].iter().zip(e.get("misc").arr()).enumerate() {
            assert_eq!(o.visible(), j.get("v").b(), "{w}: marker {i} visible");
            let p = o.pos();
            let r = o.rot();
            assert_vec_close(&[p.x, p.y, p.z], &j.get("p").fs(), TOL, &format!("{w}: marker {i} position"));
            assert_vec_close(&[r.x, r.y, r.z], &j.get("r").fs(), TOL, &format!("{w}: marker {i} rotation"));
        }
        for (i, (o, j)) in [&s.exit_door, &s.exit_glow, &s.exit_handle, &s.floor].iter().zip(e.get("bottom").arr()).enumerate() {
            let p = o.pos();
            let r = o.rot();
            assert_vec_close(&[p.x, p.y, p.z], &j.get("p").fs(), TOL, &format!("{w}: bottom {i} position"));
            assert_vec_close(&[r.x, r.y, r.z], &j.get("r").fs(), TOL, &format!("{w}: bottom {i} rotation"));
        }
        for (i, j) in e.get("chunks").arr().iter().enumerate() {
            let c = &s.chunks[i];
            assert_eq!(c.group.visible(), j.get("v").b(), "{w}: chunk {i} visible");
            if c.group.visible() {
                assert_close(c.group.pos().y, j.get("y").f(), TOL, &format!("{w}: chunk {i} y"));
                assert_eq!(c.right.visible(), j.get("right").b(), "{w}: chunk {i} right");
                assert_eq!(c.near.visible(), j.get("near").b(), "{w}: chunk {i} near");
            }
        }
        assert_eq!(s.top.visible(), e.get("top").b(), "{w}: top");
        assert_eq!(s.landing() as i64, e.get("landing").i(), "{w}: landing");
        assert_close(s.depth_m(), e.get("depth").f(), TOL, &format!("{w}: depth"));
        let v = s.voice_pos();
        assert_vec_close(&[v.x, v.y, v.z], &e.get("voice").fs(), TOL, &format!("{w}: voice_pos"));
        assert!(s.group.visible());
    }
}

#[test]
fn stairwell_chunks_follow_the_player() {
    let fx = Json::load("stairwell.json");
    let (mut s, _, _) = make(1234, None);
    s.enter(40);
    let cam = StairInput { camera_pos: Vec3::new(0.0, 1.62, -0.5), camera_dir: Vec3::new(0.0, 0.0, 1.0), ..Default::default() };
    for j in fx.get("chunkBySeg").arr() {
        let seg = j.get("seg").i() as i32;
        s.st.seg = seg;
        s.st.encounter_at.clear();
        s.st.behind_done = true;
        s.update(0.0, &cam);
        for (i, c) in j.get("chunks").arr().iter().enumerate() {
            let ch = &s.chunks[i];
            assert_eq!(ch.group.visible(), c.at(0).b(), "seg {seg}: chunk {i} visible");
            if ch.group.visible() {
                assert_close(ch.group.pos().y, c.at(1).f(), TOL, &format!("seg {seg}: chunk {i} y"));
                assert_eq!(ch.right.visible(), c.at(2).b(), "seg {seg}: chunk {i} right");
                assert_eq!(ch.near.visible(), c.at(3).b(), "seg {seg}: chunk {i} near");
            }
        }
        assert_eq!(s.top.visible(), j.get("top").b(), "seg {seg}: top");
        assert_eq!(s.landing() as i64, j.get("landing").i(), "seg {seg}: landing");
    }
}

fn event_row(e: &StairEvent) -> (String, f64, f64, String) {
    match e {
        StairEvent::FaceSeen { dist } => ("faceSeen".into(), *dist, f64::NAN, String::new()),
        StairEvent::FaceJerk { n, dist } => ("faceJerk".into(), *dist, *n as f64, String::new()),
        StairEvent::FaceGone { reason } => ("faceGone".into(), f64::NAN, f64::NAN, reason.as_str().into()),
        other => (other.type_name().into(), f64::NAN, f64::NAN, String::new()),
    }
}

#[test]
fn stairwell_scripted_descent_matches_reference() {
    let fx = Json::load("stairwell.json");
    let w = fx.get("walk");
    let (mut s, n, _) = make(w.get("seed").i() as u32, None);
    s.enter(w.get("flights").i() as i32);
    let mut seen_types = std::collections::BTreeSet::new();
    for (f, rec) in w.get("steps").arr().iter().enumerate() {
        let i = rec.get("i").fs();
        let inp = StairInput {
            fwd: i[1],
            strafe: i[2],
            speed: i[3],
            yaw: i[4],
            pitch: i[5],
            camera_pos: Vec3::new(i[6], i[7], i[8]),
            camera_dir: Vec3::new(i[9], i[10], i[11]),
        };
        let out = s.update(i[0], &inp);
        let o = rec.get("o").fs();
        let wh = format!("frame {f}");
        assert_close(out.x, o[0], TOL, &format!("{wh} x"));
        assert_close(out.y, o[1], TOL, &format!("{wh} y"));
        assert_close(out.z, o[2], TOL, &format!("{wh} z"));
        assert_eq!(out.seg as f64, o[3], "{wh} seg");
        assert_eq!(out.near_exit, o[4] != 0.0, "{wh} near_exit");
        assert_eq!(s.st.deepest_seg as f64, o[5], "{wh} deepest_seg");
        assert_close(s.st.step_phase, o[6], TOL, &format!("{wh} step_phase"));
        assert_close(s.st.time, o[7], TOL, &format!("{wh} time"));
        let want_ev = if rec.has("e") { rec.get("e").arr().to_vec() } else { vec![] };
        assert_eq!(out.events.len(), want_ev.len(), "{} event count ({:?})", wh, out.events);
        for (e, j) in out.events.iter().zip(want_ev.iter()) {
            let (t, d, nn, reason) = event_row(e);
            seen_types.insert(t.clone());
            assert_eq!(t, j.at(0).s(), "{wh} event type");
            assert_close(d, j.at(1).f(), TOL, &format!("{wh} {t} dist"));
            assert_close(nn, j.at(2).f(), 0.0, &format!("{wh} {t} n"));
            assert_eq!(reason, if j.at(3).is_null() { "" } else { j.at(3).s() }, "{wh} {t} reason");
        }
        if rec.has("f") {
            let jf = rec.get("f").arr();
            let kind = s.st.face.as_ref().map(|f| f.kind.as_str());
            assert_eq!(kind, if jf[0].is_null() { None } else { Some(jf[0].s()) }, "{wh} face kind");
            assert_eq!(s.face.visible(), jf[1].b(), "{wh} face visible");
            let p = s.face.pos();
            let q = s.face.quat();
            assert_vec_close(&[p.x, p.y, p.z], &[jf[2].f(), jf[3].f(), jf[4].f()], TOL, &format!("{wh} face position"));
            assert_vec_close(&[q.x, q.y, q.z, q.w], &[jf[5].f(), jf[6].f(), jf[7].f(), jf[8].f()], 1e-8, &format!("{wh} face quaternion"));
            let (stare, jerks) = s.st.face.as_ref().map_or((0.0, 0.0), |f| (f.stare, f.jerks as f64));
            assert_close(stare, jf[9].f(), TOL, &format!("{wh} stare"));
            assert_eq!(jerks, jf[10].f(), "{wh} jerks");
        } else {
            assert!(s.st.face.is_none() && !s.face.visible(), "{wh}: no face expected");
        }
        if rec.has("c") {
            let c = rec.get("c").arr();
            for k in 0..4 {
                let ch = &s.chunks[k].group;
                if c[k].is_null() {
                    assert!(!ch.visible(), "{wh} chunk {k} hidden");
                } else {
                    assert!(ch.visible(), "{wh} chunk {k} shown");
                    assert_close(ch.pos().y, c[k].f(), TOL, &format!("{wh} chunk {k} y"));
                }
            }
            assert_eq!(s.top.visible(), c[4].b(), "{wh} top");
        }
    }
    assert_eq!(n.get() as i64, w.get("draws").i(), "rng draws over the walk");
    for t in ["faceAppear", "faceSeen", "faceJerk", "faceGone", "faceBehindArmed", "faceBehind"] {
        assert!(seen_types.contains(t), "the walk never produced {t}");
    }
}

#[test]
fn hypot_is_exact() {
    // `hypot` scales by the largest magnitude and sums with Kahan compensation (bit-reproducible)
    assert_eq!(hypot2(3.0, 4.0), 5.0);
    assert_eq!(hypot(&[]), 0.0);
    assert_eq!(hypot2(f64::INFINITY, f64::NAN), f64::INFINITY);
    assert!(hypot2(1.0, f64::NAN).is_nan());
    assert_eq!(hypot3(0.0, -0.0, 0.0), 0.0);
}
