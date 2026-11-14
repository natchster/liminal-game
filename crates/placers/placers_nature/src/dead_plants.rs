//! `place_dead_plants`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::noise::worley;
use placers_kit::paint::opaque as rgb;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Dead potted plants: 0-1 per level (25% chance).
pub(crate) fn place_dead_plants(g: &mut Game, _dist: &[i32]) {
    // Zonal geraniums in a 31 cm terracotta pot (batch `r18`), not watered for
    // years: the stems woody and grey, knuckled at their nodes, a leaf or two
    // dried on them, the rest fallen; the soil shrunk from the wall and
    // cracked; salts bloomed white through the clay (r18.notes.md). One in
    // twelve is still alive, leggy, a few green leaves at its tips.
    use game::assets::props::prop_part_as_modelled;
    g.pp.living_plant_positions.clear(); // reset per level
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let pk = pick(&open);
    // 8% chance the plant is still ALIVE
    let is_alive = rnd() < 0.08;
    let plant = group();
    let salt = (pk.0 as u32).wrapping_mul(0x9e37_79b9) ^ (pk.1 as u32).wrapping_mul(0x85eb_ca6b);
    let put = |grp: &Obj, name: &str, mat: &Mat| -> Option<Obj> {
        let geo = prop_part_as_modelled(name)?;
        let m = mesh(geo, mat);
        grp.add(&m);
        Some(m)
    };
    put(&plant, "Real__deadPlantPot", &lambert!({ map: &pot_tex(salt) }));
    put(&plant, "Real__deadPlantSoil", &lambert!({ map: &soil_tex(salt) }));
    let stem_mat = if is_alive { lambert!({ color: 0x5f6a3e }) } else { lambert!({ color: 0x6f665a }) };
    let leaf_mat = if is_alive {
        lambert!({ map: &leaf_tex(true, salt), side: DoubleSide })
    } else {
        lambert!({ map: &leaf_tex(false, salt), side: DoubleSide })
    };
    if !is_alive {
        put(&plant, "Real__deadPlantLitter", &leaf_mat);
    }
    // 3-4 stems, each its old draws: its way out, how far it leans, how deep
    // its foot is and the turn of the leaf at its top
    let stem_count = 3 + (rnd() * 2.0).floor() as i32;
    for s in 0..stem_count {
        let ang = (s as f64 / stem_count as f64) * PI * 2.0 + rnd() * 0.5;
        // living plants barely lean; the dead lean out further
        let droop = if is_alive { 0.05 + rnd() * 0.15 } else { 0.4 + rnd() * 0.4 };
        let lean = if is_alive { droop * 0.6 } else { 0.12 + (droop - 0.4) * 0.7 };
        let stem_dir = v3(ang.cos() * lean.sin(), lean.cos(), ang.sin() * lean.sin());
        // its foot 1-3 cm down in the soil (its surface 0.150), off the centre
        let stem_foot = v3(ang.cos() * 0.03, SOIL_Y - 0.03 + (0.172 + rnd() * 0.02 - 0.172), ang.sin() * 0.03);
        let spin = rnd() * PI;
        let k = s as usize % 3;
        let stem = group();
        stem.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &stem_dir);
        stem.position().copy(&stem_foot);
        // the stem's own lean turned outward
        let inner = group();
        inner.rotation().y = STEM_LEAN_AZ[k] - ang;
        stem.add(&inner);
        put(&inner, ["Real__deadPlantStemA", "Real__deadPlantStemB", "Real__deadPlantStemC"][k], &stem_mat);
        // a leaf at its top node, dried on it (or living)
        let (c, d) = STEM_TOP[k];
        let leaf = group();
        leaf.position().set(c[0], c[1], c[2]);
        leaf.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &v3(d[0], d[1], d[2]));
        leaf.rotate_y(spin);
        put(&leaf, if is_alive { "Real__deadPlantLeafLive" } else { "Real__deadPlantLeafDry" }, &leaf_mat);
        inner.add(&leaf);
        plant.add(&stem);
    }
    // Position near wall (bias toward corners).
    let wall_bias = rnd() < 0.75;
    let ox = if wall_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (1.3 + rnd() * 0.3) } else { (rnd() - 0.5) * 2.4 };
    let oz = if wall_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (1.3 + rnd() * 0.3) } else { (rnd() - 0.5) * 2.4 };
    plant.position().set(g.wx(pk.0) + ox, 0, g.wz(pk.1) + oz);
    plant.rotation().y = rnd() * PI * 2.0;
    g.world.add(&plant);
    let pp = plant.pos();
    // Track living plants for rustle audio events.
    if is_alive {
        g.pp.living_plant_positions.push(v3(pp.x, 0.4, pp.z));
        // the old glow's draws, kept: a plant gives no light
        if rnd() < 0.05 {
            if rnd() < 0.30 {
                let _ = rnd(); // keeps the old draw (its pulse's phase)
            }
        }
    }
    // 40% chance: dry leaves fallen round the pot (dead plants only),
    // each its old draws: its size, its tilt, its turn, where it lies
    if !is_alive && rnd() < 0.40 {
        let debris_count = 3 + (rnd() * 3.0).floor() as i32;
        for d in 0..debris_count {
            let (sw, _sh) = (rnd(), rnd());
            let _tilt = rnd(); // keeps the old draw (it lies as it curled)
            let turn = rnd() * PI * 2.0;
            let ang = rnd() * PI * 2.0;
            let rad = 0.20 + rnd() * 0.20;
            let leaf = group();
            put(&leaf, "Real__deadPlantFallen", &leaf_mat);
            leaf.scale().set_scalar(0.85 + 0.3 * sw);
            leaf.rotation().y = turn;
            leaf.position().set(pp.x + ang.cos() * rad, 0.0, pp.z + ang.sin() * rad);
            g.world.add(&leaf);
        }
    }
}

