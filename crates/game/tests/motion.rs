//! The creatures' motion (`entities::motion`): their clips, the retarget onto
//! their bodies, and what each kind plays in each of its modes.
//!
//! - Every kind's library reads, and its neutral stance poses its body
//!   exactly at rest (the trunk, head and arms; the legs, and on all fours
//!   the arms, solved back onto their rest ends): a clip is laid on a body
//!   as its turn away from that stance, so a body bound in a stoop keeps it.
//! - Every mode its brain puts it in, still and moving, over your body or
//!   not, plays a clip its library has.
//! - A body bound with straight legs (under a long garment) stands straight.
//! - Planted feet stay planted while it walks.
//! - A pose depends only on what drives it and its own seed (invariant 1).
//! - Every bridge (the motion between two clips, made by MotionBricks) joins
//!   the clips it goes between, the Hound's on all fours; a change of clip
//!   goes by its bridge, with no joint jumping.
use engine::prelude::*;
use game::entities::body::{BodyRig, build_body};
use game::entities::brain::Mode;
use game::entities::glass::AtGlass;
use game::entities::motion::anim::{Anim, Drive};
use game::entities::motion::retarget::{Ends, Gains};
use game::entities::motion::{BRIDGE_LANDS, Frame, KINDS, Mind, body_to_world, library};
use game::entities::reader::Page;

fn body(kind: &str) -> BodyRig {
    build_body(kind, &|_| lambert!()).unwrap_or_else(|| panic!("bodies.bin: no {kind}"))
}

fn anim(kind: &str, rig: &BodyRig, seed: u64) -> Anim {
    let lib = library(kind).unwrap_or_else(|| panic!("assets/motion/{kind}.motion does not read"));
    Anim::new(rig, lib, kind == "hound", lib.clips[0].name.as_str(), seed).expect("a humanoid map")
}

fn angle(q: &Quat) -> f64 {
    2.0 * q.w.abs().min(1.0).dacos()
}

/// One frame of kind `kind`'s own pose (the function the game calls).
fn pose(kind: &str, a: &mut Anim, rig: &BodyRig, d: &Drive, m: &Mind) {
    match kind {
        "smiler" => game::entities::smiler::pose(a, rig, d, m),
        "hound" => game::entities::hound::pose(a, rig, d, m),
        "corner" => game::entities::corner::pose(a, rig, d, m, 0.0),
        "reader" => game::entities::reader::pose(a, rig, d, m, &Page { stage: 1.0, t: 0.3, tears: false }),
        _ => game::entities::glass::pose(a, rig, d, m, &AtGlass::default()),
    }
}

#[test]
fn every_neutral_stance_is_its_bodys_rest_pose() {
    for kind in KINDS {
        let lib = library(kind).unwrap_or_else(|| panic!("{kind}: no library"));
        assert!(lib.clips.len() >= 4, "{kind}: {} clips", lib.clips.len());
        let rig = body(kind);
        let a = anim(kind, &rig, 1);
        let rt = &a.rt;
        assert!(rt.k > 0.7 && rt.k < 1.7, "{kind}: its legs over the mocap's {}", rt.k);
        rt.pose(&rig, &lib.neutral, &Gains::default(), &Ends::default());
        let m = &rt.map;
        let mut bones = vec![m.hips, m.neck, m.head];
        bones.extend(m.spine);
        bones.extend(m.chest);
        bones.extend(m.upperchest);
        bones.extend(m.arm.iter().flat_map(|a| if rt.fours { vec![a[0]] } else { a.to_vec() }));
        for b in bones {
            let turned = angle(&rig.body_rot(b));
            assert!(turned < 1e-6, "{kind}: {} turned {turned} rad from rest", rig.def.bones[b].name);
        }
        let mut ends: Vec<usize> = m.leg.iter().map(|l| l[2]).collect();
        if rt.fours {
            ends.extend(m.arm.iter().map(|a| a[3]));
        }
        for e in ends {
            let (at, want) = (rig.head(e), rig.def.bones[e].head);
            let off = (at.x - want[0]).dhypot(at.y - want[1]).dhypot(at.z - want[2]);
            assert!(off < 3e-3, "{kind}: {} {off} m from its rest", rig.def.bones[e].name);
        }
    }
}

