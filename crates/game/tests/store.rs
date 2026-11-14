//! `game::anomalies::store_kit` against recorded reference data (fixtures in `tests/fixtures/`).
#[path = "json_common/mod.rs"]
mod common;
use common::*;
use engine::prelude::*;
use game::anomalies::store_kit::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const TOL: f64 = 1e-9;

fn phase_rows(rows: &Json, h: &StoreHours, what: &str) {
    for r in rows.arr() {
        let t = r.at(0).f();
        let p = store_phase(t, h);
        let w = format!("{what} store_phase({t})");
        assert_eq!(p.phase.as_str(), r.at(1).s(), "{w}: phase");
        assert_close(p.left, r.at(2).f(), TOL, &format!("{w}: left"));
        assert_close(p.into, r.at(3).f(), TOL, &format!("{w}: into"));
        assert_eq!(p.cycle as i64, r.at(4).i(), "{w}: cycle");
        assert_close(p.len, r.at(5).f(), TOL, &format!("{w}: len"));
        let clock = store_clock(&p);
        assert_eq!(clock.as_deref(), if r.at(6).is_null() { None } else { Some(r.at(6).s()) }, "{w}: store_clock");
    }
}

#[test]
fn store_hours_and_clock() {
    let fx = Json::load("store.json");
    let h = fx.get("hours");
    assert_eq!(
        STORE_HOURS,
        StoreHours { first_day: h.get("firstDay").f(), day: h.get("day").f(), warn: h.get("warn").f(), night: h.get("night").f() }
    );
    phase_rows(fx.get("phase"), &STORE_HOURS, "default");
    assert_eq!(store_phase_default(12.0), store_phase(12.0, &STORE_HOURS));
    phase_rows(fx.get("phase2"), &StoreHours { first_day: 10.0, day: 20.0, warn: 3.0, night: 5.0 }, "custom");
}

#[test]
fn store_names_sets_and_tables() {
    let fx = Json::load("store.json");
    let mut m = game::base::rng::Mulberry32::new(2024);
    let mut rng = move || m.next();
    for (i, n) in fx.get("names").arr().iter().enumerate() {
        assert_eq!(product_name(&mut rng), n.s(), "product_name #{i}");
    }
    let mut m = game::base::rng::Mulberry32::new(7);
    let mut rng = move || m.next();
    for (i, s) in fx.get("sets").arr().iter().enumerate() {
        let theme = s.at(0).s();
        assert_eq!(theme, ROOM_SET_NAMES[i % 7]);
        let got = room_set(theme, &mut rng).unwrap();
        let want = s.at(1).arr();
        assert_eq!(got.len(), want.len(), "{theme} #{i}: pieces");
        for (p, j) in got.iter().zip(want.iter()) {
            assert_eq!(p.name, j.at(0).s(), "{theme}: name");
            assert_vec_close(
                &[p.x, p.z, p.yaw, p.y_off],
                &[j.at(1).f(), j.at(2).f(), j.at(3).f(), j.at(4).f()],
                TOL,
                &format!("{} {}", theme, p.name),
            );
            let opt = |v: &Json| if v.is_null() { None } else { Some(v.f()) };
            assert_eq!((p.a1, p.a2), (opt(j.at(5)), opt(j.at(6))), "{} {}: args", theme, p.name);
        }
    }
    assert!(room_set("nope", &mut rng).is_none());
    let w: Vec<(String, f64)> = fx.get("weights").arr().iter().map(|e| (e.at(0).s().to_string(), e.at(1).f())).collect();
    assert_eq!(ROOM_SET_WEIGHTS.iter().map(|(k, v)| (k.to_string(), *v)).collect::<Vec<_>>(), w);
    let s: Vec<(String, String)> = fx.get("signs").arr().iter().map(|e| (e.at(0).s().to_string(), e.at(1).s().to_string())).collect();
    assert_eq!(DEPARTMENT_SIGNS.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<Vec<_>>(), s);
    assert_eq!(department_sign("kids"), Some("CHILDREN'S ROOMS"));
}

