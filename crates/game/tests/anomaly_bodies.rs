//! The anomalies' figures on the new bodies (`docs/design/ENTITIES.md`, E9). The
//! Showroom's staff and the Borrowed House's shadows walk on the procedural
//! human rig (`anomalies::rig`), not on a body's own skeleton: the staff's
//! body, built in Blender (`assets/src/creatures/body_staff.py`), is skinned
//! to the rig's bones as it stands at rest (the shadows keep their old
//! body, which blind reviews preferred). The scripts read
//! that rest pose from `assets/src/creatures/rigs.json`, written here, and
//! the bodies are held to it, so a rig that changes cannot leave its body
//! bound to the wrong joints.
//!
//! The bodies are held to their rigs (each bone where the rig's stands),
//! bound whole (every part to the rig's bones), kept in their budgets, and
//! walked: skinned on the CPU through the gait's longest stride, no edge of
//! them may stretch out of all measure (weights gone wrong tear a body).
//! The seated man of the Loop sits on his chair with his feet on the floor.
//!
//!   REGEN=1 cargo test --release -p game --test anomaly_bodies   (after a rig changes)
use game::anomalies::rig::{GaitStyle, HUMAN_RIG_SHADOW, HUMAN_RIG_STAFF, HumanRig, bind_figure, human_gait_entity, human_rig};
use game::base::records::Entity;
use game::prelude::*;
use serde_json::{Value, json};

const RIGS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/src/creatures/rigs.json");

fn world(o: &Obj, local: [f64; 3]) -> [f64; 3] {
    let mut p = Vec3::new(local[0], local[1], local[2]);
    p.apply_matrix4(&o.matrix_world());
    [p.x, p.y, p.z]
}

fn r4(p: [f64; 3]) -> Value {
    json!(p.iter().map(|v| (v * 1e4).round() / 1e4).collect::<Vec<_>>())
}

/// The rig's bones at rest (game frame, the feet on the floor): each with
/// its parent, its head, and its tail (its main child's head, or the tail
/// it keeps, or its only child's head).
fn rest(r: &HumanRig) -> Value {
    let g = group();
    human_rig(&g, r, None);
    g.update_matrix_world(true);
    let mut bones = vec![];
    fn visit(o: &Obj, parent: Option<String>, out: &mut Vec<Value>) {
        for c in o.children() {
            if !c.is_bone() {
                continue;
            }
            let kids: Vec<Obj> = c.children().into_iter().filter(|k| k.is_bone()).collect();
            // (read the user data out first: it borrows the node)
            let main = c.user().s.get("main").cloned();
            let kept = {
                let u = c.user();
                u.has("tail_x").then(|| [u.get("tail_x"), u.get("tail_y"), u.get("tail_z")])
            };
            let tail = match main.and_then(|m| kids.iter().find(|k| k.name() == m).cloned()) {
                Some(m) => world(&m, [0.0; 3]),
                _ => {
                    if let Some(t) = kept {
                        world(&c, t)
                    } else if kids.len() == 1 {
                        world(&kids[0], [0.0; 3])
                    } else {
                        panic!("{}: no tail", c.name())
                    }
                }
            };
            out.push(json!({ "name": c.name(), "parent": parent, "head": r4(world(&c, [0.0; 3])), "tail": r4(tail) }));
            visit(&c, Some(c.name()), out);
        }
    }
    visit(&g, None, &mut bones);
    json!({ "bones": bones })
}

fn rigs() -> Value {
    json!({ "staff": rest(&HUMAN_RIG_STAFF), "shadow": rest(&HUMAN_RIG_SHADOW) })
}

#[test]
fn the_rigs_rest_pose_is_what_the_body_scripts_read() {
    let now = rigs();
    if std::env::var("REGEN").is_ok() {
        std::fs::write(RIGS, serde_json::to_string_pretty(&now).unwrap() + "\n").unwrap();
        return;
    }
    let file: Value = serde_json::from_str(&std::fs::read_to_string(RIGS).expect("rigs.json (REGEN=1 writes it)")).unwrap();
    assert_eq!(file, now, "a rig changed: REGEN=1, then rebuild its body (`cargo xtask assets creatures --write`)");
}

const BUDGET: usize = 18_000;

