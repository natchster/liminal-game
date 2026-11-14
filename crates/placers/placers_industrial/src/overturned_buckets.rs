//! `place_overturned_buckets`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::tide;
use placers_kit::noise::vn;
use placers_kit::paint::mark_mat;
use placers_kit::paint::mark_tex;
use placers_kit::paint::paint_px_indexed;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;
use placers_kit::place::skip;
use placers_kit::wet_floor::Px;
use placers_kit::wet_floor::floor_kind;

// Overturned bucket: 15% chance per level. Bucket tipped on
// its side — "someone kicked or knocked over a bucket".
pub(crate) fn place_overturned_buckets(g: &mut Game, _dist: &[i32]) {
    // A 10 l moulded polypropylene pail (family `washroom_janitor_classroom_and_plant`), kicked over: on its
    // side, resting on its rim and its foot, or upside down on its rim; its
    // wire bail fallen to the floor; scuffed, grimed at its foot, the litre
    // marks inside it and the muck dried in its bottom (docs/research/props-remodelling/washroom_janitor_classroom_and_plant.md)
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // the old colour draw: yellow, red or grey polypropylene
    let kind = ri(3).min(2);
    let bucket_yaw = rnd() * PI * 2.0;
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    // one in three lies upside down (from the yaw's draw: no new draw)
    let upside = (bucket_yaw * 977.0).fract() < 0.33;
    let salt = ((bucket_yaw * 1.0e6) as u32) ^ ((px * 911.0) as i32 as u32).rotate_left(11) ^ ((pz * 613.0) as i32 as u32).rotate_left(21);
    let col = PLASTIC[kind];
    let bucket = group();
    // each part turned as it rests, the turn baked into its own copy (a turned
    // mesh is measured by its turned box, which would lift it off the floor)
    let (turn, lift) = if upside { (PI, HEIGHT) } else { (TILT, LIFT) };
    let put = |name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            let geo = geo.duplicate();
            geo.rotate_x(turn);
            geo.translate(0, lift, 0);
            geo.compute_bounding_box();
            bucket.add(&mesh(geo, mat));
        }
    };
    put("Real__overturnedPailBody", &phong!({ map: &body_tex(col, salt), specular: 0x161616, shininess: 18 }));
    put("Real__overturnedPailInside", &phong!({ map: &inside_tex(col, salt), specular: 0x101010, shininess: 14 }));
    put("Real__overturnedPailInsideFloor", &lambert!({ map: &floor_tex(col, salt) }));
    let (bail, grip) = if upside {
        ("Real__overturnedPailBailDown", "Real__overturnedPailGripDown")
    } else {
        ("Real__overturnedPailBailSide", "Real__overturnedPailGripSide")
    };
    put(bail, &phong!({ color: 0x77736c, specular: 0x2a2a28, shininess: 30 }));
    put(grip, &phong!({ color: 0x1d1c1a, specular: 0x1a1a1a, shininess: 26 }));
    let parent = group();
    parent.add(&bucket);
    parent.rotation().y = bucket_yaw;
    parent.position().set(px, 0, pz);
    g.world.add(&parent);
    // 40% chance: what was in it, run out and dried on the floor: from its
    // mouth when it lies on its side, all round its rim when upside down
    if rnd() < 0.40 {
        let r = 0.20 + rnd() * 0.10;
        let turn = rnd() * PI * 2.0;
        let hard = floor_kind(g, x, z) != 0;
        let (rx, rz) = if upside { (r * 1.15, r * 1.05) } else { (r * 0.85, r * 1.6) };
        let mat = spill_mat(rx, rz, hard, salt ^ ((turn * 1.0e5) as u32));
        let geo = plane_geo!(rx * 2.5, rz * 2.5);
        geo.rotate_x(-PI / 2.0);
        let spill = mesh(geo, &mat);
        // its long axis along the pail's, its near edge under the mouth
        spill.rotation().y = bucket_yaw;
        let reach = if upside { 0.0 } else { 0.17 + rz * 0.75 };
        let (fx, fz) = (bucket_yaw.dsin(), bucket_yaw.dcos());
        let pp = parent.pos();
        spill.position().set(pp.x + fx * reach, 0.004, pp.z + fz * reach);
        g.world.add(&spill);
    }
}

/// Lying on its side: its turn about x and lift, so the flange and the
/// foot touch the floor (printed by `industrial/washroom_janitor_classroom_and_plant.py`)
const TILT: f64 = 1.41368;

const LIFT: f64 = 0.10265;

/// To the top of its rim
const HEIGHT: f64 = 0.230;

/// Polypropylene gone dull: yellow, oxide red, grey (sRGB 0..1)
const PLASTIC: [[f64; 3]; 3] = [[0.78, 0.63, 0.24], [0.56, 0.22, 0.16], [0.44, 0.44, 0.42]];

