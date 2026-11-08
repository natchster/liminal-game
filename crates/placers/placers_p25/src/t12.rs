//! Batch `t12`'s props in this crate, made real (`docs/PROPS.md` P3, "the
//! party and the shrine"): what a party room leaves, years on, in cheap
//! paper, card, foil and crepe, faded, crumpled and dusty
//! (`assets/src/blender/props/t12.py`, `t12.notes.md`): the party crown,
//! the crepe streamers, the tissue tassel garland, the star piñata. Their
//! placers and what only they use; each keeps the old placer's draws, call
//! for call.
use crate::{open_tiles, pick};
use game::prelude::*;
use kit::*;

// ------------------------------------------------------------- the crown
/// The crown's card (`t12.py` CR_*): its length round (fastened) and its
/// tab, the band and the points, how many, where a bent point folds
const CR_C: f64 = 0.530;
const CR_OV: f64 = 0.045;
const CR_HB: f64 = 0.030;
const CR_HP: f64 = 0.059;
const CR_NP: usize = 8;
const CR_VF: f64 = CR_HB + 0.004;
/// the fastened ring pressed oval (half axes along x and z); the open one's
/// curl
const CR_AX: f64 = 0.094;
const CR_AZ: f64 = 0.075;
const CR_OPEN_R: f64 = 0.130;
/// The points bent over in each (the print cracks at their folds), and the
/// ring's crease across its band (u along the card)
const CR_FOLDED: [&[usize]; 2] = [&[1, 4, 6], &[2, 5]];
const CR_KINK: f64 = 0.205;
/// The print's share of the texture; above it the grey back of the card
const PRINT_V: f64 = 0.92;

/// Small paper crown: 8% chance per level. A party crown die-cut from
/// foil card or printed card (Blender batch `t12`): 89 mm tall, eight points
/// over a 30 mm band, a printed jewel in each point; fastened round by its
/// tab and trodden (pressed oval, creased, three points bent over), or come
/// unfastened and sprung open in a curl. The foil dulled, scuffed through
/// to the card, cracked white where it bent; dust, grime at its foot.
pub fn place_small_paper_crowns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour: the crown's foil or card
    let crown_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let cc = pick(&crown_colors);
    let design = crown_colors.iter().position(|&c| c == cc).unwrap_or(0);
    let crown = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    crown.position().set(px, 0, pz);
    crown.rotation().y = rnd() * PI * 2.0;
    let salt = salt_of(px, pz);
    let kind = usize::from(salt % 5 < 2);
    let tex = crown_print_tex(design, kind, salt);
    // foil (rose, gold, pink) dark in its diffuse and coloured in the
    // highlight; or matt printed card
    let mat = match design {
        0 => phong!({ map: &tex, specular: 0x8a5a62, shininess: 30 }),
        2 => phong!({ map: &tex, specular: 0x9a8650, shininess: 30 }),
        5 => phong!({ map: &tex, specular: 0x8a6a7c, shininess: 30 }),
        _ => lambert!({ map: &tex }),
    };
    put_with(&crown, ["Real__paperCrown", "Real__paperCrownOpen"][kind], &mat, |gg| {
        let n = gg.position.len() / 3;
        let length = CR_C + CR_OV;
        for i in 0..n {
            let (x, y, z) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 1]), f64::from(gg.position[i * 3 + 2]));
            let (nx, nz) = (f64::from(gg.normal[i * 3]), f64::from(gg.normal[i * 3 + 2]));
            // where it is along the card, and the card's own normal there
            let (u, ex, ez, sx, sz) = if kind == 0 {
                let th = (x / CR_AX).atan2(z / CR_AZ).rem_euclid(2.0 * PI);
                let (ex, ez) = (CR_AX * th.sin(), CR_AZ * th.cos());
                let (a, b) = (CR_AZ * th.sin(), CR_AX * th.cos());
                let l = a.hypot(b);
                (th / (2.0 * PI) * CR_C, ex, ez, a / l, b / l)
            } else {
                let span = length / CR_OPEN_R;
                let th0 = PI + (2.0 * PI - span) / 2.0;
                let th = x.atan2(z);
                let u = ((th - th0).rem_euclid(2.0 * PI) * CR_OPEN_R).min(length);
                let r = CR_OPEN_R * (1.0 + 0.10 * (2.0 * u / length - 1.0).abs().powi(3));
                (u, r * th.sin(), r * th.cos(), th.sin(), th.cos())
            };
            let off = (x - ex) * sx + (z - ez) * sz;
            // a bent point: how far up the card it was, unfolded
            let v = if y > CR_VF { CR_VF + (y - CR_VF).hypot(off) } else { y };
            let front = nx * sx + nz * sz > 0.0;
            gg.uv[i * 2] = (u / length) as f32;
            gg.uv[i * 2 + 1] = if front { (v / (CR_HB + CR_HP) * PRINT_V).min(PRINT_V - 0.002) } else { 0.97 } as f32;
        }
    });
    g.world.add(&crown);
}