#[test]
fn every_mode_plays_a_clip_its_library_has() {
    let modes =
        [Mode::Idle, Mode::Roam, Mode::Alert, Mode::Search, Mode::Stalk, Mode::Hunt, Mode::Strike, Mode::Withdraw, Mode::Hold, Mode::Read];
    for kind in KINDS {
        let rig = body(kind);
        let mut a = anim(kind, &rig, 7);
        let eye = v3(0.0, 1.6, 6.0);
        for mode in modes {
            for (speed, killer) in [(0.0, false), (1.0, false), (3.5, false), (6.0, false), (0.0, true)] {
                let mut pos = v3(0.0, 0.0, 0.0);
                for f in 0..20 {
                    let mind = Mind { mode, mode_t: f as f64 / 30.0, killer, eye, heard: None, chase_t: 0.2 };
                    pos = v3(pos.x, 0.0, pos.z + speed / 30.0);
                    let d = a.drive(pos, 0.0, 1.0 / 30.0);
                    pose(kind, &mut a, &rig, &d, &mind);
                }
                assert!(a.missing.is_none(), "{kind} in {mode:?} at {speed} m/s (over you: {killer}) asked for {:?}", a.missing);
                for (i, b) in rig.bones.iter().enumerate() {
                    let q = b.quaternion();
                    assert!(
                        q.x.is_finite() && q.y.is_finite() && q.z.is_finite() && q.w.is_finite(),
                        "{kind}: {} not finite",
                        rig.def.bones[i].name
                    );
                }
            }
        }
    }
}

#[test]
fn a_body_bound_with_straight_legs_stands_straight() {
    // (the one in the corner's nightshirt and the one behind the glass's coat
    // are skinned to the legs: a knee bent for nothing creased them)
    for kind in ["corner", "glass"] {
        let rig = body(kind);
        let mut a = anim(kind, &rig, 3);
        for f in 0..60 {
            let d = a.drive(v3(0, 0, 0), 0.0, 1.0 / 30.0);
            pose(kind, &mut a, &rig, &d, &Mind::new(Mode::Hold, f as f64 / 30.0, v3(0.0, 1.6, 6.0)));
        }
        let m = &a.rt.map;
        let drop = rig.def.bones[m.hips].head[1] - rig.head(m.hips).y;
        assert!(drop < 0.015, "{kind}: its hips {drop} m below rest as it stands");
        for l in &m.leg {
            let bent = angle(&{
                let mut q = rig.body_rot(l[0]);
                q.invert();
                let mut s = q;
                s.multiply(&rig.body_rot(l[1]));
                s
            });
            assert!(bent < 0.2, "{kind}: a knee bent {bent} rad as it stands");
        }
    }
}

