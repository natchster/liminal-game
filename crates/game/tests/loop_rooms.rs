//! `game::modes::loop_rooms` against recorded reference data (fixtures in `tests/fixtures/`).
#[path = "json_common/mod.rs"]
mod common;
use common::*;
use engine::prelude::*;
use game::modes::loop_rooms::*;
use game::modes::stairwell::{game_canvas_tex, CanvasTexFn};
use std::cell::RefCell;
use std::rc::Rc;

const TOL: f64 = 1e-9;

type TexLog = Rc<RefCell<Vec<(f64, f64, bool)>>>;
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
fn dummy_tex() -> Tex {
    Tex::data_rgba(vec![255; 4], 1, 1)
}
fn all_textures() -> LoopTextures {
    LoopTextures {
        wall: Some(dummy_tex()),
        wall_dirty: Some(dummy_tex()),
        carpet: Some(dummy_tex()),
        ceiling: Some(dummy_tex()),
        panel: Some(dummy_tex()),
    }
}
fn make(seed: u32) -> (LoopMode, Rc<std::cell::Cell<u64>>, TexLog) {
    let (rng, n) = counted_rng(seed);
    let (ct, log) = logging_canvas_tex();
    (LoopMode::new(LoopDeps { canvas_tex: Some(ct), rng, textures: all_textures() }), n, log)
}
fn capacity(o: &Obj) -> usize {
    match &o.r().kind {
        Kind::Instanced { inst, .. } => inst.matrices.len(),
        _ => 0,
    }
}
fn instances(o: &Obj) -> (Vec<Mat4>, Vec<Color>) {
    match &o.r().kind {
        Kind::Instanced { inst, .. } => {
            (inst.matrices[..inst.count].to_vec(), inst.colors.as_ref().map(|c| c[..inst.count].to_vec()).unwrap_or_default())
        }
        _ => (vec![], vec![]),
    }
}