/// The figures on the rig with a body of their own in `bodies.bin` (the
/// shadows keep their old one: their MakeHuman body lost or tied four
/// blind reviews, E9)
const FIGURES: [(&str, &HumanRig); 1] = [("staff", &HUMAN_RIG_STAFF)];

#[test]
fn the_figures_bodies_fit_their_rigs_and_bind_whole() {
    let rigs = rigs();
    for (name, rig) in FIGURES {
        let def = game::assets::bodies::body(name).unwrap_or_else(|| panic!("bodies.bin: no {name}"));
        let want = rigs[name]["bones"].as_array().unwrap();
        assert_eq!(def.bones.len(), want.len(), "{name}: bones");
        for (b, w) in def.bones.iter().zip(want) {
            assert_eq!(b.name, w["name"].as_str().unwrap(), "{name}: bone order");
            for k in 0..3 {
                let d = (b.head[k] - w["head"][k].as_f64().unwrap()).abs();
                assert!(d < 1e-3, "{name}: {} stands {d} m off the rig's", b.name);
            }
        }
        let g = group();
        let mut parts = human_rig(&g, rig, None);
        bind_figure(&g, name, &mut parts, &|_| lambert!());
        assert!(parts.skin.is_some(), "{name}: no skin bound");
        assert_eq!(1 + parts.skins.len(), def.meshes.len(), "{name}: a part not bound");
        let tris: usize = def.meshes.iter().map(|m| m.t).sum();
        assert!(tris <= BUDGET, "{name}: {tris} triangles (budget {BUDGET})");
    }
    // the seated man, on the Loop's chair (its seat's top 0.51 m up)
    let def = game::assets::bodies::body("seated").expect("bodies.bin: no seated");
    let tris: usize = def.meshes.iter().map(|m| m.t).sum();
    assert!(tris <= BUDGET, "seated: {tris} triangles");
    let mut low = f64::MAX;
    for m in &def.meshes {
        let geo = game::assets::bodies::body_geometry("seated", &m.name).unwrap();
        let p = &geo.r().position;
        for k in (1..p.len()).step_by(3) {
            low = low.min(p[k] as f64);
        }
    }
    assert!((-0.005..0.02).contains(&low), "seated: his soles {low} m from the floor");
}

/// Every part of body `name` on the rig `rig` skinned on the CPU at the
/// rig's pose now: (rest positions, posed positions, triangles) per part.
fn skinned_parts(parts: &[Obj]) -> Vec<(Vec<Vec3>, Vec<Vec3>, Vec<u32>, Vec<bool>)> {
    let mut out = vec![];
    for sm in parts {
        sm.update_matrix_world(true);
        let r = sm.r();
        let Kind::Skinned { skeleton: Some(sk), bind_matrix, bind_matrix_inverse, .. } = &r.kind else { panic!("not skinned") };
        let mats: Vec<Mat4> = sk.bones.iter().zip(&sk.bone_inverses).map(|(b, inv)| Mat4::mul(&b.matrix_world(), inv)).collect();
        let geo = r.geo().unwrap();
        let gr = geo.r();
        let n = gr.position.len() / 3;
        let (mut rest, mut posed) = (Vec::with_capacity(n), Vec::with_capacity(n));
        // (on a hand: its strongest bone is the hand's or a finger's)
        let names: Vec<String> = sk.bones.iter().map(|b| b.name()).collect();
        let mut hand = Vec::with_capacity(n);
        for i in 0..n {
            let v = Vec3::new(gr.position[i * 3] as f64, gr.position[i * 3 + 1] as f64, gr.position[i * 3 + 2] as f64);
            let mut b = v;
            b.apply_matrix4(bind_matrix);
            let mut acc = Vec3::new(0.0, 0.0, 0.0);
            for k in 0..4 {
                let w = gr.skin_weight[i * 4 + k] as f64;
                if w <= 0.0 {
                    continue;
                }
                let mut q = b;
                q.apply_matrix4(&mats[gr.skin_index[i * 4 + k] as usize]);
                acc.add_scaled_vector(&q, w);
            }
            acc.apply_matrix4(bind_matrix_inverse);
            rest.push(v);
            posed.push(acc);
            let top = (0..4).max_by(|&a, &b| gr.skin_weight[i * 4 + a].total_cmp(&gr.skin_weight[i * 4 + b])).unwrap();
            let bone = &names[gr.skin_index[i * 4 + top] as usize];
            hand.push(bone.starts_with("finger") || bone.starts_with("hand"));
        }
        out.push((rest, posed, gr.index.clone().unwrap_or_default(), hand));
    }
    out
}

