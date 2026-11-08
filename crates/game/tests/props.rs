//! `assets::props` against recorded reference data (fixtures in `tests/fixtures/`).
mod common;
use common::*;
use engine::prelude::*;
use game::assets::props::*;
use game::base::numeric::*;

const TOL: f64 = 1e-5;

/// The engine primitive built from these recorded constructor parameters.
fn primitive(ty: &str, p: &[f64]) -> Geo {
    match ty {
        "box" => box_geometry(p[0], p[1], p[2], p[3], p[4], p[5]),
        "cylinder" => cylinder_geometry(p[0], p[1], p[2], p[3], p[4], p[5] != 0.0, p[6], p[7]),
        "cone" => cone_geometry(p[0], p[1], p[2], p[3], p[4] != 0.0, p[5], p[6]),
        "sphere" => sphere_geometry(p[0], p[1], p[2], p[3], p[4], p[5], p[6]),
        "torus" => torus_geometry(p[0], p[1], p[2], p[3], p[4]),
        "capsule" => capsule_geometry(p[0], p[1], p[2], p[3]),
        "circle" => circle_geometry(p[0], p[1], p[2], p[3]),
        "plane" => plane_geometry(p[0], p[1], p[2], p[3]),
        "octahedron" => octahedron_geometry(p[0], p[1]),
        "ring" => ring_geometry(p[0], p[1], p[2], p[3], p[4], p[5]),
        _ => panic!("unknown primitive {}", ty),
    }
}

fn check_geo(what: &str, c: &Case, g: &Geo) {
    let gg = g.r();
    c.rec("position").check(what, &f32s(&gg.position), TOL);
    c.rec("normal").check(what, &f32s(&gg.normal), TOL);
    c.rec("uv").check(what, &f32s(&gg.uv), TOL);
    c.rec("index").check(what, &u32s(gg.index.as_ref().unwrap()), 0.0);
    drop(gg);
    let b = g.compute_bounding_box();
    c.rec("bbox").check(what, &[b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z], TOL);
    let s = g.compute_bounding_sphere();
    c.rec("sphere").check(what, &[s.center.x, s.center.y, s.center.z, s.radius], TOL);
}

#[test]
fn props_match_reference() {
    let buf = fixture("props_fixtures.bin");
    let mut r = Rd::new(&buf);
    let cases = read_cases(&mut r, b"PFIX0001");
    assert!(cases.len() >= 30);
    set_props_enabled(true);
    clear_prop_cache();
    reset_prop_stats();
    let mut results: Vec<Geo> = vec![];
    for (ci, c) in cases.iter().enumerate() {
        let what = format!("case {} {} {:?}", ci, c.name, c.spec);
        let ty = c.spec["type"].clone();
        let fb = primitive(&ty, &c.spec_f("params"));
        let t = c.spec_f("translate");
        if t.iter().any(|&x| x != 0.0) {
            fb.translate(t[0], t[1], t[2]);
        }
        let fb = if c.spec["nested"].is_empty() { fb } else { prop_part(&c.spec["nested"], fb, false) };
        let disabled = c.spec_i("disabled") == 1;
        if disabled {
            set_props_enabled(false);
        }
        let before = prop_stats();
        let g = prop_part(&c.name, fb.clone(), c.spec_i("own") == 1);
        if disabled {
            set_props_enabled(true);
        }
        let cached = prop_stats().cached > before.cached;
        let same = results.iter().position(|x| x.ptr_eq(&g)).map(|i| i as i64).unwrap_or(-1);
        results.push(g.clone());
        assert_eq!(g.ptr_eq(&fb), c.res_i("fallback") == 1, "{}: fallback", what);
        assert_eq!(cached, c.res_i("cached") == 1, "{}: cached", what);
        assert_eq!(same, c.res_i("same"), "{}: shared geometry", what);
        assert_eq!(g.type_name(), c.result["gtype"], "{}: type", what);
        assert_eq!(g.r().is_prop_part, c.res_i("part") == 1, "{}: prop_part flag", what);
        if c.res_i("keepsParams") == 1 && !g.ptr_eq(&fb) {
            assert_eq!(g.params(), fb.params(), "{}: parameters", what);
        }
        check_geo(&what, c, &g);
    }
    // the counters the recording ended with
    let rec = kv(&r.str());
    let st = prop_stats();
    assert_eq!(st.swapped.to_string(), rec["swapped"]);
    assert_eq!(st.missing.to_string(), rec["missing"]);
    assert_eq!(st.cached.to_string(), rec["cached"]);
    assert_eq!(st.tris.to_string(), rec["tris"]);

    // every recorded part, fitted to its own frame (after the cases, cache
    // kept), as they were recorded; parts made since (batch v01 on) are
    // held by `asset_lint` and its parts table
    let n = r.u32() as usize;
    assert!(n <= prop_part_names().len(), "PROP_PART_NAMES length");
    for i in 0..n {
        let name = r.str();
        let frame = r.str();
        let (pn, pt, nv, ni) = (r.u32() as usize, r.u32() as usize, r.u32() as usize, r.u32() as usize);
        let sums = [r.f64(), r.f64(), r.f64(), r.f64()];
        let _ = i;
        let m = prop_part_info(&name).unwrap_or_else(|| panic!("{name}: a recorded part is gone"));
        assert_eq!((m.frame.as_str(), m.n, m.t), (frame.as_str(), pn, pt), "{}: metadata", name);
        let fb = Geo::new();
        fb.set_position([m.fmin, m.fmax].iter().flat_map(|a| a.iter().map(|&x| x as f32)).collect());
        let g = prop_part(&name, fb, false);
        let gg = g.r();
        assert_eq!(gg.vertex_count(), nv, "{}: vertices after the seam split", name);
        assert_eq!(gg.index.as_ref().unwrap().len(), ni, "{}: indices", name);
        let sum = |a: &[f32]| a.iter().fold(0.0f64, |t, &x| t + x as f64);
        let isum = gg.index.as_ref().unwrap().iter().fold(0.0f64, |t, &x| t + x as f64);
        let tol = 1e-5 * nv as f64 + 1e-6;
        assert!((sum(&gg.position) - sums[0]).abs() <= tol, "{}: position sum {} vs {}", name, sum(&gg.position), sums[0]);
        assert!((sum(&gg.normal) - sums[1]).abs() <= tol, "{}: normal sum {} vs {}", name, sum(&gg.normal), sums[1]);
        assert!((sum(&gg.uv) - sums[2]).abs() <= tol, "{}: uv sum {} vs {}", name, sum(&gg.uv), sums[2]);
        assert_eq!(isum, sums[3], "{}: index sum", name);
    }
    assert!(r.done());
}