#[test]
fn loop_constants_and_pure_functions() {
    let fx = Json::load("loop.json");
    let c = fx.get("consts");
    let l = c.get("LOOP");
    assert_eq!(LOOP::ROOMS as f64, l.get("ROOMS").f());
    for (k, v) in [("W", LOOP::W), ("L", LOOP::L), ("H", LOOP::H), ("T", LOOP::T), ("DOOR_W", LOOP::DOOR_W), ("DOOR_H", LOOP::DOOR_H)] {
        assert_eq!(v, l.get(k).f(), "LOOP.{}", k);
    }
    for (k, v) in [
        ("VIEW", LOOP::VIEW),
        ("NEAR", LOOP::NEAR),
        ("ENTRY_ROOM", LOOP::ENTRY_ROOM),
        ("EXIT_ROOM", LOOP::EXIT_ROOM),
        ("FIGURE_ROOM", LOOP::FIGURE_ROOM),
        ("PICKUP_LAP", LOOP::PICKUP_LAP),
        ("FIGURE_LAP", LOOP::FIGURE_LAP),
        ("BARE_LAP", LOOP::BARE_LAP),
    ] {
        assert_eq!(v as f64, l.get(k).f(), "LOOP.{}", k);
    }
    assert_eq!(LOOP_PITCH, c.get("LOOP_PITCH").f());
    assert_eq!(LOOP_STRIP, c.get("LOOP_STRIP").f());
    assert_eq!(LAP_MIN as f64, c.get("LAP_MIN").f());
    let f = c.get("FURN");
    let d = f.get("desk");
    assert_eq!(
        [FURN.desk.x, FURN.desk.v, FURN.desk.w, FURN.desk.d, FURN.desk.h],
        [d.get("x").f(), d.get("v").f(), d.get("w").f(), d.get("d").f(), d.get("h").f()]
    );
    let cb = f.get("cabinet");
    assert_eq!(
        [FURN.cabinet.x, FURN.cabinet.v, FURN.cabinet.w, FURN.cabinet.d, FURN.cabinet.h],
        [cb.get("x").f(), cb.get("v").f(), cb.get("w").f(), cb.get("d").f(), cb.get("h").f()]
    );
    for (seat, j) in [(FURN.desk_chair, f.get("deskChair")), (FURN.visit_chair, f.get("visitChair"))] {
        assert_eq!([seat.x, seat.v, seat.yaw, seat.turned], [j.get("x").f(), j.get("v").f(), j.get("yaw").f(), j.get("turned").f()]);
    }
    assert_eq!(
        [FURN.figure.x, FURN.figure.v, FURN.figure.yaw],
        [f.get("figure").get("x").f(), f.get("figure").get("v").f(), f.get("figure").get("yaw").f()]
    );
    assert_eq!([FURN.clock_v, FURN.clock_y], [f.get("clock").get("v").f(), f.get("clock").get("y").f()]);
    assert_eq!([FURN.plant_x, FURN.plant_v], [f.get("plant").get("x").f(), f.get("plant").get("v").f()]);
    assert_eq!(FURN.entry_door_v, f.get("entryDoor").get("v").f());
    assert_eq!(FURN.exit_door_v, f.get("exitDoor").get("v").f());
    for (sp, j) in [(FURN.almond, f.get("almond")), (FURN.battery, f.get("battery"))] {
        assert_eq!(
            [sp.x, sp.y, sp.v, sp.bx, sp.by, sp.bv],
            [j.get("x").f(), j.get("y").f(), j.get("v").f(), j.get("bx").f(), j.get("by").f(), j.get("bv").f()]
        );
    }

    for row in fx.get("lapLook").arr() {
        let r = row.fs();
        let lk = lap_look(r[0] as i32);
        let got = [
            lk.lap as f64,
            lk.tint[0],
            lk.tint[1],
            lk.tint[2],
            lk.light[0],
            lk.light[1],
            lk.light[2],
            lk.bright,
            lk.flicker,
            lk.turned as i32 as f64,
            lk.red as i32 as f64,
            lk.stains as i32 as f64,
            lk.dirty as i32 as f64,
            lk.figure as i32 as f64,
            lk.bare as i32 as f64,
            lk.pickups as i32 as f64,
            lk.exit as i32 as f64,
        ];
        assert_vec_close(&got, &r[1..], TOL, &format!("lap_look({})", r[0]));
    }
    for row in fx.get("pickupAt").arr() {
        let (lap, room, k) = (row.at(0).i() as i32, row.at(1).i() as i32, row.at(2).i());
        let got = match pickup_at(lap, room) {
            None => 0,
            Some(PickupKind::Almond) => 1,
            Some(PickupKind::Battery) => 2,
        };
        assert_eq!(got, k, "pickup_at({}, {})", lap, room);
    }
    for row in fx.get("loopReverb").arr() {
        let r = row.fs();
        let v = loop_reverb(r[0]);
        assert_vec_close(&[v.t60, v.damp, v.wet, v.pre, v.er as f64], &r[1..], TOL, &format!("loop_reverb({})", r[0]));
    }
    for row in fx.get("loopHumCents").arr() {
        assert_close(loop_hum_cents(row.at(0).f()), row.at(1).f(), TOL, "loopHumCents");
    }
    for row in fx.get("loopDrain").arr() {
        assert_close(loop_drain(row.at(0).i() as i32), row.at(1).f(), TOL, &format!("loop_drain({})", row.at(0).f()));
    }
}

#[test]
fn loop_build_matches_reference() {
    let fx = Json::load("loop.json");
    let b = fx.get("build");
    let (lp, n, log) = make(4242);
    assert_eq!(n.get() as i64, b.get("draws").i(), "rng draws while building");
    let texs: Vec<(f64, f64, bool)> = b.get("texs").arr().iter().map(|t| (t.at(0).f(), t.at(1).f(), t.at(2).b())).collect();
    assert_eq!(*log.borrow(), texs, "canvas_tex calls");
    assert_eq!(lp.group.name(), b.get("groupName").s());
    assert_eq!(lp.group.visible(), b.get("groupVisible").b());
    let kids = lp.group.children();
    let jm = b.get("meshes").arr();
    assert_eq!(kids.len(), jm.len(), "group children");
    for (o, j) in kids.iter().zip(jm.iter()) {
        let name = j.get("name").s();
        assert_eq!(o.name(), name);
        if !j.get("cap").is_null() {
            assert_eq!(capacity(o) as i64, j.get("cap").i(), "{}: capacity", name);
            assert!(!o.r().frustum_culled);
        }
        // (the seated figure's body is E9's, `bodies.bin` "seated", not the
        // recording's: its geometry is held by `tests/anomaly_bodies.rs`)
        if name != "loop:figure" {
            assert_geo(&o.geo(), j.get("geo"), name);
        }
        assert_mat(&o.mat(), j.get("mat"), name);
    }
    // the plain variants: no `canvas_tex`, no textures; and a wall only
    let bare = LoopMode::new(LoopDeps::default());
    for (o, j) in bare.group.children().iter().zip(fx.get("build_plain").arr()) {
        assert_eq!(o.name(), j.get("name").s());
        assert_mat(&o.mat(), j.get("mat"), &format!("plain {}", o.name()));
    }
    let half = LoopMode::new(LoopDeps {
        canvas_tex: Some(game_canvas_tex()),
        rng: Box::new(game::base::rng::Mulberry32::new(1).into_fn()),
        textures: LoopTextures { wall: Some(dummy_tex()), ..Default::default() },
    });
    for (o, j) in half.group.children().iter().zip(fx.get("build_half").arr()) {
        assert_eq!(o.name(), j.get("name").s());
        assert_mat(&o.mat(), j.get("mat"), &format!("half {}", o.name()));
    }
}

