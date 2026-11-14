//! `assets::creatures` and `wt_geo` against recorded reference data (fixtures
//! in `tests/fixtures/`).
mod common;
use common::*;
use game::assets::creatures::*;

const TOL: f64 = 1e-5;

#[test]
fn creatures_match_reference() {
    let buf = fixture("creature_fixtures.bin");
    let mut r = Rd::new(&buf);
    let cases = read_cases(&mut r, b"CFIX0001");
    for c in &cases {
        let what = format!("{} {:?}", c.name, c.spec);
        match c.spec["kind"].as_str() {
            "geometry" => {
                let g = creature_geometry(&c.name).expect("part");
                let gg = g.r();
                assert_eq!(gg.vertex_count().to_string(), c.result["n"], "{}", what);
                assert_eq!((gg.index.as_ref().unwrap().len() / 3).to_string(), c.result["t"], "{}", what);
                assert_eq!(gg.bone_names.join(","), c.result["bones"], "{}: bones", what);
                c.rec("position").check(&what, &f32s(&gg.position), TOL);
                c.rec("normal").check(&what, &f32s(&gg.normal), TOL);
                c.rec("uv").check(&what, &f32s(&gg.uv), TOL);
                c.rec("color").check(&what, &f32s(&gg.color), TOL);
                c.rec("skinIndex").check(&what, &f32s(&gg.skin_index), 0.0);
                c.rec("skinWeight").check(&what, &f32s(&gg.skin_weight), TOL);
                c.rec("index").check(&what, &u32s(gg.index.as_ref().unwrap()), 0.0);
                let s = gg.bounding_sphere.unwrap();
                c.rec("sphere").check(&what, &[s.center.x, s.center.y, s.center.z, s.radius], TOL);
            }
            "cavity" => {
                let strength: f64 = c.spec["strength"].parse().unwrap();
                let base = creature_geometry(&c.name).unwrap();
                let g = with_cavity_shade(&base, strength, None);
                assert!(!g.ptr_eq(&base));
                assert_eq!(g.r().color_size, 3);
                c.rec("color").check(&what, &f32s(&g.r().color), TOL);
            }
            "watcher" => {
                let w = watcher_geometry();
                assert_eq!(w.morphs.to_string(), c.result["morphs"]);
                assert!(w.body.ptr_eq(&creature_geometry("wtBody").unwrap()), "the morphs go on the shared wtBody");
                let b = w.body.r();
                assert!(b.morph_relative);
                let s = b.bounding_sphere.unwrap();
                c.rec("sphere").check(&what, &[s.center.x, s.center.y, s.center.z, s.radius], TOL);
                for i in 0..w.morphs {
                    c.rec(&format!("mp{}", i)).check(&what, &f32s(&b.morph_position[i]), TOL);
                    c.rec(&format!("mn{}", i)).check(&what, &f32s(&b.morph_normal[i]), TOL);
                    c.rec(&format!("mc{}", i)).check(&what, &f32s(&w.morph_color[i]), TOL);
                }
            }
            k => panic!("unknown case kind {}", k),
        }
    }
    // every part: counts and sums
    let n = r.u32() as usize;
    assert_eq!(n, creature_part_names().len());
    for i in 0..n {
        let name = r.str();
        assert_eq!(creature_part_names()[i], name, "CREATURE_PART_NAMES order");
        let (vn, tn) = (r.u32() as usize, r.u32() as usize);
        let sums: Vec<f64> = (0..7).map(|_| r.f64()).collect();
        let m = creature_part_info(&name).unwrap();
        assert_eq!((m.n, m.t), (vn, tn), "{}: metadata", name);
        let g = creature_geometry(&name).unwrap();
        let gg = g.r();
        assert_eq!(gg.vertex_count(), vn, "{}", name);
        assert_eq!(gg.index.as_ref().unwrap().len(), tn * 3, "{}", name);
        let sum = |a: &[f32]| a.iter().fold(0.0f64, |t, &x| t + x as f64);
        let tol = 1e-5 * vn as f64 + 1e-6;
        for (k, (label, a)) in [
            ("position", &gg.position),
            ("normal", &gg.normal),
            ("uv", &gg.uv),
            ("color", &gg.color),
            ("skinIndex", &gg.skin_index),
            ("skinWeight", &gg.skin_weight),
        ]
        .iter()
        .enumerate()
        {
            assert!((sum(a) - sums[k]).abs() <= tol, "{}: {} sum {} vs recorded {}", name, label, sum(a), sums[k]);
        }
        let isum = gg.index.as_ref().unwrap().iter().fold(0.0f64, |t, &x| t + x as f64);
        assert_eq!(isum, sums[6], "{}: index sum", name);
    }
    assert!(r.done());
}

#[test]
fn decodes_every_part() {
    let names = creature_part_names();
    assert_eq!(names.len(), 44);
    for &name in names {
        let m = creature_part_info(name).unwrap();
        let g = Geo_of(name);
        let gg = g.r();
        assert_eq!(gg.vertex_count(), m.n, "{}", name);
        assert_eq!(gg.index.as_ref().unwrap().len(), m.t * 3, "{}", name);
        assert!(gg.index.as_ref().unwrap().iter().all(|&i| (i as usize) < m.n), "{}: index out of range", name);
        assert_eq!(gg.normal.len(), m.n * 3);
        assert_eq!(gg.uv.len(), m.n * 2);
        assert_eq!(gg.color.is_empty(), !m.has_color, "{}", name);
        if m.skinned {
            assert_eq!(gg.skin_index.len(), m.n * 4);
            assert!(!gg.bone_names.is_empty());
            assert!(gg.skin_index.iter().all(|&j| (j as usize) < gg.bone_names.len()), "{}: bone slot out of range", name);
            for w in gg.skin_weight.as_chunks::<4>().0 {
                assert!((w.iter().sum::<f32>() - 1.0).abs() < 0.02, "{}: weights sum to 255", name);
            }
        }
        assert!(gg.position.iter().all(|x| x.is_finite()));
    }
    assert!(creature_geometry("noSuchPart").is_none());
}

#[allow(non_snake_case)]
fn Geo_of(name: &str) -> engine::geometry::Geo {
    creature_geometry(name).unwrap()
}

#[test]
fn cavity_shade_center_and_cache() {
    let a = creature_geometry("smThigh").unwrap();
    let b = creature_geometry("smThigh").unwrap();
    assert!(a.ptr_eq(&b), "cached and shared");
    let c = with_cavity_shade(&a, 0.75, Some(engine::math::Vec3::new(0.0, -0.3, 0.0)));
    assert_eq!(c.r().color.len(), a.r().position.len());
    assert!(c.r().color.iter().all(|&x| (0.0..=1.0).contains(&x)));
    assert_eq!(c.r().position, a.r().position);
}
