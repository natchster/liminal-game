//! `place_skull_stacks`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_tile;
use placers_kit::place::salt;

// Skull stack cluster: 4% chance per level. Life-size cast resin skulls
// (the 19.6 cm kind sold as replicas), aged: an ivory resin under a brown
// wash that lies dark in the orbits, the nose and the ears, the teeth a
// yellower resin. Two, one set on the other's crown; or three, a pair side
// by side and one across them; each a little askew.
pub(crate) fn place_skull_stacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let n_skulls = 2 + (rnd() * 2.0).floor() as i32;
    // each skull's old offset, turn and (above the first) tilt: here how far
    // each is set off its place (a few mm), turned and tilted
    let mut each = vec![];
    for i in 0..n_skulls {
        let ox = (rnd() - 0.5) * 0.04;
        let oz = (rnd() - 0.5) * 0.04;
        let yaw = rnd() * PI * 2.0;
        let roll = if i > 0 { (rnd() - 0.5) * 0.4 } else { 0.0 };
        each.push((ox * 0.15, oz * 0.15, (yaw / (PI * 2.0) - 0.5) * 0.5, roll * 0.3));
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let turn = rnd() * PI * 2.0;
    let places: &[(f64, f64)] =
        if n_skulls == 2 { &[(0.0, 0.0), (0.0, STACK_ON_ONE)] } else { &[(-STACK_APART, 0.0), (STACK_APART, 0.0), (0.0, STACK_ON_TWO)] };
    let bone = lambert!({ map: &bone_tex(salt(px, pz, 4)) });
    let wash = lambert!({ color: 0x3e3226 });
    let teeth = lambert!({ color: 0xbfae88 });
    let cluster = group();
    for (&(sx, sy), &(ox, oz, yaw, roll)) in places.iter().zip(each.iter()) {
        let skull = group();
        put(&skull, "Real__skullStackBone", &bone);
        put(&skull, "Real__skullStackHollows", &wash);
        put(&skull, "Real__skullStackTeeth", &teeth);
        // a tilted one sits a little higher on its rim
        skull.position().set(sx + ox, sy + roll.abs() * 0.06, oz);
        set_rot_yz(&skull, yaw, roll);
        cluster.add(&skull);
    }
    cluster.position().set(px, 0, pz);
    cluster.rotation().y = turn;
    g.world.add(&cluster);
}

/// The stack (`t09.py`, measured on the model): where a skull sits on one
/// (a pair) or across two side by side (three), and how far apart the two
/// below stand
const STACK_ON_ONE: f64 = 0.201;

const STACK_ON_TWO: f64 = 0.207;

const STACK_APART: f64 = 0.077;

fn set_rot_yz(o: &Obj, yaw: f64, roll: f64) {
    o.rotation().y = yaw;
    o.rotation().z = roll;
}

/// Aged resin (laid on the skull from every side): ivory under a brown
/// wash, the wash pooled darker in blotches and along the sutures' lines,
/// a little rubbed off on what stands out, dust
fn bone_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x5c0;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 5.0, v * 5.0, 4);
            let mut c = mix([0.80, 0.74, 0.60], [0.66, 0.58, 0.44], n);
            // the wash: a brown that pooled in patches
            let wash = sstep(0.52, 0.72, fbm(seed ^ 3, u * 9.0, v * 9.0, 4));
            c = mix(c, [0.48, 0.39, 0.28], wash * 0.55);
            // the sutures: thin wandering lines the wash ran into
            let sut = (fbm(seed ^ 5, u * 4.0, v * 4.0, 3) - 0.5).abs();
            c = mix(c, [0.52, 0.43, 0.32], (1.0 - sstep(0.002, 0.008, sut)) * 0.35);
            // pores and specks
            let p = vn(seed ^ 7, u * 200.0, v * 200.0);
            if p > 0.84 {
                c = mix(c, [0.42, 0.36, 0.28], (p - 0.84) * 3.0);
            }
            let dust = vn(seed ^ 9, u * 140.0, v * 140.0);
            if dust > 0.86 {
                c = mix(c, [0.70, 0.68, 0.64], (dust - 0.86) * 3.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}
