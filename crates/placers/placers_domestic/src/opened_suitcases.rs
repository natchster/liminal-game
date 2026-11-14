//! `place_opened_suitcases`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::salt;
use placers_kit::place::scatter;

// Opened suitcase: 5% chance per level. An old hard-shell suitcase
// left open on the floor (r02's: a 1960s moulded case, 23 x 16 x 7 in, its
// lid stood up on its straps), gone through: what was packed in it pushed
// about, four to six things, a cardigan pulled half out over its rim.
pub(crate) fn place_opened_suitcases(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old leather draw: the shell's vinyl (saddle, oxblood, tan, black), dulled
    let leather_colors = [0x603820, 0x502010, 0x8a5030, 0x201a10];
    let lc = pick_hex(&leather_colors);
    let li = leather_colors.iter().position(|&c| c == lc).unwrap_or(0);
    let shell = lambert!({ color: [0x6e5038u32, 0x5a3029, 0x86684a, 0x2e2a26][li] });
    // the old cloths' colours: each thing's own palette, by the pick
    let cloth_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0, 0xf0ece0, 0x2a2418];
    const PALETTES: [[u32; 4]; 5] = [
        [0x38383a, 0x2a2e3a, 0x4a3c2e, 0x5e5a54], // trousers: charcoal, navy, brown, grey
        [0x9a8e76, 0x34463a, 0x5a2c2c, 0x6a6864], // jumper: oatmeal, bottle green, burgundy, grey
        [0xc4beb0, 0x9eaab4, 0xb8ae98, 0xbca49c], // shirt: white, pale blue, cream, faded pink
        [0x2c2a2a, 0x625e5a, 0x2a3040, 0x4a3c30], // socks
        [0x76623e, 0x5a2a2c, 0x2c3446, 0x56626a], // cardigan: mustard, maroon, navy, grey-blue
    ];
    // 4-6 things in it, each drawn as the old cloth was: colour, three
    // sizes, where (x, z), its turn and tilt
    let n_clothes = 4 + (rnd() * 3.0).floor() as i32;
    let mut things = vec![];
    for _i in 0..n_clothes {
        let cc = pick_hex(&cloth_colors);
        let k = cloth_colors.iter().position(|&c| c == cc).unwrap_or(0) % 4;
        let (sw, _sh, sd) = (rnd(), rnd(), rnd());
        let (px, pz) = (rnd() - 0.5, rnd() - 0.5);
        let (yaw, tilt) = (rnd(), rnd() - 0.5);
        things.push((k, sw, sd, px, pz, yaw, tilt));
    }
    let salt = salt(g.wx(x), g.wz(z), li as u32);
    let suitcase = group();
    let put_at = |parent: &Obj, name: &str, mat: &Mat, x: f64, y: f64, z: f64, yaw: f64| {
        if let Some(geo) = prop_part_as_modelled(name) {
            let m = mesh(geo, mat);
            m.position().set(x, y, z);
            m.rotation().y = yaw;
            parent.add(&m);
        }
    };
    let lining = lambert!({ map: &lining_tex(salt) });
    let trim = phong!({ color: 0x8a8a86, specular: 0x6a6a66, shininess: 50 });
    let grip = lambert!({ color: 0x2a231e });
    for (name, mat) in [
        ("Real__suitcaseBase", &shell),
        ("Real__suitcaseBaseLining", &lining),
        ("Real__suitcaseBaseTrim", &trim),
        ("Real__suitcaseHandle", &grip),
    ] {
        put_at(&suitcase, name, mat, 0.0, 0.0, 0.0, 0.0);
    }
    let lid = group();
    lid.position().set(0, HINGE_Y, HINGE_Z);
    lid.rotation().x = -OPEN;
    for (name, mat) in [("Real__suitcaseLid", &shell), ("Real__suitcaseLidLining", &lining), ("Real__suitcaseLidTrim", &trim)] {
        put_at(&lid, name, mat, 0.0, -HINGE_Y, -HINGE_Z, 0.0);
    }
    suitcase.add(&lid);
    // what is left in it, each about where it was packed and pushed about
    // by a hand going through it: trousers folded at the back, the jumper
    // at the front, a shirt on the trousers, socks in a corner; a fifth, a
    // cardigan pulled out over the rim; a sixth, another jumper on the first
    // (the jumpers' fronts inside the case's front wall however they turn:
    // at 85 mm in, the first poked through it, M8)
    const SLOTS: [(&str, usize, f64, f64, f64); 6] = [
        ("Real__suitcaseTrousers", 0, -0.07, -0.085, 0.0),
        ("Real__suitcaseSweater", 1, 0.12, 0.050, 0.0),
        ("Real__suitcaseShirt", 2, -0.11, -0.08, 0.050),
        ("Real__suitcaseSocks", 3, -0.205, 0.12, 0.0),
        ("Real__suitcaseSpill", 4, 0.0, 0.0, 0.0),
        ("Real__suitcaseSweater", 4, 0.11, 0.050, 0.075),
    ];
    let flip = if things[0].6 > 0.0 { -1.0 } else { 1.0 };
    for (i, &(k, sw, sd, px, pz, yaw, _)) in things.iter().enumerate() {
        let (name, pal, sx, sz, sy) = SLOTS[i.min(5)];
        let cloth = lambert!({ color: PALETTES[pal][k] });
        if i == 4 {
            put_at(&suitcase, name, &cloth, px * 0.08, 0.0, 0.0, 0.0);
            continue;
        }
        let turn = (yaw - 0.5) * 0.16 + (sw - 0.5) * 0.04 + if i == 3 { 0.5 } else { 0.0 };
        // a jumper is wider than the room in front of it: turned less
        let turn = if name == "Real__suitcaseSweater" { turn * 0.4 } else { turn };
        put_at(&suitcase, name, &cloth, flip * (sx + px * 0.02), FLOOR + sy, sz + pz * 0.02 + (sd - 0.5) * 0.01, flip * turn);
    }
    scatter(g, &suitcase, x, z, 1.8);
    g.world.add(&suitcase);
}

/// r02's case: its hinge line (height, back) and the lining's floor inside
const HINGE_Y: f64 = 0.112;

const HINGE_Z: f64 = -0.209;

const FLOOR: f64 = 0.0063;

/// How far its lid stands open, held by its check straps
const OPEN: f64 = PI / 2.0 + 0.21;

/// The lining: rayon gone the colour of weak tea, its stripe woven in,
/// tide marks where it got wet, grubby toward its corners and its rim
fn lining_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x11e;
        fill(ctx, w, h, |u, v| {
            let stripe = if ((u * w) as i32 % 6) < 2 { 0.9 } else { 1.0 };
            let n = fbm(seed, u * 8.0, v * 8.0, 4);
            let mut c = mix([0.54, 0.49, 0.44], [0.44, 0.39, 0.34], n);
            c = [c[0] * stripe, c[1] * stripe, c[2] * stripe];
            for (cu, cv, r) in [(0.3, 0.6, 0.18), (0.72, 0.35, 0.12)] {
                let d = (u - cu).dhypot(v - cv) / r + (fbm(seed ^ 3, u * 9.0, v * 9.0, 3) - 0.5) * 0.4;
                c = mix(c, [0.40, 0.33, 0.26], (1.0 - ((d - 1.0).abs() / 0.05)).max(0.0) * 0.5);
            }
            let edge = sstep(0.12, 0.0, u.min(1.0 - u).min(v.min(1.0 - v)));
            c = mix(c, [0.24, 0.19, 0.14], edge * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}