/// The soil's surface in the pot
const SOIL_Y: f64 = 0.150;

/// Each stem part's top node and its direction there, in the part's frame
/// (printed by `r18.py`)
const STEM_TOP: [([f64; 3], [f64; 3]); 3] = [
    ([0.04409, 0.27403, 0.03280], [0.3067, 0.9472, 0.0939]),
    ([0.01691, 0.26503, -0.00006], [0.2041, 0.9717, -0.1187]),
    ([0.04742, 0.26873, -0.05838], [0.2606, 0.9368, -0.2335]),
];

/// The way each stem part leans of itself (its top's azimuth from +x
/// toward +z), turned outward in the pot
const STEM_LEAN_AZ: [f64; 3] = [0.6397, -0.0035, -0.8885];

const POT_H: f64 = 0.193;

/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right,
/// v up, both 0..1, as the texture is laid) gives straight sRGB and alpha
fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
    let (iw, _) = (w as usize, h as usize);
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, iw * 4, |j, out| {
        for i in 0..iw {
            let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
            for k in 0..4 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

/// The pot, round it and up it: terracotta faded, the salts bloomed white
/// through it in a band under the rim and in patches, green-black where it
/// stood damp at its foot, dust, a chip or two to the fresh clay
fn pot_tex(salt: u32) -> Tex {
    canvas_tex_opt(384, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x7e11;
        let wrap = |sd: u32, u: f64, y: f64, fu: f64, fy: f64, o: u32| {
            fbm(sd, u * fu, y * fy, o) * (1.0 - u) + fbm(sd, (u - 1.0) * fu, y * fy, o) * u
        };
        fill(ctx, w, h, |u, v| {
            let y = v * POT_H;
            let clay = [0.63, 0.40, 0.29];
            let mut c = mix(clay, [0.55, 0.34, 0.25], wrap(seed, u, y, 30.0, 25.0, 4));
            // the white bloom: a band under the collar and patches
            let bloom = (wrap(seed ^ 1, u, y, 18.0, 14.0, 4) - 0.42) * 2.6 + sstep(0.09, 0.15, y) * 0.5 - sstep(0.165, 0.19, y) * 0.25;
            let lace = 0.75 + 0.25 * wrap(seed ^ 5, u, y, 160.0, 140.0, 2);
            c = mix(c, [0.8, 0.78, 0.72], bloom.clamp(0.0, 1.0) * 0.85 * lace);
            // the foot dark where it stood in its own water, green-black
            let foot = sstep(0.05, 0.0, y) * (0.6 + 0.4 * wrap(seed ^ 2, u, y, 50.0, 60.0, 3));
            c = mix(c, [0.2, 0.21, 0.15], foot * 0.85);
            // dust and grime over all
            c = mix(c, [0.45, 0.42, 0.37], 0.18 + 0.1 * wrap(seed ^ 3, u, y, 8.0, 6.0, 2));
            // pits in the clay
            let pit = (vn(seed ^ 4, u * 700.0, y * 1200.0) - 0.8).max(0.0) * 3.0;
            c = mix(c, [0.32, 0.22, 0.17], pit);
            rgb(c)
        });
    })
}

/// The soil from above: a dry grey-brown crust cracked into plates,
/// paler where it dried last, perlite's white grains, a crumb or two
fn soil_tex(salt: u32) -> Tex {
    canvas_tex_opt(192, 192, false, move |ctx, w, h| {
        let seed = salt ^ 0x5011;
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 0.26, v * 0.26);
            // plates of every size: the cracks meander (the lattice warped)
            let (wx, wy) =
                (x + 0.012 * (fbm(seed ^ 6, x * 30.0, y * 30.0, 2) - 0.5), y + 0.012 * (fbm(seed ^ 7, x * 30.0, y * 30.0, 2) - 0.5));
            let (d1, d2) = worley(seed, wx / 0.034, wy / 0.034);
            let (e1, e2) = worley(seed ^ 9, wx / 0.012, wy / 0.012);
            let crack = sstep(0.045, 0.0, d2 - d1).max(sstep(0.03, 0.0, e2 - e1) * 0.5);
            let mut c = mix([0.27, 0.22, 0.17], [0.38, 0.33, 0.27], fbm(seed ^ 1, x * 90.0, y * 90.0, 4));
            c = mix(c, [0.44, 0.4, 0.34], (fbm(seed ^ 8, x * 25.0, y * 25.0, 3) - 0.45).max(0.0) * 1.2);
            c = mix(c, [0.09, 0.07, 0.05], crack * 0.85);
            let grain = (vn(seed ^ 2, x * 900.0, y * 900.0) - 0.88).max(0.0) * 9.0;
            c = mix(c, [0.82, 0.8, 0.75], grain.min(1.0));
            // the gap at its edge, dark (the edge of the map)
            let rho = ((u - 0.5).hypot(v - 0.5) * 2.0).min(1.0);
            c = mix(c, [0.08, 0.06, 0.05], sstep(0.9, 1.0, rho));
            rgb(c)
        });
    })
}