/// The crown's print along its card (u along it, v up) and its wear; the
/// back of the card above `PRINT_V`
fn crown_print_tex(design: usize, kind: usize, salt: u32) -> Tex {
    canvas_tex_opt(1024, 192, false, move |ctx, w, h| {
        let length = CR_C + CR_OV;
        let pitch = (CR_C - CR_OV) / CR_NP as f64;
        let foil = matches!(design, 0 | 2 | 5);
        // ground, its printed edging, the jewels' inks
        let (ground, edge) = match design {
            0 => ([0.50, 0.24, 0.27], [0.62, 0.52, 0.30]),
            1 => ([0.30, 0.36, 0.52], [0.80, 0.68, 0.36]),
            2 => ([0.56, 0.45, 0.21], [0.36, 0.20, 0.16]),
            3 => ([0.33, 0.47, 0.32], [0.80, 0.68, 0.36]),
            4 => ([0.42, 0.29, 0.47], [0.80, 0.68, 0.36]),
            _ => ([0.58, 0.40, 0.48], [0.62, 0.62, 0.64]),
        };
        let jewels = [[0.55, 0.14, 0.16], [0.14, 0.42, 0.26], [0.16, 0.24, 0.52]];
        let card = [0.82, 0.80, 0.74];
        let folded = CR_FOLDED[kind];
        fill(ctx, w, h, |tu, tv| {
            if tv > PRINT_V {
                let n = fbm(salt ^ 0x51, tu * 40.0, tv * 8.0, 3);
                let c = mix([0.66, 0.64, 0.59], [0.55, 0.53, 0.48], n);
                return [c[0], c[1], c[2], 1.0];
            }
            let u = tu * length;
            let v = tv / PRINT_V * (CR_HB + CR_HP);
            let mut c = ground;
            // the card's outline here: its top edge
            let in_points = u > CR_OV && u < CR_C;
            let f = ((u - CR_OV) / pitch).rem_euclid(1.0);
            let top = if in_points { CR_HB + CR_HP * (1.0 - (2.0 * f - 1.0).abs()) } else { CR_HB };
            // printed edging: along the band's foot and top, and up each
            // point's sides
            let line = |d: f64, wd: f64| 1.0 - sstep(wd * 0.6, wd, d.abs());
            let mut e = line(v - 0.004, 0.0014).max(line(v - (CR_HB - 0.003), 0.0012));
            if in_points && v > CR_HB - 0.003 {
                let side = (top - v) * (pitch / 2.0) / CR_HP;
                e = e.max(line(side - 0.0035, 0.0012));
            }
            // a row of dots along the band
            let dk = (u / 0.012).floor();
            let dd = (u - (dk + 0.5) * 0.012).hypot(v - CR_HB * 0.5);
            e = e.max(1.0 - sstep(0.0016, 0.0022, dd));
            c = mix(c, edge, e);
            // a jewel in each point, a pearl at its tip
            if in_points {
                let k = ((u - CR_OV) / pitch).floor() as usize;
                let cu = CR_OV + (k as f64 + 0.5) * pitch;
                let (du, dv) = ((u - cu) / 0.0075, (v - (CR_HB + 0.014)) / 0.0105);
                let r = du.hypot(dv);
                if r < 1.25 {
                    let jw = jewels[k % 3];
                    let rim = 1.0 - sstep(1.0, 1.25, r);
                    c = mix(c, edge, rim);
                    let inner = 1.0 - sstep(0.85, 1.0, r);
                    // its facets printed: lighter up and left
                    let lit = (0.5 - 0.35 * du + 0.45 * dv).clamp(0.0, 1.0);
                    let jc = mix(jw, [0.85, 0.82, 0.78], 0.35 * lit * lit);
                    c = mix(c, jc, inner);
                }
                let pd = (u - cu).hypot(v - (CR_HB + CR_HP - 0.008));
                c = mix(c, [0.80, 0.78, 0.72], 1.0 - sstep(0.0026, 0.0034, pd));
            }
            // wear: foil dulled in patches, scuffed through to the card
            c = mix(c, [0.50, 0.48, 0.43], (0.08 + 0.14 * fbm(salt, u * 40.0, v * 40.0, 3)) * if foil { 1.0 } else { 0.6 });
            let sc = vn(salt ^ 3, u * 1100.0, v * 1100.0) * 0.55 + fbm(salt ^ 4, u * 140.0, v * 260.0, 2) * 0.45;
            if sc > 0.71 {
                c = mix(c, card, ((sc - 0.71) * 6.0).min(0.9));
            }
            // cracked white where it folded: a bent point's root, the crease
            let mut crack = 0.0;
            if in_points && folded.contains(&(((u - CR_OV) / pitch).floor() as usize)) {
                crack = (1.0 - sstep(0.0002, 0.0010, (v - CR_VF).abs())) * f64::from(u8::from(vn(salt ^ 5, u * 1500.0, 2.0) > 0.25));
            }
            if kind == 0 {
                crack = crack
                    .max((1.0 - sstep(0.0002, 0.0012, (u - CR_KINK).abs())) * f64::from(u8::from(vn(salt ^ 6, v * 1500.0, 3.0) > 0.3)));
            }
            c = mix(c, card, crack * 0.8);
            // grime at its foot, dust settled on it
            c = mix(c, [0.24, 0.21, 0.17], (1.0 - sstep(0.0, 0.012, v)) * 0.55);
            let dust = fbm(salt ^ 7, u * 16.0, v * 16.0, 4);
            c = mix(c, [0.58, 0.55, 0.50], 0.10 + 0.22 * sstep(0.35, 0.75, dust));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

// --------------------------------------------------------- the streamers
/// The streamers as modelled (`t12.py` STR_KINDS): each one's length, and
/// the way it ran from its tape (deg round y from +x) when it was a swag
const STR_LEN: [f64; 4] = [1.62, 1.36, 1.12, 0.92];
const STR_HEAD: [f64; 4] = [12.0, -20.0, 5.0, -8.0];
/// Crepe paper after years: each old colour faded (crepe's dyes are
/// fugitive)
const CREPE: [u32; 6] = [0xb0707e, 0x6d7fa6, 0xcbb878, 0x7c9a76, 0x8f7495, 0xc597ac];

/// Hanging streamers: 8% chance per level. Lengths of crepe paper
/// streamer (1.75 in, Blender batch `t12`) hanging from the ceiling, each
/// from the bit of masking tape that held it when it was a swag across the
/// room: twisted as it was hung, the crepe stretched and sagged, the ends
/// torn; faded, water-marked, dusty. Some still stir in the draught.
pub fn place_hanging_streamers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let streamer_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let grp = group();
    let n_streamers = 3 + (rnd() * 4.0).floor() as i32;
    let crepe = crepe_tex();
    let tape = lambert!({ color: 0x8e7b58 });
    for _ in 0..n_streamers {
        // the old draws, in their order: its colour, its length, where under
        // the tape's spot it hangs, its turn, and whether it stirs
        let c = pick(&streamer_colors);
        let k = streamer_colors.iter().position(|&s| s == c).unwrap_or(0);
        let len = 0.9 + rnd() * 0.8;
        let sx = (rnd() - 0.5) * 0.35;
        let sz = (rnd() - 0.5) * 0.35;
        let turn = rnd() * PI * 2.0;
        // the modelled length nearest the old one
        let kind = (0..4).min_by(|&a, &b| (STR_LEN[a] - len).abs().total_cmp(&(STR_LEN[b] - len).abs())).unwrap_or(0);
        let hang = group();
        hang.position().set(sx, WALL_H, sz);
        hang.rotation().y = turn;
        grp.add(&hang);
        let mat = lambert!({ map: &crepe, color: CREPE[k], side: DoubleSide });
        let off = (k as f64) * 0.37 + sx * 3.0;
        put_with(&hang, ["Real__streamerA", "Real__streamerB", "Real__streamerC", "Real__streamerD"][kind], &mat, |gg| {
            // the grain runs across the strip, so along its length (down)
            let n = gg.position.len() / 3;
            for i in 0..n {
                let (x, y, z) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 1]), f64::from(gg.position[i * 3 + 2]));
                gg.uv[i * 2] = ((x + z) * 4.0 + off) as f32;
                gg.uv[i * 2 + 1] = (-y * 2.0 + off) as f32;
            }
        });
        // the tape across its end, under it against the ceiling
        let t = group();
        t.rotation().y = -(STR_HEAD[kind].to_radians() + PI / 2.0);
        t.position().y = -0.0008;
        hang.add(&t);
        if let Some(m) = put(&t, "Real__posterTape", &tape) {
            m.rotation().x = PI / 2.0;
        }
        if rnd() < 0.50 {
            // it stirs, barely, about its tape
            let sway_phase = rnd() * PI * 2.0;
            let sway_period = 6.0 + rnd() * 6.0;
            let s = hang.clone();
            g.animate(move |_g, _dt, t| {
                s.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.012;
            });
        }
    }
    grp.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

/// Crepe paper's grain over its colour (multiplied): the creping across
/// the strip, faded and water-marked in patches, dusty (u across, v along:
/// 0.5 m a repeat)
fn crepe_tex() -> Tex {
    canvas_tex(64, 512, |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let y = v * 0.5;
            // the creping: fine ridges, uneven, a shade darker in their folds
            let r = (y * 520.0 + vn(11, u * 4.0, y * 60.0) * 2.0).sin() * 0.5 + 0.5;
            let mut c = 0.90 - 0.07 * r * (0.6 + 0.4 * vn(12, u * 8.0, y * 30.0));
            // faded pale in patches, a tide line where water ran down it
            c += 0.08 * sstep(0.55, 0.85, fbm(13, u * 2.0, y * 6.0, 3));
            let tide = (y * 2.0 + 0.15 * fbm(14, u * 3.0, y * 4.0, 2)).rem_euclid(1.0);
            c -= 0.10 * (1.0 - sstep(0.0, 0.025, (tide - 0.62).abs())) * sstep(0.4, 0.6, vn(15, 1.0, y * 2.0));
            let cc = [c, c * 0.985, c * 0.96];
            let cc = mix(cc, [0.70, 0.67, 0.61], 0.15 * fbm(16, u * 6.0, y * 20.0, 3));
            [cc[0], cc[1], cc[2], 1.0]
        });
    })
}

