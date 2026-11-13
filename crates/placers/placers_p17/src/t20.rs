//! Batch `t20`'s chess table, made real (`docs/design/PROPS.md` P3; the Blender
//! batch `assets/src/blender/props/t20.py`, its notes `t20.notes.md`): a
//! Victorian tripod table with a game of Staunton chessmen left on its
//! folding board, a brandy snifter (batch `t10`'s glass) and a cigar gone
//! out on a brass ashtray beside it. It keeps the old placer's draws, call
//! for call, and its place (`chess_set_positions`, the clicks of a move)
//! as the old one. The old cigar's tip glowed: this one has gone out, its
//! ash grey, as a cigar left on a rest does in minutes.
use super::*;
use t20_kit::*;

/// The table (t20.py `CH_*`): its top's face, the board's face, a square
const CH_TOP_Y: f64 = 0.700;
const CH_BOARD_Y: f64 = 0.714;
const CH_SQ: f64 = 0.035;
/// The snifter's bowl inside (t10.py `SN_*`): its widest (y), its radius
/// there, below it its half height; the low pour's surface
const SN_YC: f64 = 0.058;
const SN_A_IN: f64 = 0.0365 - 0.0012;
const SN_B_IN: f64 = 0.034;
const SN_LOW: f64 = 0.040;

