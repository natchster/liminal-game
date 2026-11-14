//! `world::placement` / `world::instancing` against recorded reference data on
//! synthetic worlds (fixtures in `tests/fixtures/`), plus smoke tests.
mod common;
use common::*;
use engine::prelude::*;
use game::world::instancing::*;
use game::world::placement::*;

fn nums(it: &[&str]) -> Vec<f64> {
    it.iter().map(|x| x.parse::<f64>().unwrap()).collect()
}

#[test]
fn placement_matches_reference() {
    let text = fixture_text("placement_world.txt");
    let world = group();
    let mut walls: Vec<[f64; 6]> = vec![];
    let mut objects: Vec<PlaceObject> = vec![];
    let mut want: Vec<(bool, [f64; 3])> = vec![];
    let mut want_stats = vec![];
    let mut cur: Option<Obj> = None;
    for line in text.lines() {
        let t: Vec<&str> = line.split(' ').collect();
        match t[0] {
            "W" => {
                let v = nums(&t[1..7]);
                walls.push([v[0], v[1], v[2], v[3], v[4], v[5]]);
            }
            "O" => {
                let v = nums(&t[2..9]);
                let o = group();
                o.set_pos(v[2], v[3], v[4]);
                o.rotation().y = v[5];
                o.user().placer = Some(t[1].to_string());
                o.user().floats = v[1] == 1.0;
                world.add(&o);
                objects.push(PlaceObject { obj: o.clone(), protect: v[0] == 1.0, placer: t[1].to_string() });
                cur = Some(o);
            }
            "M" => {
                let v = nums(&t[2..14]);
                let (a, b, c) = (v[0], v[1], v[2]);
                let geo = match t[1] {
                    "box" => box_geo!(a, b, c),
                    "cyl" => cyl_geo!(a / 2.0, a / 2.0, b, 12),
                    _ => plane_geo!(a, c),
                };
                let mat = lambert!({ transparent: v[8] == 1.0, opacity: v[9], depthWrite: v[10] == 1.0, blending: if v[11] == 1.0 { AdditiveBlending } else { NormalBlending } });
                let m = mesh(geo, &mat);
                m.set_pos(v[3], v[4], v[5]);
                m.set_rot(v[6], 0.0, v[7]);
                cur.as_ref().unwrap().add(&m);
            }
            "R" => {
                let v = nums(&t[2..6]);
                want.push((v[0] == 1.0, [v[1], v[2], v[3]]));
            }
            "S" => want_stats = nums(&t[1..]),
            _ => panic!("bad line {line}"),
        }
    }
    let wq = |bb: &[f64; 6]| -> Vec<[f64; 6]> {
        walls.iter().filter(|w| !(w[0] > bb[3] || w[3] < bb[0] || w[2] > bb[5] || w[5] < bb[2])).copied().collect()
    };
    let ceil_at = |x: f64, _z: f64| if x > 3.0 { 2.4 } else { 3.0 };
    let in_pool = |x: f64, z: f64| x < -4.5 && z < -4.5;
    let objs: Vec<Obj> = objects.iter().map(|o| o.obj.clone()).collect();
    let st = resolve_placement(&PlacementOpts { objects, walls: &wq, ceil_at: &ceil_at, in_pool: Some(&in_pool), tol: 0.015 });
    let got = [st.props, st.lifted, st.dropped, st.stacked, st.unburied, st.nudged, st.removed, st.resettled, st.skipped].map(|x| x as f64);
    assert_eq!(got.to_vec(), want_stats, "stats (props lifted dropped stacked unburied nudged removed resettled skipped)");
    for (i, (o, (removed, p))) in objs.iter().zip(&want).enumerate() {
        assert_eq!(o.parent().is_none(), *removed, "object {i} removed");
        let q = o.pos();
        for (k, (a, b)) in [q.x, q.y, q.z].iter().zip(p).enumerate() {
            assert!((a - b).abs() <= 1e-9, "object {i} axis {k}: got {a} vs recorded {b}");
        }
    }
}