// ------------------------------------------------------ the tassel garland
/// The garland as modelled (`t12.py` GT_*): the twine's spans, each one's
/// catenary, and how far under the ceiling its ends run
const GT_SPANS: [f64; 3] = [1.55, 1.90, 2.25];
const GT_CAT: [f64; 3] = [1.490_950, 1.859_480, 2.228_835];
const GT_DROP: f64 = 0.004;
/// Tissue paper after years: each old colour faded
const TISSUE: [u32; 7] = [0xb57884, 0x7383a6, 0xcdbd84, 0x86a07e, 0x96809c, 0xc9a2b3, 0xcfc8b8];

/// Paper garland: 7% chance per level. A tissue paper tassel garland
/// (Blender batch `t12`): tassels of fringed tissue, each twisted into a
/// loop on a jute twine swagged between two pieces of tape on the ceiling;
/// the tissue faded and limp, the strands hanging in clumps, some torn
/// short, a tassel or two gone from the twine.
pub fn place_paper_garlands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let g_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0, 0xf0ece4];
    let garland = group();
    let span_len = 1.4 + rnd() * 1.0;
    let n_dots = 10 + (rnd() * 6.0).floor() as i32;
    // the old cut-outs' colours: the tassels', one each
    let colours: Vec<usize> = (0..n_dots)
        .map(|_| {
            let c = pick(&g_colors);
            g_colors.iter().position(|&s| s == c).unwrap_or(0)
        })
        .collect();
    let px = g.wx(x) + (rnd() - 0.5) * 1.5;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.5;
    garland.position().set(px, 0, pz);
    garland.rotation().y = rnd() * PI * 2.0;
    let salt = salt_of(px, pz);
    // the twine of the span nearest the old one, the whole garland scaled to
    // it (within 11%)
    let kind = (0..3).min_by(|&a, &b| (GT_SPANS[a] - span_len).abs().total_cmp(&(GT_SPANS[b] - span_len).abs())).unwrap_or(0);
    let span = GT_SPANS[kind];
    let swag = group();
    swag.position().y = WALL_H;
    swag.scale().set_scalar(span_len / span);
    garland.add(&swag);
    put(&swag, ["Real__garlandTwineS", "Real__garlandTwineM", "Real__garlandTwineL"][kind], &lambert!({ color: 0x7a6a50 }));
    let tape = lambert!({ color: 0x8e7b58 });
    for sx in [-1.0, 1.0] {
        let t = group();
        t.position().set(sx * (span / 2.0 + 0.016), -0.0031, 0);
        t.rotation().y = PI / 2.0;
        swag.add(&t);
        if let Some(m) = put(&t, "Real__posterTape", &tape) {
            m.rotation().x = PI / 2.0;
        }
    }
    let a = GT_CAT[kind];
    let n = colours.len();
    for (i, &k) in colours.iter().enumerate() {
        let h = hash(salt, i as i32, 7);
        if h < 0.10 {
            continue; // fallen off the twine long ago
        }
        let tx = ((i as f64 + 0.5) / n as f64 - 0.5) * span;
        let ty = a * ((tx / a).cosh() - (span / (2.0 * a)).cosh()) - GT_DROP;
        let tg = group();
        tg.position().set(tx, ty, 0);
        tg.rotation().y = (hash(salt, i as i32, 8) - 0.5) * 0.6 + if h > 0.55 { PI } else { 0.0 };
        swag.add(&tg);
        let which =
            ["Real__garlandTasselA", "Real__garlandTasselB", "Real__garlandTasselC"][((h - 0.10) / 0.9 * 3.0).floor().min(2.0) as usize];
        put(&tg, which, &lambert!({ color: TISSUE[k], side: DoubleSide }));
    }
    g.world.add(&garland);
}