#[test]
fn the_figures_hold_together_as_they_walk() {
    for (name, rig) in FIGURES {
        let g = group();
        let mut parts = human_rig(&g, rig, None);
        bind_figure(&g, name, &mut parts, &|_| lambert!());
        let skins: Vec<Obj> = parts.skin.iter().cloned().chain(parts.skins.iter().map(|s| s.1.clone())).collect();
        let mut e = Entity::new(name, g.clone());
        e.parts = parts.to_map();
        // its longest stride (hunting), looking round at you beside it
        let st = GaitStyle { hunt: 1.0, glide: 0.0, reach: 0.0, look: 1.0, shiver: 0.0, tilt: 0.1, breath: 1.0 };
        let (mut worst, mut worst_at) = (0.0f64, String::new());
        let mut ratios = vec![];
        let (mut hand_worst, mut hand_ratios) = (0.0f64, vec![]);
        for k in 0..150 {
            let t = k as f64 / 60.0;
            g.position().z = t * 4.5;
            human_gait_entity(&mut e, v3(2.0, 1.6, t * 4.5), 1.0 / 60.0, t, rig, &st);
            if k % 15 != 14 {
                continue;
            }
            g.update_matrix_world(true);
            for (pi, (rest, posed, idx, hand)) in skinned_parts(&skins).iter().enumerate() {
                for tri in idx.as_chunks::<3>().0 {
                    for (a, b) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
                        let (a, b) = (a as usize, b as usize);
                        // (not slivers: a hem's edge levelled leaves edges of a
                        // fraction of a millimetre, and a ratio of nothing
                        // means nothing; a tear shows on the edges round it)
                        let l0 = rest[a].distance_to(&rest[b]);
                        if l0 < 0.002 {
                            continue;
                        }
                        let r = posed[a].distance_to(&posed[b]) / l0;
                        // (the hands apart: the skin between fingers that
                        // curl each its own way draws out more)
                        if hand[a] && hand[b] {
                            hand_ratios.push(r);
                            hand_worst = hand_worst.max(r);
                            continue;
                        }
                        ratios.push(r);
                        if r > worst {
                            worst = r;
                            worst_at = format!("part {pi}, frame {k}, at {:?}", rest[a]);
                        }
                    }
                }
            }
        }
        ratios.sort_by(|a, b| a.total_cmp(b));
        let p999 = ratios[(ratios.len() as f64 * 0.999) as usize];
        hand_ratios.sort_by(|a, b| a.total_cmp(b));
        let hand_p99 = hand_ratios.get((hand_ratios.len() as f64 * 0.99) as usize).copied().unwrap_or(1.0);
        println!(
            "{name}: edges stretched at most {worst:.2}× ({worst_at}), 99.9% within {p999:.2}×; the hands at most {hand_worst:.2}×, 99% within {hand_p99:.2}×"
        );
        assert!(p999 < STRETCH_P999, "{name}: 0.1% of its edges stretched past {p999:.2}×");
        assert!(worst < STRETCH_MAX, "{name}: an edge stretched {worst:.2}× ({worst_at})");
        assert!(hand_worst < STRETCH_HAND, "{name}: an edge of a hand stretched {hand_worst:.2}×");
    }
}

/// How far a body's edges may stretch in its walk: a knee or an elbow
/// bending folds and draws out the skin over it, and the skin between
/// fingers curling each its own way draws out more; a torn weight shows as
/// an edge many times its length (a waistband weighted to the thighs opened
/// 12× at the back; slivers left by a levelled hem, 47×, are not counted).
/// Measured at E9: the staff 1.59 / 2.21, its hands 1.52 (the shadow tried
/// then, its gaunt thighs parting and its claws, 2.34 / 3.23 and 5.93).
const STRETCH_P999: f64 = 1.8;
const STRETCH_MAX: f64 = 3.0;
const STRETCH_HAND: f64 = 3.0;