#[test]
fn instancing_matches_reference() {
    let text = fixture_text("instancing_world.txt");
    let geos = [box_geo!(0.1, 0.2, 0.3), cyl_geo!(0.05, 0.05, 0.4, 8), sphere_geo!(0.1, 8, 6)];
    let mut mats: Vec<Mat> = vec![];
    let world = group();
    let mut top: Option<Obj> = None;
    let mut parent: Option<Obj> = None;
    let mut want_stats = vec![];
    let mut want_children: Vec<Vec<f64>> = vec![];
    let mut kinds: Vec<&str> = vec![];
    for line in text.lines() {
        let t: Vec<&str> = line.split(' ').collect();
        match t[0] {
            "GEOS" => {}
            "MAT" => {
                let v = nums(&t[3..7]);
                let m = if t[2] == "basic" { basic_mat!({ color: v[0] }) } else { lambert!({ color: v[0], emissive: v[1] }) };
                m.m().transparent = v[2] == 1.0;
                m.m().depth_write = v[3] == 1.0;
                mats.push(m);
            }
            "T" => {
                let v = nums(&t[1..6]);
                let g = group();
                g.set_pos(v[2], 0.0, v[3]);
                g.rotation().y = v[4];
                g.user().static_prop = v[0] == 1.0;
                g.user().no_instance = v[1] == 1.0;
                world.add(&g);
                top = Some(g.clone());
                parent = Some(g);
            }
            "SUB" => {
                let s = group();
                s.set_pos(0.0, 0.5, 0.0);
                top.as_ref().unwrap().add(&s);
                parent = Some(s);
            }
            "M" => {
                let v = nums(&t[1..7]);
                let m = mesh(&geos[v[0] as usize], &mats[v[1] as usize]);
                m.set_pos(v[2], v[3], v[4]);
                m.set_visible(v[5] == 1.0);
                parent.as_ref().unwrap().add(&m);
            }
            "S" => want_stats = nums(&t[1..]),
            "I" | "G" => {
                kinds.push(if t[0] == "I" { "I" } else { "G" });
                want_children.push(nums(&t[1..]));
            }
            _ => panic!("bad line {line}"),
        }
    }
    let chunk_of = |x: f64, z: f64| (x / 4.0).floor() as i64 * 64 + (z / 4.0).floor() as i64;
    let st = instancify_static(&world, &chunk_of);
    assert_eq!(vec![st.meshes as f64, st.instanced as f64, st.buckets as f64, st.removed_groups as f64], want_stats, "stats");
    let kids = world.children();
    assert_eq!(kids.len(), want_children.len(), "world children");
    for (i, c) in kids.iter().enumerate() {
        if kinds[i] == "I" {
            assert!(c.is_instanced(), "child {i} instanced");
            let w = &want_children[i];
            assert_eq!(c.instance_count() as f64, w[0]);
            let gi = geos.iter().position(|g| g.ptr_eq(&c.geo())).unwrap() as f64;
            let mi = mats.iter().position(|m| m.ptr_eq(&c.mat())).unwrap() as f64;
            assert_eq!((gi, mi), (w[1], w[2]), "child {i}: geometry / material");
            let (mut sx, mut sy, mut sz) = (0.0, 0.0, 0.0);
            for k in 0..c.instance_count() {
                let m = c.get_matrix_at(k);
                sx += m.e[12];
                sy += m.e[13];
                sz += m.e[14];
            }
            // instance matrices are stored as f32
            for (a, b) in [sx, sy, sz].iter().zip(&w[3..6]) {
                assert!((a - b).abs() < 1e-5, "child {i}: instance translations {a} vs {b}");
            }
            assert!(c.r().user.static_prop && c.r().user.instanced && !c.r().matrix_auto_update);
        } else {
            assert!(!c.is_instanced(), "child {i} is a group");
            let (mut n, mut g) = (0.0, 0.0);
            c.traverse(&mut |o: &Obj| if o.is_mesh() { n += 1.0 } else { g += 1.0 });
            assert_eq!(vec![n, g], want_children[i], "child {i}: meshes / groups left");
        }
    }
}