// Chess set mid-game on small side stand (floor): 3% chance per level.
// A tripod table of about 1870 (Blender batch `t20`): a moulded round top
// on its bearers, a turned column, three cabriole legs on pad feet; a
// folding board of 35 mm squares on it, and a game of Staunton chessmen
// (boxwood and ebony, a 64 mm king) left on it, the taken ones lying beside
// it; now and then a brandy snifter with a little brandy left in it, a
// small cast brass ashtray with a cigar gone out on its rest. The mahogany
// ringed by wet glasses and dusty, the board's squares worn at the edge
// the players sat at.
pub(super) fn place_chess_side_stands(g: &mut Game, _dist: &[i32]) {
    g.pp.chess_set_positions.clear(); // reset per level
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old draws, call for call: the wood
    let wood_colors = [0x604020, 0x2a1810, 0x8b6f47, 0x502010];
    let wood = choice(&wood_colors);
    // the pieces on the board: each one's square (drawn again while taken,
    // up to 30 times), its side, its kind and its turn
    let piece_count = 8 + (rnd() * 5.0).floor() as i32;
    let piece_types = ["pawn", "pawn", "pawn", "rook", "knight", "bishop", "queen", "king"];
    let mut used: Vec<(i32, i32)> = vec![];
    let mut pieces: Vec<(i32, i32, bool, &str, f64)> = vec![];
    for _ in 0..piece_count {
        let (mut ci, mut cj) = (0i32, 0i32);
        let mut attempts = 0;
        loop {
            ci = (rnd() * 8.0).floor() as i32;
            cj = (rnd() * 8.0).floor() as i32;
            attempts += 1;
            if !(used.contains(&(ci, cj)) && attempts < 30) {
                break;
            }
        }
        if attempts >= 30 {
            continue;
        }
        used.push((ci, cj));
        let white = rnd() < 0.5;
        let ty = piece_types[(rnd() * piece_types.len() as f64).floor() as usize];
        let turn = rnd() * PI * 2.0;
        pieces.push((ci, cj, white, ty, turn));
    }
    // the taken pawns beside the board, each one's side
    let captured_count = 2 + (rnd() * 3.0).floor() as i32;
    let captured: Vec<bool> = (0..captured_count).map(|_| rnd() < 0.5).collect();
    let snifter = rnd() < 0.3;
    let ashtray = rnd() < 0.3;
    let px = g.wx(x) + (rnd() - 0.5) * 1.6;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.6;
    let turn = rnd() * PI * 2.0;
    let cs = group();
    cs.position().set(px, 0, pz);
    cs.rotation().y = turn;
    let vary = vary(px, pz, turn);
    // the table
    let stain = ch_stain(wood);
    put(&cs, "Real__chessTableTop", &phong!({ map: &ch_top_tex(vary, stain), specular: 0x1a1814, shininess: 18 }));
    put(&cs, "Real__chessTableBase", &phong!({ map: &ch_wood_tex(vary, stain), specular: 0x161412, shininess: 14 }));
    put(&cs, "Real__chessBoard", &phong!({ map: &ch_board_tex(vary), specular: 0x1a1814, shininess: 20 }));
    put(&cs, "Real__chessBoardBody", &phong!({ map: &ch_wood_tex(vary ^ 0xb0d, [0.24, 0.15, 0.09]), specular: 0x161412, shininess: 16 }));
    put(&cs, "Real__chessBoardHinges", &phong!({ color: 0x6a5631, specular: 0x4a3c22, shininess: 30 }));
    // the chessmen: boxwood gone honey-yellow, ebony
    let boxwood = phong!({ color: 0xb08a58, specular: 0x2a241a, shininess: 26 });
    let ebony = phong!({ color: 0x1c1712, specular: 0x34302a, shininess: 40 });
    for &(ci, cj, white, ty, rot) in &pieces {
        let p = group();
        p.position().set(-4.0 * CH_SQ + (ci as f64 + 0.5) * CH_SQ, CH_BOARD_Y, -4.0 * CH_SQ + (cj as f64 + 0.5) * CH_SQ);
        p.rotation().y = rot;
        put(&p, ch_part(ty), if white { &boxwood } else { &ebony });
        cs.add(&p);
    }
    // the taken pawns lying on their sides in a row along the board's +x edge,
    // each resting on its foot's rim and its ball
    for (i, &white) in captured.iter().enumerate() {
        let p = group();
        p.position().set(0.150 + 0.016, CH_TOP_Y + 0.0108, -0.070 + i as f64 * 0.036);
        p.rotation().x = PI / 2.0 + 0.18;
        put(&p, "Real__chessPawn", if white { &boxwood } else { &ebony });
        cs.add(&p);
    }
    // a brandy snifter beside the board (batch `t10`'s), a little brandy left
    if snifter {
        let glass = group();
        glass.position().set(-0.150 - 0.047, CH_TOP_Y, 0.050);
        cs.add(&glass);
        let brandy = phong!({ color: 0x5a2a0e, specular: 0x3a3226, shininess: 80, transparent: true, opacity: 0.82, depthWrite: false });
        if let Some(b) = put(&glass, "Real__snifterBrandyLow", &brandy) {
            b.set_render_order(1);
        }
        let surf = phong!({ color: 0x5a2a0e, specular: 0x6a5a44, shininess: 120, transparent: true, opacity: 0.88, depthWrite: false });
        let k = (SN_LOW - SN_YC) / SN_B_IN;
        let surface = mesh(circle_geo!(SN_A_IN * (1.0 - k * k).max(0.0).sqrt() - 0.0002, 32), &surf);
        surface.rotation().x = -PI / 2.0;
        surface.position().y = SN_LOW;
        surface.set_render_order(1);
        glass.add(&surface);
        if let Some(m) = put(&glass, "Real__snifterBowl", &ch_glass_mat(0.55, 0.32)) {
            m.set_render_order(2);
        }
        if let Some(m) = put(&glass, "Real__snifterFoot", &ch_glass_mat(0.9, 0.7)) {
            m.set_render_order(2);
        }
    }
    // a cast brass ashtray, a cigar gone out on its rest
    if ashtray {
        let tray = group();
        tray.position().set(-0.150 - 0.047, CH_TOP_Y, -0.062);
        tray.rotation().y = PI / 2.0;
        cs.add(&tray);
        put(&tray, "Real__chessAshtray", &phong!({ color: 0x5e4a2a, specular: 0x3a3020, shininess: 24 }));
        // the cigar across the dish, its ash on the dish's floor, its body in
        // the rest at +x and out over the table's edge: along x, its foot to -x
        let cigar = group();
        cigar.position().set(0.0498, 0.0182, 0);
        cigar.rotation().z = -PI / 2.0 + 0.0877;
        tray.add(&cigar);
        put(&cigar, "Real__humidorCigar", &lambert!({ map: &ch_leaf_tex(vary) }));
        put(&cigar, "Real__humidorCigarAsh", &lambert!({ color: 0x7c7870 }));
    }
    g.world.add(&cs);
    let p = cs.pos();
    g.pp.chess_set_positions.push(v3(p.x, 0, p.z));
}

fn ch_part(ty: &str) -> &'static str {
    match ty {
        "rook" => "Real__chessRook",
        "knight" => "Real__chessKnight",
        "bishop" => "Real__chessBishop",
        "queen" => "Real__chessQueen",
        "king" => "Real__chessKing",
        _ => "Real__chessPawn",
    }
}

/// The old wood colours' stains (sRGB): mahogany, rosewood, walnut, red mahogany
fn ch_stain(c: u32) -> [f64; 3] {
    match c {
        0x604020 => [0.30, 0.15, 0.08],
        0x2a1810 => [0.13, 0.075, 0.05],
        0x8b6f47 => [0.36, 0.25, 0.15],
        _ => [0.26, 0.11, 0.06],
    }
}