/// fbm seamless round the pail (u in 0..1 is once round)
fn wrap_fbm(seed: u32, u: f64, y: f64, fu: f64, fy: f64, oct: u32) -> f64 {
    fbm(seed, u * fu, y * fy, oct) * (1.0 - u) + fbm(seed, (u - 1.0) * fu, y * fy, oct) * u
}

fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 3] + Sync) {
    let (iw, _ih) = (w as usize, h as usize);
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, iw * 4, |j, out| {
        for i in 0..iw {
            let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
            for k in 0..3 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            out[i * 4 + 3] = 255;
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

/// Scratches and scuffs: short strokes, paler (stress-whitened) or dark,
/// most where it was dragged (the foot) and carried (the rim)
fn scuffs(ctx: &mut Canvas, w: f64, h: f64, col: [f64; 3], n: usize) {
    let pale = format!(
        "rgba({},{},{},",
        (col[0] * 255.0 + 50.0).min(255.0) as u8,
        (col[1] * 255.0 + 50.0).min(255.0) as u8,
        (col[2] * 255.0 + 45.0).min(255.0) as u8
    );
    for k in 0..n {
        let x = rnd() * w;
        let y = if rnd() < 0.6 { h * (1.0 - rnd() * rnd() * 0.5) } else { h * rnd() * rnd() * 0.4 };
        let a = (rnd() - 0.5) * 1.2 + if rnd() < 0.5 { 0.0 } else { PI / 2.0 };
        let l = 3.0 + rnd() * 18.0;
        if k % 3 == 0 {
            ctx.set_stroke_style("rgba(40,34,28,0.35)");
        } else {
            ctx.set_stroke_style(format!("{pale}{:.2})", 0.25 + 0.35 * rnd()).as_str());
        }
        ctx.set_line_width(0.6 + rnd() * 1.2);
        ctx.begin_path();
        ctx.move_to(x, y);
        ctx.line_to(x + a.dcos() * l, y + a.dsin() * l * 0.35);
        ctx.stroke();
    }
}

/// Its outside (round it, up it): the plastic mottled where the sun and
/// the cleaning left it, grime up from the foot, streaks of dried
/// limescale run down from the rim, scuffs
fn body_tex(col: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(512, 128, false, move |ctx, w, h| {
        skip(salt);
        let seed = salt ^ 0x9a11;
        fill(ctx, w, h, |u, v| {
            let y = v * HEIGHT;
            let mot = wrap_fbm(seed, u, y, 40.0, 30.0, 3);
            let mut c = mix(col, [col[0] * 0.86, col[1] * 0.84, col[2] * 0.82], (mot - 0.4) * 2.0);
            // the dirt film everywhere, thicker toward the foot
            let film = 0.18 + 0.25 * wrap_fbm(seed ^ 3, u, y, 14.0, 9.0, 3);
            c = mix(c, [0.33, 0.30, 0.25], film * 0.45);
            let foot = sstep(0.055, 0.0, y) * (0.55 + 0.45 * wrap_fbm(seed ^ 5, u, y, 60.0, 80.0, 2));
            c = mix(c, [0.2, 0.18, 0.15], foot * 0.8);
            // dried runs down from the rim: pale, uneven, fading out
            let run = (wrap_fbm(seed ^ 7, u, 0.0, 140.0, 1.0, 2) - 0.62).max(0.0) * 4.0;
            let reach = 0.05 + 0.12 * wrap_fbm(seed ^ 9, u, 0.0, 40.0, 1.0, 1);
            let runs = run * sstep(HEIGHT - reach, HEIGHT - 0.01, y) * (0.7 + 0.3 * vn(seed ^ 11, u * 300.0, y * 40.0));
            c = mix(c, [0.74, 0.72, 0.66], runs * 0.5);
            c
        });
        scuffs(ctx, w, h, col, 140);
    })
}

/// Its inside wall: darker, the tide line where something stood in it
/// and dried, the muck below it thicker toward the bottom
fn inside_tex(col: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(512, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x1de;
        let tide_y = 0.07 + 0.08 * hash(seed, 1, 2);
        fill(ctx, w, h, |u, v| {
            let y = v * HEIGHT;
            let mut c = [col[0] * 0.82, col[1] * 0.8, col[2] * 0.78];
            c = mix(c, [0.3, 0.28, 0.24], 0.2 + 0.2 * wrap_fbm(seed, u, y, 20.0, 15.0, 3));
            let wob = 0.004 * (wrap_fbm(seed ^ 2, u, 0.0, 30.0, 1.0, 2) - 0.5);
            let d = y - (tide_y + wob);
            // under the line: the film it left, browner and thicker down to the foot
            let under = sstep(0.002, -0.004, d);
            let thick = under * (0.45 + 0.35 * sstep(tide_y, 0.0, y)) * (0.75 + 0.25 * wrap_fbm(seed ^ 4, u, y, 90.0, 60.0, 2));
            c = mix(c, [0.27, 0.23, 0.17], thick);
            // the line itself: a dark crust, a paler band just above it
            c = mix(c, [0.17, 0.14, 0.1], (-(d / 0.0014).dpowi(2)).dexp() * 0.7);
            c = mix(c, [0.6, 0.57, 0.5], (-((d - 0.003) / 0.0018).dpowi(2)).dexp() * 0.25);
            c
        });
    })
}

/// Its bottom, seen from above: what was in it dried to a crust and
/// cracked into flakes, curled at their edges
fn floor_tex(col: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0xf100;
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 0.21, v * 0.21);
            // cracks: where the two nearest of a jittered lattice are near equal
            let (cx, cy) = (x / 0.011, y / 0.011);
            let (mut d1, mut d2) = (9.0f64, 9.0f64);
            for dj in -1..=1 {
                for di in -1..=1 {
                    let (ii, jj) = (cx.floor() as i32 + di, cy.floor() as i32 + dj);
                    let fx = ii as f64 + hash(seed, ii, jj);
                    let fy = jj as f64 + hash(seed ^ 77, ii, jj);
                    let d = (fx - cx).dhypot(fy - cy);
                    if d < d1 {
                        d2 = d1;
                        d1 = d;
                    } else if d < d2 {
                        d2 = d;
                    }
                }
            }
            // the crust lies thick in the middle and in patches, thin to
            // the plastic at the edge
            let rho = ((u - 0.5).dhypot(v - 0.5) * 2.0).min(1.0);
            let cover = sstep(0.38, 0.55, fbm(seed ^ 2, x * 25.0, y * 25.0, 3) + 0.25 * (1.0 - rho));
            let crack = sstep(0.06, 0.0, d2 - d1) * cover;
            let mut c = [col[0] * 0.55, col[1] * 0.55, col[2] * 0.55];
            c = mix(c, [0.25, 0.22, 0.18], 0.6);
            let crust = mix([0.27, 0.24, 0.19], [0.34, 0.31, 0.25], fbm(seed ^ 1, x * 80.0, y * 80.0, 3));
            c = mix(c, crust, cover);
            // the flakes' edges curled up and catching the light
            c = mix(c, [0.4, 0.37, 0.31], sstep(0.16, 0.05, d2 - d1) * cover * 0.3);
            c = mix(c, [0.12, 0.1, 0.08], crack * 0.8);
            c
        });
    })
}