// ------------------------------------------------------------------ smoke tests
fn crate_mesh(w: f64, h: f64, d: f64, mat: &Mat) -> Obj {
    let m = mesh(box_geo!(w, h, d), mat);
    m.set_pos(0.0, h / 2.0, 0.0);
    m
}

#[test]
fn placement_smoke() {
    let world = group();
    let mat = lambert!({ color: 0x808080 });
    let add = |x: f64, y: f64, z: f64, w: f64, h: f64, d: f64, placer: &str| -> Obj {
        let o = group();
        o.set_pos(x, y, z);
        o.add(&crate_mesh(w, h, d, &mat));
        o.user().placer = Some(placer.into());
        world.add(&o);
        o
    };
    let sunk = add(0.0, -0.05, 0.0, 0.5, 0.5, 0.5, "place_a"); // sunk into the floor → lifted
    let hover = add(3.0, 0.3, 3.0, 0.3, 0.3, 0.3, "place_b"); // hovering → dropped
    let table = add(-3.0, 0.0, -3.0, 1.0, 0.7, 1.0, "place_c");
    let cup = add(-3.0, 0.75, -3.0, 0.1, 0.1, 0.1, "place_d"); // above the table → set down on it
    let buried = add(4.9, 0.0, 0.0, 0.4, 0.4, 0.4, "place_e"); // in the wall at x = 5 → pushed out
    let a = add(0.0, 0.0, 3.0, 0.6, 0.6, 0.6, "place_f");
    let b = add(0.2, 0.0, 3.0, 0.3, 0.3, 0.3, "place_g"); // inside `a` → nudged aside
    let floater = add(-1.0, 1.0, -1.0, 0.2, 0.2, 0.2, "place_h");
    floater.user().floats = true;
    let walls = [[5.0, 0.0, -5.0, 5.2, 3.0, 5.0]];
    let wq = |bb: &[f64; 6]| -> Vec<[f64; 6]> {
        walls.iter().filter(|w| !(w[0] > bb[3] || w[3] < bb[0] || w[2] > bb[5] || w[5] < bb[2])).copied().collect()
    };
    let objects: Vec<PlaceObject> = world
        .children()
        .iter()
        .map(|o| PlaceObject { obj: o.clone(), protect: false, placer: o.r().user.placer.clone().unwrap() })
        .collect();
    let st = resolve_placement(&PlacementOpts { objects, walls: &wq, ceil_at: &|_, _| 3.0, in_pool: None, tol: 0.015 });
    assert_eq!(st.props, 7);
    assert_eq!(st.skipped, 1);
    assert!((sunk.pos().y - 0.0).abs() < 1e-6, "lifted to the floor: {}", sunk.pos().y);
    assert!((hover.pos().y - 0.0).abs() < 1e-6, "dropped: {}", hover.pos().y);
    assert!((cup.pos().y - 0.7).abs() < 1e-6, "on the table: {}", cup.pos().y);
    assert!((table.pos().y).abs() < 1e-6);
    assert!(buried.pos().x + 0.2 <= 5.0 + 0.015 + 1e-9, "out of the wall: {}", buried.pos().x);
    assert!(b.parent().is_some(), "the smaller one is kept");
    assert!((b.pos().x - 0.2).abs() > 0.1 || (b.pos().z - 3.0).abs() > 0.1, "nudged: {:?}", b.pos());
    assert!((a.pos().x).abs() < 1e-9 && (a.pos().z - 3.0).abs() < 1e-9, "the bigger one stays");
    assert_eq!(floater.pos().y, 1.0);
    assert!(st.lifted >= 1 && st.dropped >= 1 && st.resettled >= 1 && st.unburied >= 1 && st.nudged >= 1, "{st:?}");
    // the wall grid gives the same boxes a brute-force filter does
    let mut wg = WallGrid::new(21, 21, 2.0);
    wg.add_tile_box(10.0, 3.0, 10.1, 17.0, 0.0, 3.0);
    wg.add_tile_box(3.0, 10.0, 17.0, 10.2, 0.0, 2.5);
    let hits = wg.query(&[-0.5, 0.0, -0.5, 0.5, 1.0, 0.5]);
    assert_eq!(hits.len(), 2);
    assert!(wg.query(&[-15.0, 0.0, -15.0, -14.0, 1.0, -14.0]).is_empty());
}