#[test]
fn planted_feet_stay_planted_while_it_walks() {
    // each foot, while its lock holds and it is on the floor: how far it
    // moves across the floor
    for (kind, speed) in [("smiler", 1.6), ("reader", 0.9), ("glass", 2.6), ("hound", 2.0)] {
        let rig = body(kind);
        let mut a = anim(kind, &rig, 11);
        let mode = if kind == "glass" { Mode::Stalk } else { Mode::Roam };
        let foot = |rig: &BodyRig, a: &Anim, s: usize, pos: Vec3| body_to_world(pos, 0.0, rig.head(a.rt.map.leg[s][2]));
        let (mut slid, mut held) = (0.0, 0);
        let mut last: [Option<Vec3>; 2] = [None, None];
        let mut pos = v3(0, 0, 0);
        for f in 0..240 {
            pos = v3(0.0, 0.0, pos.z + speed / 30.0);
            let d = a.drive(pos, 0.0, 1.0 / 30.0);
            pose(kind, &mut a, &rig, &d, &Mind::new(mode, f as f64 / 30.0, v3(0.0, 1.6, 50.0)));
            let locks = a.locks();
            // (the Hound's left hind drags by design: its right one)
            for s in if kind == "hound" { 1..2 } else { 0..2 } {
                let p = foot(&rig, &a, s, pos);
                // (on the floor: a foot lifting away at the end of its step is not
                // skating)
                let floor = rig.def.bones[a.rt.map.leg[s][2]].head[1];
                if f > 30
                    && locks[s].0
                    && p.y < floor + 0.03
                    && let Some(q) = last[s]
                {
                    slid += (p.x - q.x).dhypot(p.z - q.z);
                    held += 1;
                }
                last[s] = locks[s].0.then_some(p);
            }
        }
        let per = slid * 30.0 / f64::max(held as f64, 1.0);
        assert!(held > 40, "{kind}: its feet were planted only {held} frames");
        assert!(per < 0.08, "{kind}: a planted foot slid {per} m/s at {speed} m/s");
    }
}

#[test]
fn a_pose_is_a_function_of_what_drives_it() {
    for kind in KINDS {
        let (r1, r2) = (body(kind), body(kind));
        let (mut a1, mut a2) = (anim(kind, &r1, 99), anim(kind, &r2, 99));
        for f in 0..90 {
            let mode = [Mode::Search, Mode::Hunt, Mode::Alert][f / 30];
            let pos = v3((f as f64 * 0.1).dsin(), 0.0, f as f64 * 0.05);
            let m = Mind::new(mode, (f % 30) as f64 / 30.0, v3(0.0, 1.6, 6.0));
            let d1 = a1.drive(pos, 0.1, 1.0 / 30.0);
            let d2 = a2.drive(pos, 0.1, 1.0 / 30.0);
            pose(kind, &mut a1, &r1, &d1, &m);
            pose(kind, &mut a2, &r2, &d2, &m);
        }
        for (i, (b1, b2)) in r1.bones.iter().zip(&r2.bones).enumerate() {
            let (q1, q2) = (*b1.quaternion(), *b2.quaternion());
            assert!(
                q1.x.to_bits() == q2.x.to_bits() && q1.w.to_bits() == q2.w.to_bits(),
                "{kind}: {} differs between two runs of the same input",
                r1.def.bones[i].name
            );
        }
    }
}

/// The most any joint of one frame is from the same joint of another, m.
fn gap(x: &Frame, y: &Frame) -> f64 {
    x.p.iter().zip(&y.p).map(|(p, q)| (p.x - q.x).dhypot(p.y - q.y).dhypot(p.z - q.z)).fold(0.0, f64::max)
}

#[test]
fn every_bridge_joins_the_clips_it_goes_between() {
    // its first frame is its first clip's where it leaves it, its last its
    // second's where it lands (to the 0.5 mm it is kept in), no later than
    // BRIDGE_LANDS into it; it plays once; the Hound's stay on all fours
    for kind in KINDS {
        let lib = library(kind).expect("a library");
        let mut n = 0;
        for c in &lib.clips {
            let Some(b) = &c.bridge else { continue };
            n += 1;
            let from = &lib.clips[lib.clip(&b.from).unwrap_or_else(|| panic!("{kind} {}: no clip {}", c.name, b.from))];
            let to = &lib.clips[lib.clip(&b.to).unwrap_or_else(|| panic!("{kind} {}: no clip {}", c.name, b.to))];
            assert!(!c.looped, "{kind} {}: a bridge loops", c.name);
            assert!(b.to_t <= BRIDGE_LANDS, "{kind} {}: lands {} s into {}", c.name, b.to_t, b.to);
            let g0 = gap(&c.frames[0], &from.sample(b.from_t, lib.fps));
            let g1 = gap(c.frames.last().expect("frames"), &to.sample(b.to_t, lib.fps));
            assert!(g0 < 0.005, "{kind} {}: leaves {} {g0} m from where it is", c.name, b.from);
            assert!(g1 < 0.005, "{kind} {}: lands {g1} m from {}", c.name, b.to);
            if kind == "hound" {
                let head = c.frames.iter().map(|f| f.p[game::entities::motion::j::HEAD].y).fold(0.0, f64::max);
                assert!(head < 1.0, "{kind} {}: its head rose to {head} m", c.name);
            }
        }
        assert!(n > 20, "{kind}: {n} bridges");
    }
}