/// What ran out of it, dried on the floor: on a hard floor a grey film
/// with a dark tide line at its edge and silt settled in lobes; on carpet
/// a dark patch soaked in, its rim darker
fn spill_mat(rx: f64, rz: f64, hard: bool, salt: u32) -> Mat {
    let (pw, ph) = (rx * 2.5, rz * 2.5);
    let (tw, th) = (160.0, (160.0 * rz / rx).round().clamp(80.0, 256.0));
    let bleed = if hard { [0.36, 0.35, 0.33] } else { [0.16, 0.13, 0.1] };
    let tex = mark_tex(tw, th, bleed, move |ctx, cw, ch| {
        let seed = salt ^ 0x5b11;
        let (sx, sy) = (pw / cw, ph / ch);
        let aa = sx.max(sy) * 1.2;
        let r0 = rx.min(rz);
        paint_px_indexed(ctx, cw, ch, |i, j| {
            let (x, y) = ((i as f64 + 0.5) * sx - pw / 2.0, (j as f64 + 0.5) * sy - ph / 2.0);
            let (u, v) = (x / rx, y / rz);
            let rho = u.dhypot(v);
            let (cu, cv) = (u / rho.max(1e-6), v / rho.max(1e-6));
            let lobe = 1.0
                + 0.5 * (fbm(seed, cu * 1.3 + 4.0, cv * 1.3 + 4.0, 3) - 0.5)
                + 0.14 * (vn(seed ^ 9, cu * 5.0 + 2.0, cv * 5.0 + 2.0) - 0.5);
            let d = (rho - 0.85 * lobe) * r0;
            let mut px = Px::CLEAR;
            if hard {
                let n = fbm(seed ^ 3, x * 40.0, y * 40.0, 3);
                if d < 0.0 {
                    px.lay([0.4, 0.39, 0.36], (0.16 + 0.14 * n) * sstep(0.0, -0.01, d));
                }
                px.lay([0.22, 0.2, 0.17], tide(d, aa, 0.006) * (0.55 + 0.3 * n));
                // silt dried in the low spots
                let silt = (fbm(seed ^ 5, x * 18.0, y * 18.0, 3) - 0.55).max(0.0) * 3.0 * sstep(0.0, -0.03, d);
                px.lay([0.3, 0.27, 0.22], silt * 0.5);
            } else {
                let soak = sstep(0.02, -0.02, d) * (0.5 + 0.3 * fbm(seed ^ 3, x * 25.0, y * 25.0, 3));
                px.lay([0.12, 0.1, 0.08], soak * 0.6);
                px.lay([0.09, 0.07, 0.05], tide(d, aa * 3.0, 0.02) * 0.45);
            }
            px.rgba()
        });
    });
    mark_mat(&tex)
}