/// The column and legs (the grain up them, v): stained, dull, rubbed on
/// the knees, dust
fn ch_wood_tex(vary: u32, stain: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0xc4e;
        fill(ctx, w, h, |u, v| {
            let mut o = wood(seed, v, u, stain, 6.0, 40.0);
            let rub = ss(0.62, 0.8, fbm(seed ^ 3, u * 7.0, v * 22.0, 3));
            o = mix(o, stain.map(|c| (c * 1.6 + 0.06).min(0.6)), rub * 0.3);
            let dust = 0.08 + 0.12 * fbm(seed ^ 5, u * 6.0, v * 12.0, 3);
            opaque(mix(o, [0.42, 0.39, 0.34], dust))
        });
    })
}

/// The top's face as it is mapped (u across, v from front to back): its
/// figure, the white rings of wet glasses, a scorch, scratches, dust thick
/// where nothing stood
fn ch_top_tex(vary: u32, stain: [f64; 3]) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(vary, 0x70b);
        let seed = (r.next() * 4.0e9) as u32;
        let rings: Vec<(f64, f64, f64)> = (0..4).map(|_| (r.range(0.12, 0.88), r.range(0.12, 0.88), r.range(0.06, 0.08))).collect();
        fill(ctx, w, h, |u, v| {
            let mut o = wood(seed, u, v, stain, 10.0, 34.0);
            for &(cx, cy, cr) in &rings {
                let d = (((u - cx).powi(2) + (v - cy).powi(2)).sqrt() - cr).abs();
                o = mix(o, [0.50, 0.45, 0.38], (1.0 - ss(0.0, 0.006, d)) * 0.18);
            }
            let scratch = ss(0.93, 0.97, vn(seed ^ 3, u * 3.0 + v * 40.0, v * 3.0 - u * 2.0));
            o = mix(o, [0.45, 0.33, 0.22], scratch * 0.5);
            let dust = 0.14 + 0.16 * fbm(seed ^ 5, u * 6.0, v * 6.0, 3);
            opaque(mix(o, [0.45, 0.42, 0.37], dust))
        });
    })
}

/// The folding board's face as it is mapped (u across, v from front to
/// back): squares of boxwood and rosewood veneer inlaid in a walnut border
/// with a stringing line, the fold down its middle; the lacquer worn grey
/// at the two edges the players sat at, chipped at the corners, dusty
fn ch_board_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0xb0a;
        fill(ctx, w, h, |u, v| {
            let (x, z) = ((u - 0.5) * 0.300, (0.5 - v) * 0.300);
            let inside = x.abs() < 4.0 * CH_SQ && z.abs() < 4.0 * CH_SQ;
            let mut o = if inside {
                let (i, j) = (((x / CH_SQ) + 4.0).floor() as i32, ((z / CH_SQ) + 4.0).floor() as i32);
                let light = (i + j) % 2 == 0;
                let s2 = seed ^ (i * 8 + j) as u32;
                // each square's veneer laid its own way
                let (a, b) = if (i + j) % 4 < 2 { (u, v) } else { (v, u) };
                if light {
                    wood(s2, a, b, [0.66, 0.52, 0.32], 8.0, 40.0)
                } else {
                    wood(s2, a, b, [0.20, 0.10, 0.06], 8.0, 40.0)
                }
            } else {
                wood(seed ^ 7, u, v, [0.30, 0.19, 0.11], 6.0, 30.0)
            };
            // the stringing round the squares
            let edge = (x.abs().max(z.abs()) - 4.0 * CH_SQ).abs();
            if edge < 0.0012 {
                o = [0.70, 0.62, 0.45];
            }
            // worn where the players' hands and sleeves went: the near and far edges
            let wear = ss(0.10, 0.15, z.abs()) * ss(0.6, 0.85, fbm(seed ^ 9, u * 30.0, v * 6.0, 3));
            o = mix(o, [0.46, 0.40, 0.32], wear * 0.45);
            let dust = 0.08 + 0.12 * fbm(seed ^ 11, u * 7.0, v * 7.0, 3);
            opaque(mix(o, [0.45, 0.42, 0.37], dust))
        });
    })
}

/// A cigar's wrapper leaf as the cigar is mapped (u round it, v along it):
/// its veins running across, the leaf gone dry and dull, the head darker
/// where it was held in the mouth
fn ch_leaf_tex(vary: u32) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x1ea;
        fill(ctx, w, h, |u, v| {
            let vein = ss(0.9, 0.98, (u * 9.0 + v * 22.0 + fbm(seed, u * 4.0, v * 8.0, 2) * 2.0).sin() * 0.5 + 0.5) * 0.25;
            let n = fbm(seed ^ 3, u * 8.0, v * 24.0, 3);
            let mut o = [0.30, 0.19, 0.11].map(|c| c * (0.8 + 0.35 * n) * (1.0 - vein));
            o = o.map(|c| c * (1.0 - 0.3 * ss(0.85, 0.98, v)));
            opaque(mix(o, [0.40, 0.36, 0.30], 0.12))
        });
    })
}