#[test]
fn fmt_fixed_matches_reference() {
    for line in fixture_text("fmt_fixed.txt").lines() {
        let (v, want) = line.split_once(' ').unwrap();
        let x: f64 = if v == "-0" { -0.0 } else { v.parse().unwrap() };
        assert_eq!(fmt_fixed(x, 4), want, "fmt_fixed({}, 4)", v);
    }
}

#[test]
fn decodes_every_part() {
    let names = prop_part_names();
    // as many as the parts table lists (`asset_lint` keeps it current)
    let table = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/parts.tsv")).unwrap();
    assert_eq!(names.len(), table.lines().filter(|l| l.starts_with("prop\t")).count(), "prop parts against parts.tsv");
    let mut tris = 0;
    for &name in names {
        let m = prop_part_info(name).unwrap();
        let d = decode(m);
        assert_eq!(d.pos.len(), m.n * 3, "{}", name);
        assert_eq!(d.nrm.len(), m.n * 3, "{}", name);
        assert_eq!(d.idx.len(), m.t * 3, "{}", name);
        assert!(d.idx.iter().all(|&i| (i as usize) < m.n), "{}: index out of range", name);
        assert!(d.pos.iter().all(|x| x.is_finite()), "{}", name);
        // decoded positions stay inside the quantization box
        for i in 0..m.n {
            for k in 0..3 {
                let v = d.pos[i * 3 + k] as f64;
                assert!(v >= m.min[k] - 1e-5 && v <= m.max[k] + 1e-5, "{}: position outside its bounds", name);
            }
        }
        tris += m.t;
    }
    assert!(tris > 600_000);
}

#[test]
fn cache_own_and_switches() {
    clear_prop_cache();
    let name = prop_part_names()[0];
    let a = prop_part(name, box_geo!(0.3, 0.2, 0.1), false);
    let b = prop_part(name, box_geo!(0.3, 0.2, 0.1, 3, 3, 3), false);
    assert!(a.ptr_eq(&b), "same bounds share one geometry");
    let c = prop_part(name, box_geo!(0.3, 0.2, 0.1), true);
    assert!(!c.ptr_eq(&a), "own returns a private copy");
    assert_eq!(c.r().position, a.r().position);
    assert!(c.r().is_prop_part);
    assert_eq!(c.type_name(), "buffer");
    assert_eq!(a.params(), box_geo!(0.3, 0.2, 0.1).params(), "keeps the fallback's parameters");
    assert_eq!(prop_cache_len(), 1);
    clear_prop_cache();
    assert_eq!(prop_cache_len(), 0);
    let d = prop_part(name, box_geo!(0.3, 0.2, 0.1), false);
    assert!(!d.ptr_eq(&a), "cleared cache builds anew");
    set_props_enabled(false);
    let fb = box_geo!(1, 1, 1);
    assert!(prop_part(name, fb.clone(), false).ptr_eq(&fb));
    set_props_enabled(true);
    let fb = box_geo!(1, 1, 1);
    let before = prop_stats().missing;
    assert!(prop_part("No__suchPart", fb.clone(), false).ptr_eq(&fb));
    assert_eq!(prop_stats().missing, before + 1);
    assert!(has_prop_part(name) && !has_prop_part("No__suchPart"));
    // the placer API forwards here
    let e = game::pg(name, box_geo!(0.3, 0.2, 0.1));
    assert!(e.ptr_eq(&d));
}
