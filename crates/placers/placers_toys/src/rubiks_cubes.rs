//! `place_rubiks_cubes`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Twisty puzzle cube: 5% chance per level. A 3x3x3 puzzle cube of no
// brand (the original's 57 mm), dropped and left: its black cubies
// scuffed, its stickers faded and scratched, worn at their corners, two
// peeling at a corner and one gone; scrambled as a hand left it (the old
// sticker draws turn its faces), or still solved.
pub(crate) fn place_rubiks_cubes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let scrambled = rnd() < 0.6; // 60% chance scrambled
    let mut turns = vec![];
    if scrambled {
        // the old sticker colours' 54 draws: 27 turns of a face
        for _ in 0..54 {
            turns.push(rnd());
        }
    }
    let state = scramble(&turns);
    let (px, pz) = jitter(g, x, z, 2.4);
    let yaw = rnd() * PI * 2.0;
    let props = game::assets::props::prop_part_as_modelled;
    let cube = group();
    if let Some(body) = props("Real__twistyCubies") {
        cube.add(&mesh(body, phong!({ map: &cubie_tex(), specular: 0x2a2a2a, shininess: 28 })));
    }
    if let Some(stickers) = props("Real__twistyStickers") {
        let mat = phong!({ map: &sticker_tex(&state, salt(px, pz, 623)), specular: 0x2c2c2c, shininess: 22, side: DoubleSide });
        cube.add(&mesh(sticker_geo(&stickers), &mat));
    }
    cube.position().set(px, 0, pz);
    cube.rotation().y = yaw;
    g.world.add(&cube);
}

/// Half the cube (`party/small_everyday_things.py` `CUBE`), metres
const HALF: f64 = 0.0285;

/// The faces in the atlas, by (axis, toward +): +x, -x, +y, -y, +z, -z;
/// their stickers' colours as the puzzle comes (red opposite orange, white
/// opposite yellow, green opposite blue), faded, the white gone cream
const FACE_COLOURS: [[f64; 3]; 6] =
    [[0.66, 0.13, 0.11], [0.86, 0.42, 0.12], [0.84, 0.81, 0.70], [0.88, 0.76, 0.22], [0.14, 0.50, 0.26], [0.12, 0.24, 0.54]];

/// The face (0..6, as `FACE_COLOURS`) that shows at each facelet: by face,
/// then the face's two other axes in order (a, b in -1..=1), after the
/// turns `d` (two draws a turn: which face, a quarter, a half or three)
fn scramble(d: &[f64]) -> [[[usize; 3]; 3]; 6] {
    // each facelet: its cubie's place, the axis and side it faces, its colour
    let mut fl: Vec<([i32; 3], usize, i32, usize)> = vec![];
    for f in 0..6 {
        let (axis, sign) = (f / 2, if f % 2 == 0 { 1 } else { -1 });
        let o = others(axis);
        for a in -1..=1 {
            for b in -1..=1 {
                let mut p = [0; 3];
                p[axis] = sign;
                p[o.0] = a;
                p[o.1] = b;
                fl.push((p, axis, sign, f));
            }
        }
    }
    for t in d.chunks_exact(2) {
        let f = ((t[0] * 6.0) as usize).min(5);
        let q = 1 + ((t[1] * 3.0) as usize).min(2);
        let (axis, layer) = (f / 2, if f % 2 == 0 { 1 } else { -1 });
        let (i, j) = others(axis);
        for _ in 0..q {
            for e in fl.iter_mut() {
                if e.0[axis] != layer {
                    continue;
                }
                // a quarter turn about the axis: (i, j) -> (-j, i)
                let (pi, pj) = (e.0[i], e.0[j]);
                e.0[i] = -pj;
                e.0[j] = pi;
                if e.1 == i {
                    e.1 = j;
                } else if e.1 == j {
                    e.1 = i;
                    e.2 = -e.2;
                }
            }
        }
    }
    let mut out = [[[0; 3]; 3]; 6];
    for (p, axis, sign, col) in fl {
        let face = axis * 2 + usize::from(sign < 0);
        let o = others(axis);
        out[face][(p[o.0] + 1) as usize][(p[o.1] + 1) as usize] = col;
    }
    out
}

/// The two axes other than `axis`, in order
fn others(axis: usize) -> (usize, usize) {
    match axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    }
}