/// Clear glass as the windows' (`Patch::Glass`): the room seen in it by
/// Fresnel (`room`: how much), its dust (`dust` of the map's grey)
fn ch_glass_mat(dust: f32, room: f32) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 260, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(ch_dust_tex());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "chess_snifter_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = room;
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        f[8] = dust;
        f[9] = 0.20;
        f[10] = 0.19;
        f[11] = 0.17;
        f[12] = 0.0;
        f[13] = 0.07;
    }
    m
}

/// Dust and fingermarks on the glass (grey: how much)
fn ch_dust_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                fill(ctx, w, h, |u, v| {
                    let d = fbm(181, u * 7.0, v * 7.0, 3) * 0.6 + fbm(182, u * 29.0, v * 29.0, 2) * 0.4;
                    let smear = ss(0.62, 0.8, fbm(183, u * 9.0, v * 3.0, 3)) * (1.0 - ss(0.15, 0.4, (v - 0.45).abs()));
                    let k = 0.2 + 0.4 * d + 0.5 * smear + 0.2 * v;
                    [k, k, k, 1.0]
                });
            })
        })
        .clone()
    })
}

/// Batch `t20`'s painting kit (each of its modules has its own copy): the
/// parts placed as modelled, a value of where a thing was put to vary its
/// painters, value noise, and a painter that fills a canvas pixel by pixel
/// on every core (no canvas call and no draw inside the loop)
#[allow(dead_code)]
mod t20_kit {
    use game::assets::props::prop_part_as_modelled;
    use game::prelude::*;

    /// The Blender part `name`, as modelled, in `grp` with `mat`
    pub fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let m = mesh(prop_part_as_modelled(name)?, mat);
        grp.add(&m);
        Some(m)
    }

    /// Something of where a thing was put, to vary what its painters paint
    /// (a painter's own stream starts the same on every floor)
    pub fn vary(x: f64, z: f64, k: f64) -> u32 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for v in [x, z, k] {
            h ^= v.to_bits();
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        (h ^ (h >> 29) ^ (h >> 47)) as u32
    }

    pub fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }

    pub fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }

    pub fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }

    pub fn ss(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    pub fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }

    pub fn opaque(c: [f64; 3]) -> [f64; 4] {
        [c[0], c[1], c[2], 1.0]
    }

    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right,
    /// v up, both 0..1) gives straight sRGB and alpha, 0..1
    pub fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
        let iw = w as usize;
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

    /// The distance to the nearest edge of a cell of a Voronoi field (cells
    /// of size 1): cracks in old leather, varnish or enamel
    pub fn crack(seed: u32, x: f64, y: f64) -> f64 {
        let (cx, cy) = (x.floor() as i32, y.floor() as i32);
        let (mut d1, mut d2) = (9.0f64, 9.0f64);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (ix, iy) = (cx + dx, cy + dy);
                let px = f64::from(ix) + 0.1 + 0.8 * hash(seed, ix, iy);
                let py = f64::from(iy) + 0.1 + 0.8 * hash(seed ^ 0x55, ix, iy);
                let d = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
                if d < d1 {
                    d2 = d1;
                    d1 = d;
                } else if d < d2 {
                    d2 = d;
                }
            }
        }
        (d2 - d1) * 0.5
    }

    /// Wood seen along its grain (u along it): the growth rings' streaks and
    /// the figure, `base` its colour (sRGB)
    pub fn wood(seed: u32, u: f64, v: f64, base: [f64; 3], along: f64, across: f64) -> [f64; 3] {
        let warp = fbm(seed ^ 0x1d, u * along * 0.15, v * across * 0.05, 3);
        let ring = ((v * across + warp * 6.0).sin() * 0.5 + 0.5).powf(3.0);
        let streak = fbm(seed, u * along * 0.2, v * across, 3);
        let fine = vn(seed ^ 3, u * along * 3.0, v * across * 6.0);
        base.map(|c| c * (0.82 + 0.28 * streak + 0.06 * fine - 0.14 * ring))
    }

    /// Tarnished metal's mottle (a multiplier near 1)
    pub fn tarnish(seed: u32, u: f64, v: f64, scale: f64) -> f64 {
        0.8 + 0.3 * fbm(seed, u * scale, v * scale, 3) - 0.18 * ss(0.6, 0.8, fbm(seed ^ 9, u * scale * 0.6, v * scale * 0.6, 3))
    }
}
