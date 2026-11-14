//! A long smoke run of the entities section on the synthetic test level:
//! every creature type plus staff / shadow / gone records, audio on (the
//! entity voices through the soundscape), the world queries of world.rs as
//! they are (no nav override), the alert spawn, the dead state, re-spawns.
use engine::audio::AudioContext;
use game::base::records::Entity;
use game::entities::*;
use game::prelude::*;

fn finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

#[test]
fn entities_smoke() {
    set_entity_nav(None);
    for seed in 0..4u32 {
        let (mut g, dist) = Game::test_level(seed * 101 + 7, 31 + (seed as i32 % 2) * 8);
        g.level_num = 3 + seed as i32 * 2;
        g.init_entities();
        let ac = AudioContext::new_silent();
        ac.set_muted(true);
        g.audio_init_with(ac);
        assert!(g.soundscape().is_some());
        g.state = "playing".into();
        g.god_mode = true;
        with_seed(seed + 1, || g.spawn_entities(3, &dist));
        // (what the level's places call for: the ecology; the Reader comes after the build)
        assert_eq!(g.entities.len(), g.ecology().iter().filter(|k| **k != "reader").count());
        // one of each, near the player, plus records other modules own
        for (k, kind) in ["smiler", "hound", "watcher"].iter().enumerate() {
            let ei = g.add_entity(kind, 2 + k as i32, 1, 1);
            g.entities[ei].freeze = 0.0;
        }
        let gone = Entity::new("gone", group());
        gone.g.set_pos(1e5, 0.0, 1e5);
        g.entities.push(gone);
        let mut sh = Entity::new("shadow", group());
        sh.freeze = 0.5;
        g.entities.push(sh);
        let n0 = g.scene.children().len();
        let dt = 1.0 / 60.0;
        let (mut hunted, mut voiced) = (0, 0);
        for k in 0..2400 {
            let t = k as f64 * dt;
            // the player roams, runs, lights the lamp, loses their mind
            let ph = k as f64 / 2400.0;
            g.player.yaw = t * 0.7 + (t * 0.31).sin() * 2.0;
            let f = g.player.fwd();
            let step = if ph > 0.3 && ph < 0.5 { 6.0 } else { 2.5 } * dt;
            let mut p = g.player.pos;
            g.collide_move(&mut p, f.x * step, f.z * step, PLAYER_R);
            if g.is_wall_tile(g.tx(p.x), g.tz(p.z)) {
                p = g.player.pos;
            }
            g.player.pos = p;
            g.player.audible_range = if ph > 0.3 && ph < 0.5 {
                14.0
            } else if ph > 0.7 {
                0.5
            } else {
                6.0
            };
            g.player.flashlight_on = ph > 0.2 && ph < 0.8;
            if k == 1500 {
                g.player.sanity = 12.0;
            }
            if k == 600 {
                g.chime_flash_t = 0.5;
                g.chime_flash_color = Some("rgba(90, 160, 220".into());
                g.electric_surge_t = 0.3;
                g.static_chromatic_t = 0.3;
            }
            g.camera.set_pos(g.player.pos.x, EYE, g.player.pos.z);
            g.perf_t = t;
            g.now_ms = t * 1000.0;
            g.update_entities(dt, t);
            g.run_scheduled();
            if k == 900 {
                g.alert_entities_to_player();
            }
            for e in &g.entities {
                assert!(finite(e.pos()), "seed {seed} frame {k}: {} at {:?}", e.kind, e.pos());
                assert!(e.yaw.is_finite());
                if let Some(face) = &e.face {
                    assert!(finite(face.pos()));
                }
            }
            assert!(g.player.sanity.is_finite());
            hunted += g.entities.iter().filter(|e| e.state == "hunt").count();
            voiced = voiced.max(g.ent_ext.values().filter(|x| x.voice.is_some()).count());
        }
        assert!(hunted > 100, "seed {seed}: nobody hunted ({hunted})");
        assert!(voiced >= 3, "seed {seed}: entity voices ({voiced})");
        // the skinned bodies follow their entities (attached bind mode)
        for e in &g.entities {
            if let Some(sk) = e.parts.get("skin") {
                sk.update_world_matrix(true, false);
                let want = sk.matrix_world().inverted();
                let n = sk.r();
                let engine::scene::Kind::Skinned { bind_matrix_inverse, .. } = &n.kind else { panic!("skin is not skinned") };
                for i in 0..16 {
                    assert!((bind_matrix_inverse.e[i] - want.e[i]).abs() < 1e-9);
                }
            }
        }
        // someone hunted, the vignette moved
        assert!(g.ui.vignette.opacity().is_some());
        // the dead state only billboards the faces
        g.state = "dead".into();
        for k in 0..10 {
            g.camera.position().y = EYE + (k as f64).sin() * 0.01;
            for i in 0..g.entities.len() {
                g.update_face(i);
            }
        }
        // a fresh level: the old faces leave the scene, the side table is reset
        assert!(g.scene.children().len() >= n0);
        with_seed(seed + 9, || g.spawn_entities(2, &dist));
        assert_eq!(g.entities.len(), g.ecology().iter().filter(|k| **k != "reader").count());
        assert_eq!(
            g.scene.children().iter().filter(|c| g.entities.iter().any(|e| e.face.as_ref().is_some_and(|f| f.ptr_eq(c)))).count(),
            g.entities.iter().filter(|e| e.face.is_some()).count(),
            "every billboard face (the old creatures have one) is in the scene"
        );
        assert!(g.ent_ext.len() <= g.entities.len(), "ent_ext reset on spawn ({} entries)", g.ent_ext.len());
        let faces_in_scene = g.scene.children().len();
        // (since E4 no kind that roams has a billboard face: then nothing to remove)
        assert!(n0 == 0 || faces_in_scene < n0, "old faces removed from the scene ({faces_in_scene} vs {n0})");
    }
}

#[test]
fn rig_contract() {
    let (mut g, _dist) = Game::test_level(5, 21);
    // every kind `add_entity` makes (a level's spawns, the cry's extra one)
    // is a creature of the redesign, on a body from `bodies.bin`, with no
    // billboard face (the old builders went in E7)
    for kind in ["smiler", "hound", "watcher", "corner", "reader", "glass"] {
        let ei = g.add_entity(kind, 3, 3, 0);
        let c = g.creature(ei).unwrap_or_else(|| panic!("{kind}: not a creature of the redesign"));
        assert!(c.body.ix("root").is_some() || c.body.ix("hips").is_some(), "{kind}: no skeleton");
        assert!(g.entities[ei].face.is_none(), "{kind}: a billboard face");
    }
    // `with_skin_detail` on someone else's material (the anomalies' staff)
    let m = with_skin_detail(phong!({ color: 0xffffff, vertexColors: true }), 3.4, 0.35);
    assert_eq!(m.r().bump_scale, 0.35);
    assert!(m.r().bump_map.is_some());
    // `skin_body` fails cleanly when the rig lacks a bone
    let body = game::assets::bodies::body_geometry("staff", "skin").unwrap();
    assert!(skin_body(&group(), &body, &m).is_none());
}