/// The stickers with their texture laid by face: each face's stickers in
/// its own cell of a 3 x 2 atlas (face f in column f % 3, row f / 3), across
/// its two other axes in order
fn sticker_geo(src: &Geo) -> Geo {
    let geo = src.duplicate();
    {
        let mut gg = geo.g();
        let n = gg.position.len() / 3;
        let mut uv = vec![0f32; n * 2];
        for i in 0..n {
            let d = [f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 1]) - HALF, f64::from(gg.position[i * 3 + 2])];
            let axis = (0..3).max_by(|&a, &b| d[a].abs().total_cmp(&d[b].abs())).unwrap_or(1);
            let face = axis * 2 + usize::from(d[axis] < 0.0);
            let o = others(axis);
            let (a, b) = ((d[o.0] / HALF + 1.0) / 2.0, (d[o.1] / HALF + 1.0) / 2.0);
            uv[i * 2] = (((face % 3) as f64 + a.clamp(0.0, 1.0)) / 3.0) as f32;
            uv[i * 2 + 1] = (((face / 3) as f64 + b.clamp(0.0, 1.0)) / 2.0) as f32;
        }
        gg.uv = uv;
        gg.touch();
    }
    geo
}

/// The stickers' atlas: each facelet its colour, faded and uneven, its
/// vinyl scratched and worn at its corners to the black under it, grime in
/// its edges; the face it lies on (-y) dirtier
fn sticker_tex(state: &[[[usize; 3]; 3]; 6], s: u32) -> Tex {
    let st = *state;
    canvas_tex_opt(384, 256, false, move |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let (fu, fv) = (u * 3.0, v * 2.0);
            let face = ((fv.floor() as usize).min(1) * 3 + (fu.floor() as usize).min(2)).min(5);
            let (a, b) = (fu.fract(), fv.fract());
            let (ia, ib) = (((a * 3.0) as usize).min(2), ((b * 3.0) as usize).min(2));
            let col = FACE_COLOURS[st[face][ia][ib]];
            // where in the sticker (0..1 each way) and how near its edge
            let (sa, sb) = ((a * 3.0).fract(), (b * 3.0).fract());
            let edge = (0.5 - (sa - 0.5).abs()).min(0.5 - (sb - 0.5).abs());
            let n = fbm(s % 977, u * 40.0, v * 26.0, 3);
            let mut c = mix(col, [col[0] * 0.78, col[1] * 0.78, col[2] * 0.76], n);
            // faded toward grey-cream by years of light
            c = mix(c, [0.62, 0.60, 0.55], 0.18);
            // the vinyl worn through at its corners and edges
            let corner = (sa - 0.5).abs().min((sb - 0.5).abs());
            let worn = sstep(0.17, 0.10, edge) * sstep(0.28, 0.40, corner + 0.15 * vn(s % 31 + 3, u * 90.0, v * 60.0));
            c = mix(c, [0.10, 0.10, 0.10], worn);
            // scratches across it
            let scr = vn(s % 53 + 11, u * 220.0 + v * 30.0, v * 9.0);
            if scr > 0.9 {
                c = mix(c, [0.80, 0.78, 0.74], (scr - 0.9) * 4.0);
            }
            // grime: in its edges, and over the face it lies on
            c = mix(c, [0.20, 0.18, 0.15], 0.3 * sstep(0.13, 0.09, edge));
            if face == 3 {
                c = mix(c, [0.25, 0.22, 0.18], 0.45);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The cubies' black plastic: scuffed grey where hands and the floor rubbed
/// it (laid by the part's own box), shared by every cube
fn cubie_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex(64, 64, |ctx, w, h| {
                fill(ctx, w, h, |u, v| {
                    let n = fbm(623, u * 8.0, v * 8.0, 3);
                    let scuff = vn(624, u * 40.0, v * 3.0);
                    let c = mix([0.13, 0.13, 0.13], [0.20, 0.20, 0.19], n);
                    let c = if scuff > 0.82 { mix(c, [0.32, 0.31, 0.30], (scuff - 0.82) * 4.0) } else { c };
                    [c[0], c[1], c[2], 1.0]
                });
            })
        })
        .clone()
    })
}

/// A salt from where a thing is (no draw)
fn salt(x: f64, z: f64, k: u32) -> u32 {
    ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
}
