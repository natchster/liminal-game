//! The creatures' bodies (`assets/bodies.bin`, `entities::body`): every body
//! decodes into sound meshes, and the posing helpers do what they say.
use game::assets::bodies::{body, body_names, decode_mesh};
use game::entities::body::{build_body, two_bone};
use game::prelude::*;

fn close(a: Vec3, b: Vec3, tol: f64) -> bool {
    (a.x - b.x).abs() < tol && (a.y - b.y).abs() < tol && (a.z - b.z).abs() < tol
}

#[test]
fn every_body_decodes_soundly() {
    let names = body_names();
    assert!(names.contains(&"mannequin"), "the test body is in the file: {names:?}");
    for name in names {
        let b = body(name).unwrap();
        for (i, bone) in b.bones.iter().enumerate() {
            assert!(bone.parent.is_none_or(|p| p < i), "{name}: bone {} before its parent", bone.name);
        }
        for m in &b.meshes {
            let g = decode_mesh(b, m);
            let n = g.position.len() / 3;
            assert_eq!(n, m.n, "{name}/{}: vertex count", m.name);
            let idx = g.index.as_ref().unwrap();
            assert_eq!(idx.len(), m.t * 3);
            assert!(idx.iter().all(|&i| (i as usize) < n), "{name}/{}: an index past the end", m.name);
            // normals are unit
            for v in g.normal.as_chunks::<3>().0 {
                let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
                assert!((0.9..1.1).contains(&l), "{name}/{}: a normal of length {l}", m.name);
            }
            if m.bone.is_none() {
                assert_eq!(g.skin_weight.len(), n * 4, "{name}/{}: four weights a vertex", m.name);
                for w in g.skin_weight.as_chunks::<4>().0 {
                    let s: f32 = w.iter().sum();
                    assert!((s - 1.0).abs() < 0.01, "{name}/{}: weights sum to {s}", m.name);
                }
                assert!(g.skin_index.iter().all(|&j| (j as usize) < b.bones.len()));
            }
            // a body stands on the floor and fits under a doorway's lintel or a
            // low ceiling, in every pose its morphs give it (each pose's own
            // extent: the bounding box of relative morphs is wider, for culling)
            if m.bone.is_none() {
                let mut poses = vec![g.position.clone()];
                for k in &g.morph_position {
                    poses.push(g.position.iter().zip(k).map(|(a, b)| a + b).collect());
                }
                for (pi, pos) in poses.iter().enumerate() {
                    let ys = pos.iter().skip(1).step_by(3);
                    let (lo, hi) = ys.fold((f32::MAX, f32::MIN), |(lo, hi), &y| (lo.min(y), hi.max(y)));
                    assert!(lo > -0.05 && hi < 2.6, "{name}/{} (pose {pi}): spans y {lo}..{hi}", m.name);
                }
            }
        }
    }
}

#[test]
fn aim_points_a_bone_whatever_its_parents_do() {
    let rig = build_body("mannequin", &|_| phong!({ color: 0xffffff })).unwrap();
    let (spine, fore, hand) = (rig.ix("spine").unwrap(), rig.ix("forearm.L").unwrap(), rig.ix("hand.L").unwrap());
    // at rest a tail is where the file says
    let t = rig.tail(fore);
    let d = &rig.def.bones[fore];
    assert!(close(t, v3(d.tail[0], d.tail[1], d.tail[2]), 1e-6), "rest tail {t:?}");
    // bend the spine, then aim the forearm straight ahead: it points ahead
    rig.turn(spine, &Quat::from_axis_angle(&v3(1.0, 0.0, 0.0), 0.6));
    rig.aim(fore, v3(0.0, 0.0, 1.0), 0.0);
    let (h, t) = (rig.head(fore), rig.tail(fore));
    let mut dir = v3(t.x - h.x, t.y - h.y, t.z - h.z);
    dir.normalize();
    assert!(close(dir, v3(0.0, 0.0, 1.0), 1e-6), "forearm points {dir:?}");
    // the hand follows its forearm: its head is the forearm's tail
    assert!(close(rig.head(hand), t, 1e-6));
}

#[test]
fn two_bone_reaches_and_bends_toward_its_pole() {
    let (root, target, pole) = (v3(0.0, 1.0, 0.0), v3(0.0, 0.3, 0.3), v3(0.0, 0.6, 1.0));
    let (u, l) = two_bone(root, target, pole, 0.45, 0.45);
    assert!((u.length() - 0.45).abs() < 1e-9 && (l.length() - 0.45).abs() < 1e-9);
    let end = v3(root.x + u.x + l.x, root.y + u.y + l.y, root.z + u.z + l.z);
    assert!(close(end, target, 1e-9), "reaches {end:?}");
    assert!(u.z > 0.0, "the knee bends toward the pole");
    // out of reach: straight toward the target
    let (u, l) = two_bone(root, v3(0.0, -5.0, 0.0), pole, 0.45, 0.45);
    assert!(u.y < -0.44 && l.y < -0.44);
}