#[test]
fn a_change_of_clip_goes_by_its_bridge() {
    // the Smiler walking on, then stopping: it plays the bridge from its
    // walk into its stand (waiting for none: it has stopped), lands on its
    // idle with its feet where its stand wants them, and from the stop on no
    // joint moves faster in a frame than with the old blend
    let kind = "smiler";
    let run = |bridges: bool| {
        let rig = body(kind);
        let mut a = anim(kind, &rig, 5);
        a.bridges = bridges;
        let (mut bridged, mut after, mut jump) = (None, None, (0.0f64, 0, 0));
        let mut last: Option<Vec<Vec3>> = None;
        let mut pos = v3(0, 0, 0);
        for f in 0..180 {
            if f < 75 {
                pos = v3(0.0, 0.0, pos.z + 0.7 / 30.0);
            }
            let d = a.drive(pos, 0.0, 1.0 / 30.0);
            pose(kind, &mut a, &rig, &d, &Mind::new(Mode::Roam, 2.0 + f as f64 / 30.0, v3(0.0, 1.6, 50.0)));
            if f >= 75 && bridged.is_none() && a.bridging().is_some() {
                bridged = Some(f);
            }
            if bridged.is_some() && after.is_none() && a.bridging().is_none() {
                after = Some((f, a.playing().0));
            }
            let now: Vec<Vec3> = (0..rig.bones.len()).map(|i| rig.head(i)).collect();
            if (76..130).contains(&f)
                && let Some(l) = &last
            {
                for (i, (p, q)) in now.iter().zip(l).enumerate() {
                    let dj = (p.x - q.x).dhypot(p.y - q.y).dhypot(p.z - q.z);
                    if dj > jump.0 {
                        jump = (dj, f, i);
                    }
                }
            }
            last = Some(now);
        }
        // standing at the end: how far its feet are from where its clip wants them
        let gap = a.locks().iter().take(2).map(|l| l.1).fold(0.0, f64::max) / a.rt.k;
        (bridged, after, jump, rig.def.bones[jump.2].name.clone(), gap)
    };
    let (bridged, after, jump, bone, gap) = run(true);
    let (_, _, blend, bone_blend, _) = run(false);
    println!(
        "from the stop on, the most a joint moved in a frame: {} m by its bridge ({bone}), {} m by the blend ({bone_blend})",
        jump.0, blend.0
    );
    let b = bridged.expect("it stopped with no bridge (the blend)");
    // (`moving` eases off over a few frames; then it leaves at once)
    assert!(b < 75 + 12, "its bridge began {} frames after it stopped", b - 75);
    let (f, clip) = after.expect("its bridge never ended");
    assert_eq!(clip, "idle", "it landed on {clip} (frame {f})");
    // its feet set where its stand wants them (a stance its bridge left
    // narrow is stepped right, never held crossed)
    assert!(gap < 0.1, "standing, a foot is {gap} m (on the mocap's scale) from where its clip wants it");
    assert!(
        jump.0 <= blend.0.max(0.05) * 1.1,
        "{bone} moved {} m in frame {} with its bridge, {bone_blend} {} m with the blend",
        jump.0,
        jump.1,
        blend.0
    );
}