fn geo_params(g: &Geo) -> Vec<Json> {
    let n = |v: f64| Json::Num(v);
    match g.params() {
        GeoParams::Box { width, height, depth, .. } => vec![Json::Str("box".into()), n(width), n(height), n(depth)],
        GeoParams::Cylinder { radius_top, radius_bottom, height, radial, height_seg, open_ended, theta_start, theta_length } => vec![
            Json::Str("cyl".into()),
            n(radius_top),
            n(radius_bottom),
            n(height),
            n(radial as f64),
            n(height_seg as f64),
            Json::Bool(open_ended),
            n(theta_start),
            n(theta_length),
        ],
        GeoParams::Plane { width, height, .. } => vec![Json::Str("plane".into()), n(width), n(height)],
        other => vec![Json::Str(format!("{other:?}"))],
    }
}

#[test]
fn store_kit_matches_reference() {
    let fx = Json::load("store.json");
    // the kit's rng, shared with `product_name` for the price tags (as in the recording)
    let rng = Rc::new(RefCell::new(game::base::rng::Mulberry32::new(555)));
    let draws = Rc::new(std::cell::Cell::new(0u64));
    let (r2, d2) = (rng.clone(), draws.clone());
    let pg_names: Rc<RefCell<HashMap<u64, String>>> = Rc::new(RefCell::new(HashMap::new()));
    let pn = pg_names.clone();
    let mat_hex: Rc<RefCell<HashMap<u64, u32>>> = Rc::new(RefCell::new(HashMap::new()));
    let mat_log: Rc<RefCell<Vec<u32>>> = Rc::new(RefCell::new(vec![]));
    let (mh, ml) = (mat_hex.clone(), mat_log.clone());
    let mut cache: HashMap<u32, Mat> = HashMap::new();
    let tag_ids: Rc<RefCell<HashMap<u64, i64>>> = Rc::new(RefCell::new(HashMap::new()));
    let tag_log: Rc<RefCell<Vec<TagText>>> = Rc::new(RefCell::new(vec![]));
    let (ti, tl) = (tag_ids.clone(), tag_log.clone());
    let mut kit = StoreKit::new(StoreKitDeps {
        pg: Rc::new(move |name: &str, g: Geo| {
            pn.borrow_mut().insert(g.id(), name.to_string());
            g
        }),
        mat: Box::new(move |hex: u32| {
            cache
                .entry(hex)
                .or_insert_with(|| {
                    let m = lambert!({ color: hex });
                    mh.borrow_mut().insert(m.id(), hex);
                    ml.borrow_mut().push(hex);
                    m
                })
                .clone()
        }),
        rng: Box::new(move || {
            d2.set(d2.get() + 1);
            r2.borrow_mut().next()
        }),
        tag: Box::new(move |t: &TagText| {
            tl.borrow_mut().push(t.clone());
            let m = basic_mat!();
            ti.borrow_mut().insert(m.id(), tl.borrow().len() as i64 - 1);
            m
        }),
    });
    for (row_i, row) in fx.get("kit").arr().iter().enumerate() {
        let name = row.at(0).s();
        let g = if name == "priceTag" {
            let pname = {
                let mut f = || {
                    draws.set(draws.get() + 1);
                    rng.borrow_mut().next()
                };
                product_name(&mut f)
            };
            assert_eq!(pname, row.at(1).s(), "price tag name #{row_i}");
            kit.price_tag(&TagText { name: pname, what: "Bookcase".into(), price: row.at(2).s().to_string() })
        } else {
            let opt = |v: &Json| if v.is_null() { None } else { Some(v.f()) };
            kit.build(name, opt(row.at(1)), opt(row.at(2))).unwrap()
        };
        let w = format!("{name} #{row_i}");
        let d = row.at(3);
        let hide = hide_spot(&g);
        if d.get("hide").is_null() {
            assert!(hide.is_none(), "{w}: no hide spot");
        } else {
            let h = d.get("hide").arr();
            let hs = hide.unwrap_or_else(|| panic!("{w}: hide spot missing"));
            assert_eq!((hs.eye, hs.r, hs.kind.as_str()), (h[0].f(), h[1].f(), h[2].s()), "{w}: hide");
            assert_eq!(hs.front, if h[3].is_null() { None } else { Some(h[3].f()) }, "{w}: hide front");
        }
        let kids = g.children();
        let jc = d.get("c").arr();
        assert_eq!(kids.len(), jc.len(), "{w}: parts");
        for (k, (o, j)) in kids.iter().zip(jc.iter()).enumerate() {
            let geo = o.geo();
            let pg = pg_names.borrow().get(&geo.id()).cloned();
            assert_eq!(pg.as_deref(), if j.at(0).is_null() { None } else { Some(j.at(0).s()) }, "{w} part {k}: PG name");
            let gp = geo_params(&geo);
            let jg = j.at(1).arr();
            assert_eq!(gp.len(), jg.len(), "{w} part {k}: geometry {gp:?}");
            for (a, b) in gp.iter().zip(jg.iter()) {
                match (a, b) {
                    (Json::Num(x), Json::Num(y)) => assert_close(*x, *y, TOL, &format!("{w} part {k} geometry")),
                    (a, b) => assert_eq!(a, b, "{w} part {k} geometry"),
                }
            }
            let mid = o.mat().id();
            let mv = mat_hex.borrow().get(&mid).map(|&h| h as i64).or_else(|| tag_ids.borrow().get(&mid).map(|&t| -1 - t));
            assert_eq!(mv, Some(j.at(2).i()), "{w} part {k}: material");
            let (p, r) = (o.pos(), o.rot());
            assert_vec_close(
                &[p.x, p.y, p.z, r.x, r.y, r.z],
                &[j.at(3).f(), j.at(4).f(), j.at(5).f(), j.at(6).f(), j.at(7).f(), j.at(8).f()],
                TOL,
                &format!("{w} part {k}: transform"),
            );
        }
    }
    assert_eq!(draws.get() as i64, fx.get("kitDraws").i(), "rng draws");
    let ml: Vec<i64> = fx.get("matLog").arr().iter().map(|v| v.i()).collect();
    assert_eq!(mat_log.borrow().iter().map(|&h| h as i64).collect::<Vec<_>>(), ml, "material creation order");
    let tl: Vec<(String, String, String)> =
        fx.get("tagLog").arr().iter().map(|t| (t.at(0).s().into(), t.at(1).s().into(), t.at(2).s().into())).collect();
    assert_eq!(tag_log.borrow().iter().map(|t| (t.name.clone(), t.what.clone(), t.price.clone())).collect::<Vec<_>>(), tl, "tags");
    // the shared tag plane
    assert_eq!(kit.tag_geo.params(), plane_geo!(0.2, 0.13).params());
    assert!(kit.build("priceTag", None, None).is_none());
}

#[test]
fn store_game_kit_builds_everything() {
    // the game's own wiring (PG = game::placers::kit::pg, `rnd()`)
    let mut kit = StoreKit::game(Box::new(|hex| lambert!({ color: hex })), Box::new(|_t| basic_mat!()));
    for theme in ROOM_SET_NAMES {
        for p in room_set(theme, &mut game::base::rng::rnd).unwrap() {
            let g = kit.build(p.name, p.a1, p.a2).unwrap_or_else(|| panic!("no builder for {}", p.name));
            assert!(g.child_count() > 0);
        }
    }
    let t = kit.price_tag(&TagText { name: product_name(&mut game::base::rng::rnd), what: "Desk".into(), price: "99.–".into() });
    assert_eq!(t.child_count(), 4);
}