#[test]
fn instancing_smoke() {
    let world = group();
    let geo = box_geo!(0.1, 0.1, 0.1);
    let m1 = lambert!({ color: 0x336699 });
    let m2 = lambert!({ color: 0x336699 }); // a different material that looks the same
    let glass = lambert!({ color: 0x336699, transparent: true, opacity: 0.5 });
    for i in 0..6 {
        let g = group();
        g.set_pos(i as f64 * 0.5, 0.0, 0.0);
        g.user().static_prop = i != 5;
        let leg = mesh(&geo, if i % 2 == 0 { &m1 } else { &m2 });
        g.add(&leg);
        let pane = mesh(&geo, &glass);
        g.add(&pane);
        world.add(&g);
    }
    let st = instancify_static(&world, &|x, z| (x / 20.0).floor() as i64 * 64 + (z / 20.0).floor() as i64);
    assert_eq!(st.meshes, 5);
    assert_eq!(st.buckets, 1);
    assert_eq!(st.instanced, 5);
    assert_eq!(st.removed_groups, 0, "the glass panes keep their groups");
    let inst: Vec<Obj> = world.children().into_iter().filter(|c| c.is_instanced()).collect();
    assert_eq!(inst.len(), 1);
    assert_eq!(inst[0].instance_count(), 5);
    assert!((inst[0].get_matrix_at(4).e[12] - 2.0).abs() < 1e-12);
    assert_eq!(look_key(&m1), look_key(&m2));
    assert_ne!(look_key(&m1), look_key(&lambert!({ color: 0x336698 })));
    // with the panes gone too, emptied groups are removed
    let world = group();
    for i in 0..3 {
        let g = group();
        g.user().static_prop = true;
        let s = group();
        s.add(&mesh(&geo, &m1));
        g.add(&s);
        g.set_pos(i, 0, 0);
        world.add(&g);
    }
    let st = instancify_static(&world, &|_, _| 0);
    assert_eq!((st.instanced, st.buckets, st.removed_groups), (3, 1, 6));
    assert_eq!(world.child_count(), 1);
    let im = world.children()[0].clone();
    assert_eq!(im.use_wear(), UseWear::default(), "worn alike, the instances share the mesh's own");
    // things worn differently still share one draw, each instance its own
    // wear by use (a thing hung up, one on a storey's floor)
    let world = group();
    let hung = UseWear { handled: 0.45, chips: 0.0, floor: 0.0 };
    let upstairs = UseWear { handled: 1.0, chips: 1.0, floor: 7.2 };
    for (i, w) in [UseWear::default(), hung, upstairs].into_iter().enumerate() {
        let g = group();
        g.user().static_prop = true;
        let leg = mesh(&geo, &m1);
        leg.set_use_wear(w);
        g.add(&leg);
        g.set_pos(i, 0, 0);
        world.add(&g);
    }
    let st = instancify_static(&world, &|_, _| 0);
    assert_eq!((st.instanced, st.buckets), (3, 1));
    let im = world.children().into_iter().find(|c| c.is_instanced()).unwrap();
    assert_eq!((im.use_wear_at(0), im.use_wear_at(1), im.use_wear_at(2)), (UseWear::default(), hung, upstairs));
}