trait IntoFn {
    fn into_fn(self) -> Box<dyn FnMut() -> f64>;
}
impl IntoFn for game::base::rng::Mulberry32 {
    fn into_fn(mut self) -> Box<dyn FnMut() -> f64> {
        Box::new(move || self.next())
    }
}

fn counts_json(c: &[(&'static str, usize)]) -> Vec<(String, i64)> {
    c.iter().map(|(k, v)| (k.to_string(), *v as i64)).collect()
}

#[test]
fn loop_windows_match_reference() {
    let fx = Json::load("loop.json");
    let (mut lp, _, _) = make(4242);
    lp.enter(2);
    let snaps = fx.get("snaps").arr();
    let places =
        [(5, 13.0, 0.0), (-3, 1.0, 0.0), (7, 25.9, 0.1), (0, 2.5, 0.0), (3, 0.01, -0.4), (12, 14.5, 1.2), (-5, 20.0, 0.0), (4, 7.8, 0.3)];
    for (k, s) in snaps.iter().enumerate() {
        if k > 0 {
            let (lap, z, x) = places[k - 1];
            lp.place(lap, z, x, Some(0.3));
        }
        let w = s.get("label").s();
        let st = s.get("st");
        assert_eq!(lp.st.lap as i64, st.get("lap").i(), "{}: lap", w);
        assert_vec_close(
            &[lp.st.x, lp.st.z, lp.st.yaw],
            &[st.get("x").f(), st.get("z").f(), st.get("yaw").f()],
            TOL,
            &format!("{}: x z yaw", w),
        );
        assert_eq!(lp.st.entry_door, st.get("entryDoor").b(), "{}: entry_door", w);
        assert_eq!(lp.st.started, st.get("started").b(), "{}: started", w);
        assert_eq!(lp.lap() as i64, s.get("lap").i(), "{}: reported lap", w);
        assert_eq!(lp.room() as i64, s.get("room").i(), "{}: room", w);
        let want: Vec<(String, i64)> = s.get("counts").arr().iter().map(|c| (c.at(0).s().to_string(), c.at(1).i())).collect();
        assert_eq!(counts_json(&lp.counts()), want, "{}: counts", w);
        assert_eq!(lp.draw_calls() as i64, s.get("drawCalls").i(), "{}: draw_calls", w);
        for (j, (name, m)) in s.get("inst").arr().iter().zip(lp.meshes.iter()) {
            assert_eq!(format!("loop:{}", name), j.get("name").s());
            let (ms, cs) = instances(m);
            assert_eq!(ms.len() as i64, j.get("count").i(), "{}: {} count", w, name);
            for i in 0..ms.len() {
                let got: Vec<f64> = ms[i].e.iter().map(|&v| f32r(v)).collect();
                assert_vec_close(&got, &j.get("m").at(i).fs(), 1e-7, &format!("{}: {} matrix {}", w, name, i));
                let c = cs[i];
                assert_vec_close(
                    &[f32r(c.r), f32r(c.g), f32r(c.b)],
                    &j.get("c").at(i).fs(),
                    1e-7,
                    &format!("{}: {} colour {}", w, name, i),
                );
            }
        }
        let g = s.get("ghost").fs();
        let gd = &lp.ghost_door;
        let (p, r) = (gd.pos(), gd.rot());
        assert_vec_close(&[gd.visible() as i32 as f64, p.x, p.y, p.z, r.x, r.y, r.z], &g, TOL, &format!("{}: ghost door", w));
    }
}

#[test]
fn loop_collision_and_sight_probes() {
    let fx = Json::load("loop.json");
    let p = fx.get("probes");
    let (mut lp, _, _) = make(4242);
    lp.enter(2);
    let blocked = p.get("blocked").arr();
    let look = p.get("lookLap").arr();
    let (mut k, mut kl) = (0, 0);
    for lap in [0, 2, 4, 5, 6, 8] {
        lp.place(lap, 13.0, 0.0, Some(0.0));
        for ix in 0..=26 {
            for iz in 0..=66 {
                let x = -2.6 + ix as f64 * 0.2;
                let z = -0.3 + iz as f64 * 0.4;
                let got = if lp.blocked(x, z, 0.3) {
                    1
                } else if lp.blocked(x, z, 0.1) {
                    2
                } else {
                    0
                };
                assert_eq!(got, blocked[k].i(), "lap {}: blocked({}, {})", lap, x, z);
                k += 1;
            }
        }
        for o in -12..=12 {
            assert_eq!(lp.look_lap_of(o) as i64, look[kl].i(), "lap {}: look_lap_of({})", lap, o);
            kl += 1;
        }
    }
    for c in p.get("clear").arr() {
        let v = c.fs();
        assert_eq!(lp.clear_line(v[0], v[1], v[2], v[3]), v[4] != 0.0, "clear_line({:?})", &v[..4]);
    }
}

fn ev_row(e: &LoopEvent) -> Vec<Json> {
    let num = |v: f64| Json::Num(v);
    let s = |v: &str| Json::Str(v.into());
    match e {
        LoopEvent::Seam { dir, lap } => vec![s("seam"), num(*dir as f64), num(*lap as f64), Json::Null, Json::Null, Json::Null, Json::Null],
        LoopEvent::Pickup { kind, lap, room } => {
            vec![s("pickup"), Json::Null, num(*lap as f64), s(kind.as_str()), num(*room as f64), Json::Null, Json::Null]
        }
        LoopEvent::EntryGone => vec![s("entryGone"), Json::Null, Json::Null, Json::Null, Json::Null, Json::Null, Json::Null],
        LoopEvent::FigureSeen { dist, moved } => {
            vec![s("figureSeen"), Json::Null, Json::Null, Json::Null, Json::Null, num(*dist), num(*moved)]
        }
        LoopEvent::FigureNearer { dist, moved } => {
            vec![s("figureNearer"), Json::Null, Json::Null, Json::Null, Json::Null, num(*dist), num(*moved)]
        }
    }
}

#[test]
fn loop_scripted_walk_matches_reference() {
    let fx = Json::load("loop.json");
    let w = fx.get("walk");
    let (rng, n) = counted_rng(w.get("seed").i() as u32);
    let (ct, _) = logging_canvas_tex();
    let mut lp = LoopMode::new(LoopDeps { canvas_tex: Some(ct), rng, textures: all_textures() });
    let recs = w.get("recs").arr();
    let mut ri = 0;
    let mut frame = 0;
    let mut types = std::collections::BTreeSet::new();
    let mut max_touch = 0.0f64;
    for step in w.get("script").arr() {
        let rec = &recs[ri];
        ri += 1;
        if step.has("enter") {
            lp.enter(step.get("enter").i() as i32);
            let e = rec.get("entered").fs();
            assert_vec_close(&[lp.st.lap as f64, lp.st.x, lp.st.z, lp.st.yaw], &e, TOL, "enter");
            continue;
        }
        if step.has("place") {
            let p = step.get("place").fs();
            lp.place(p[0] as i32, p[1], p[2], Some(p[3]));
            let e = rec.get("placed").fs();
            assert_vec_close(&[lp.st.lap as f64, lp.st.z, lp.st.x], &e, TOL, "place");
            continue;
        }
        let u = step.get("u").fs();
        let out = lp.update(u[0], &LoopInput { fwd: u[1], strafe: u[2], speed: u[3], yaw: u[4] });
        let wh = format!("frame {}", frame);
        let o = rec.get("o").fs();
        assert_close(out.x, o[0], TOL, &format!("{} x", wh));
        assert_close(out.z, o[1], TOL, &format!("{} z", wh));
        assert_eq!(out.lap as f64, o[2], "{} lap", wh);
        assert_eq!(out.room as f64, o[3], "{} room", wh);
        assert_eq!(out.near_exit, o[4] != 0.0, "{} near_exit", wh);
        assert_close(out.touch, o[5], TOL, &format!("{} touch", wh));
        max_touch = max_touch.max(out.touch);
        assert_eq!(out.light_count as f64, o[6], "{} light_count", wh);
        assert_eq!(lp.st.lap as f64, o[7], "{} internal lap", wh);
        assert_eq!(lp.st.seams as f64, o[8], "{} seams", wh);
        assert_eq!(lp.st.entry_door, o[9] != 0.0, "{} entry_door", wh);
        let want_ev = if rec.has("e") { rec.get("e").arr().to_vec() } else { vec![] };
        assert_eq!(out.events.len(), want_ev.len(), "{} events {:?}", wh, out.events);
        for (e, j) in out.events.iter().zip(want_ev.iter()) {
            types.insert(e.type_name());
            let row = ev_row(e);
            for (k, (a, b)) in row.iter().zip(j.arr().iter()).enumerate() {
                match (a, b) {
                    (Json::Num(x), Json::Num(y)) => assert_close(*x, *y, TOL, &format!("{} event {} field {}", wh, e.type_name(), k)),
                    (a, b) => assert_eq!(a, b, "{} event {} field {}", wh, e.type_name(), k),
                }
            }
        }
        if rec.has("env") {
            let e = &out.env;
            let got = [
                e.fog_r,
                e.fog_g,
                e.fog_b,
                e.fog,
                e.amb,
                e.hemi,
                e.sky_r,
                e.sky_g,
                e.sky_b,
                e.ground_r,
                e.ground_g,
                e.ground_b,
                e.hum,
                e.hum_cents,
                e.reverb as f64,
                e.drain,
                e.glow,
                e.lam,
            ];
            assert_vec_close(&got, &rec.get("env").fs(), TOL, &format!("{} env", wh));
            assert_eq!(lp.env(), e);
            for (k, l) in rec.get("l").arr().iter().enumerate() {
                let q = &out.lights[k];
                assert_vec_close(&[q.x, q.y, q.z, q.i, q.r, q.g, q.b], &l.fs(), TOL, &format!("{} light {}", wh, k));
            }
            let f = &lp.fig;
            let got = [
                f.active as i32 as f64,
                f.lap as f64,
                f.gz,
                f.x,
                f.yaw,
                f.seen as i32 as f64,
                f.unseen_t,
                f.moved,
                f.inst as f64,
                f.sightings as f64,
            ];
            assert_vec_close(&got, &rec.get("fig").fs(), TOL, &format!("{} figure", wh));
            let st = rec.get("st").fs();
            assert_vec_close(&[lp.st.time, lp.st.entry_t, lp.st.entry_unseen], &st[..3], TOL, &format!("{} state", wh));
            let taken: Vec<f64> = lp.st.taken.iter().map(|&v| v as f64).collect();
            assert_eq!(taken, st[3..].to_vec(), "{} taken", wh);
        }
        if rec.has("c") {
            let want: Vec<i64> = rec.get("c").arr().iter().map(|v| v.i()).collect();
            let got: Vec<i64> = lp.counts().iter().map(|c| c.1 as i64).collect();
            assert_eq!(got, want, "{} counts", wh);
            assert_eq!(lp.draw_calls() as i64, rec.get("dc").i(), "{} draw_calls", wh);
        }
        frame += 1;
    }
    assert_eq!(ri, recs.len());
    assert_eq!(frame as i64, w.get("frames").i());
    assert_eq!(n.get() as i64, w.get("draws").i(), "rng draws");
    for t in ["seam", "pickup", "entryGone", "figureSeen", "figureNearer"] {
        assert!(types.contains(t), "the walk never produced {}", t);
    }
    assert!(max_touch > 0.0, "the walk never touched the figure");
}

#[test]
fn loop_dispose_empties_the_group() {
    let (mut lp, _, _) = make(9);
    lp.enter(3);
    let scene = group();
    scene.add(&lp.group);
    lp.update(1.0 / 60.0, &LoopInput { fwd: 1.0, strafe: 0.0, speed: 3.6, yaw: std::f64::consts::PI });
    assert!(lp.draw_calls() > 0);
    lp.dispose();
    assert!(!lp.group.visible());
    assert_eq!(lp.group.child_count(), 0);
    assert_eq!(lp.counts(), vec![("ghostDoor", 0)]);
    // the glue removes the group itself (`scene.remove(&lp.group)`)
    scene.remove(&lp.group);
    let mut n = 0;
    scene.traverse(&mut |o| {
        if o.name().starts_with("loop") {
            n += 1;
        }
    });
    assert_eq!(n, 0, "loop leftovers");
}