/// A leaf: dry, a papery tan-brown blotched darker and paler, its veins
/// dark; or living, a dull green with the zonal geranium's dark horseshoe
fn leaf_tex(alive: bool, salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ if alive { 0x1eaf } else { 0xd1ef };
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u - 0.5, v - 0.5);
            let r = x.hypot(y);
            let vein = (((y.atan2(x) * 7.0 / PI).fract() - 0.5).abs() * 2.0 - 0.85).max(0.0) * 6.0;
            let n = fbm(seed, u * 6.0, v * 6.0, 4);
            let c = if alive {
                let mut c = mix([0.2, 0.3, 0.13], [0.26, 0.36, 0.17], n);
                c = mix(c, [0.15, 0.17, 0.08], sstep(0.05, 0.0, (r - 0.28).abs()) * 0.6);
                mix(c, [0.36, 0.33, 0.18], sstep(0.4, 0.5, r) * 0.6)
            } else {
                let mut c = mix([0.42, 0.33, 0.22], [0.55, 0.45, 0.31], n);
                c = mix(c, [0.27, 0.2, 0.13], (n - 0.55).max(0.0) * 2.5);
                mix(c, [0.3, 0.23, 0.15], vein.min(1.0) * 0.6)
            };
            rgb(c)
        });
    })
}