// ------------------------------------------------------------- the piñata
/// The piñata's cords as modelled (`t12.py` PN_CORDS): how far below its
/// hook the ball's tie hangs
const PN_CORD: [f64; 3] = [0.45, 0.60, 0.75];

/// Small piñata: 5% chance per level. A small star piñata (Blender
/// batch `t12`): a papier-mâché ball and seven card cones under rows of
/// fringed tissue, tissue streamers at the tips, hung by its cotton cord
/// from a cup hook in the ceiling; split open on one side where it was hit
/// (empty, its torn edge of newsprint, the dark inside), a cone crushed;
/// the tissue faded, the fringe lifted and torn. It turns, barely, on its
/// cord.
pub fn place_small_pinatas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old body and accent colours: the ball's tissue and the cones'
    let pin_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let cb = pick(&pin_colors);
    let kb = pin_colors.iter().position(|&c| c == cb).unwrap_or(0);
    let ca = pick(&pin_colors);
    let ka = pin_colors.iter().position(|&c| c == ca).unwrap_or(0);
    let pinata = group();
    // the old string's length: the cord nearest it
    let string_len = 0.4 + rnd() * 0.4;
    let kind = (0..3).min_by(|&a, &b| (PN_CORD[a] - string_len).abs().total_cmp(&(PN_CORD[b] - string_len).abs())).unwrap_or(0);
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    pinata.position().set(px, 0, pz);
    pinata.rotation().y = rnd() * PI * 2.0;
    // it hangs from the hook: what sways turns about it
    let hang = group();
    hang.position().y = WALL_H;
    pinata.add(&hang);
    put(&hang, "Real__pinataHook", &phong!({ color: 0x5a4a2c, specular: 0x7a6a44, shininess: 40 }));
    put(&hang, ["Real__pinataCordS", "Real__pinataCordM", "Real__pinataCordL"][kind], &lambert!({ color: 0xa69c86 }));
    let body = group();
    body.position().y = -PN_CORD[kind];
    hang.add(&body);
    let body_tissue = lambert!({ color: TISSUE[kb], side: DoubleSide });
    put(&body, "Real__pinataFringeBody", &body_tissue);
    put(&body, "Real__pinataTails", &body_tissue);
    let cone_tissue = lambert!({ color: TISSUE[ka], side: DoubleSide });
    put(&body, "Real__pinataFringeCones", &cone_tissue);
    put(&body, "Real__pinataTailsB", &cone_tissue);
    // the split: newsprint papier-mâché torn, the hollow dark
    put(&body, "Real__pinataBroken", &lambert!({ color: 0x8a857a }));
    put(&body, "Real__pinataInside", &lambert!({ color: 0x24211c }));
    g.world.add(&pinata);
    // the old sway: a slow turn on its cord
    if rnd() < 0.60 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 10.0 + rnd() * 8.0;
        let p = hang.clone();
        g.animate(move |_g, _dt, t| {
            p.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.012;
        });
    }
}

/// The painting kit of batch `t12`'s painters (as batches `t08`, `t09`):
/// value noise and a per-pixel fill (pure, so `par_rows` may run it on
/// every core), a salt from where a thing is, a part placed.
#[allow(dead_code)]
mod kit {
    use game::prelude::*;

    pub(super) fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    pub(super) fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    pub(super) fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    pub(super) fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    pub(super) fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v
    /// up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    pub(super) fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
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
    /// A salt from where a thing is (no draw): its painters' strokes differ
    /// from floor to floor
    pub(super) fn salt_of(x: f64, z: f64) -> u32 {
        let (a, b) = ((x * 1000.0).round() as i64 as u64, (z * 1000.0).round() as i64 as u64);
        let mut h = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
        h ^= h >> 29;
        (h ^ (h >> 32)) as u32
    }
    /// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
    /// The same with its geometry (a copy) changed by `edit` first (its
    /// texture coordinates)
    pub(super) fn put_with(grp: &Obj, name: &str, mat: &Mat, edit: impl FnOnce(&mut Geometry)) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?.duplicate();
        {
            let mut gg = geo.g();
            edit(&mut gg);
            gg.touch();
        }
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
