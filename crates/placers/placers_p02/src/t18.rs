//! Batch `t18`'s occult things of this crate, made real (`docs/PROPS.md`
//! P3, "the party and the shrine"): what a person brought and left, never a
//! glowing or working device (`assets/src/blender/props/t18.py`,
//! `t18.notes.md`): a fan of tarot cards (the Rider deck's faces, its
//! backs), a pentagram drawn in chalk on the floor, a printed talking board
//! and its wooden planchette, a short length of proof coil chain. Their
//! placers and what only they use; each keeps the old placer's draws, call
//! for call.
use super::*;
use game::visuals::pictures::{self, Age};
use t18_kit::*;

// R470 tarot card fan: 5% chance per level. Five cards of a tarot deck of
// the Rider's size (70 x 120 mm) fanned on the floor as a reading left
// off: four face down, their backs the Rider's crackled brown, and one face
// up, a card of Pamela Colman Smith's (public domain); the cards' edges
// rubbed white, grimed and creased, one corner dog-eared.
pub(super) fn place_tarot_card_fans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let vary = pictures::vary_of(px, pz, turn);
    let face = if vary % 2 == 0 { "tarot_death" } else { "tarot_moon" };
    let mut age = Age::engraving(vary);
    age.fade = 0.06;
    age.spots = (vary % 7) as usize;
    age.stain = 0.0;
    age.grime = 0.06 + 0.1 * f64::from((vary >> 8) % 17) / 17.0;
    let face_mat = lambert!({ map: &pictures::print_tex(face, CARD, 2600.0, age, vary) });
    let back_mat = lambert!({ map: &tarot_back_tex(age, vary) });
    let bent = (vary >> 4) % 5;
    let cards = group();
    // fanned from a point 10 mm in from their feet (+z), each turned about
    // it; the face-up card the middle one, turned over and laid back on top
    let r = CARD.1 / 2.0 - 0.010;
    for i in 0..5 {
        let a = -0.48 + (f64::from(i) / 4.0) * 0.96;
        let name = if i as u32 == bent && i != 2 { "Real__tarotCardBent" } else { "Real__tarotCard" };
        let mat = if i == 2 { &face_mat } else { &back_mat };
        let layer = if i == 2 { 5.0 } else { f64::from(i) };
        if let Some(m) = put(&cards, name, mat) {
            m.position().set(a.sin() * r, layer * 0.0004, r - a.cos() * r);
            m.rotation().y = -a;
        }
    }
    cards.position().set(px, 0, pz);
    cards.rotation().y = turn;
    g.world.add(&cards);
}

/// A tarot card (m)
const CARD: (f64, f64) = (0.070, 0.120);

/// A card's back as the Rider deck's 1910 printing has it: a crackle of
/// dark lines over a mottled brown ("cracked mud") within a narrow cream
/// border; aged as a print
fn tarot_back_tex(age: Age, vary: u32) -> Tex {
    pictures::sheet_tex(CARD, 2600.0, age, vary, move |ctx, w, h| {
        let seed = vary ^ 0x7a40;
        fill(ctx, w, h, move |u, v| {
            let (x, y) = (u * CARD.0 * 1000.0, (1.0 - v) * CARD.1 * 1000.0);
            let edge = x.min(CARD.0 * 1000.0 - x).min(y).min(CARD.1 * 1000.0 - y);
            if edge < 3.2 {
                return [0.86, 0.82, 0.72, 1.0];
            }
            let (d, id) = crackle(seed, x, y, 3.6);
            let n = fbm(seed ^ 9, x * 0.08, y * 0.08, 3);
            let mut c = mix([0.50, 0.36, 0.24], [0.36, 0.25, 0.16], n * 0.8 + id * 0.3);
            c = mix(c, [0.14, 0.10, 0.07], 1.0 - sstep(0.12, 0.45, d));
            // the printed frame line inside the border
            if (edge - 4.0).abs() < 0.35 {
                c = mix(c, [0.16, 0.12, 0.09], 0.8);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

// R470 pentagram chalk decal: 3% chance per level. A circle drawn twice and
// a five-pointed star inside it, 1.3 m across, chalked on the floor by hand
// with a stick of blackboard chalk: each line a stroke of grains caught on
// the floor's tooth, heavier where pressed, its ends overshooting where the
// hand went on; scuffed through where feet crossed it, its dust about it.
// No light: chalk does not shine (the old glow is gone, its pulse's draw
// kept).
pub(super) fn place_pentagram_chalks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let px = g.wx(x) + (rnd() - 0.5) * 1.5;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.5;
    let spin = rnd() * PI * 2.0;
    // the old glow's pulse phase (no glow: chalk does not shine)
    let _ = rnd();
    let tex = pentagram_tex(salt(px, pz, 0x5e7));
    let mat = lambert!({
        map: &tex,
        color: pale(chalk_color_by_weather(g.weather_key())),
        transparent: true,
        depthWrite: false,
        polygonOffset: true,
        polygonOffsetFactor: -2,
        polygonOffsetUnits: -2
    });
    let decal = mesh(plane_geo!(PENT_SIZE, PENT_SIZE), &mat);
    decal.rotation().x = -PI / 2.0;
    decal.position().set(px, 0.005, pz);
    decal.rotation().z = spin;
    g.world.add(&decal);
}

/// The place's chalk colour (r08's, by the weather) as a stick of it
/// leaves it on a floor: two thirds white
fn pale(c: u32) -> u32 {
    let ch = |s: u32| (255.0 - (255.0 - f64::from((c >> s) & 0xff)) * 0.35).round() as u32;
    (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

/// The chalked square (m) and its texture's side (px: 1.37 mm a pixel)
const PENT_SIZE: f64 = 1.4;
const PENT_PX: f64 = 1024.0;

/// One stroke of chalk (mm, on the decal): an arc of the circle drawn
/// freehand (its centre the decal's), or a line of the star, bowed; how wide
/// the stick drew and where along it the hand pressed
enum Stroke {
    Arc { a0: f64, span: f64, rad: f64, p1: f64, p2: f64, width: f64, wob: f64 },
    Line { a: (f64, f64), d: (f64, f64), l: f64, t0: f64, t1: f64, bow: f64, width: f64, wob: f64 },
}

impl Stroke {
    fn width(&self) -> f64 {
        match self {
            Stroke::Arc { width, .. } | Stroke::Line { width, .. } => *width,
        }
    }
    fn wob(&self) -> f64 {
        match self {
            Stroke::Arc { wob, .. } | Stroke::Line { wob, .. } => *wob,
        }
    }
    /// The arc's radius at angle q
    fn radius(rad: f64, p1: f64, p2: f64, q: f64) -> f64 {
        rad + 9.0 * (2.0 * q + p1).sin() + 5.0 * (5.0 * q + p2).sin()
    }
    /// The distance from (x, y) to the stroke (mm) and how far along it it
    /// is (0..1); None when far from it
    fn at(&self, x: f64, y: f64) -> Option<(f64, f64)> {
        match *self {
            Stroke::Arc { a0, span, rad, p1, p2, .. } => {
                let rho = (x * x + y * y).sqrt();
                if (rho - rad).abs() > 60.0 {
                    return None;
                }
                let q = y.atan2(x);
                let rel = (q - a0).rem_euclid(PI * 2.0);
                if rel <= span {
                    let r = Stroke::radius(rad, p1, p2, a0 + rel);
                    Some(((rho - r).abs(), rel / span))
                } else {
                    // past an end: the distance to the nearer end
                    let (e0, e1) = (a0, a0 + span);
                    let (r0, r1) = (Stroke::radius(rad, p1, p2, e0), Stroke::radius(rad, p1, p2, e1));
                    let d0 = ((x - r0 * e0.cos()).powi(2) + (y - r0 * e0.sin()).powi(2)).sqrt();
                    let d1 = ((x - r1 * e1.cos()).powi(2) + (y - r1 * e1.sin()).powi(2)).sqrt();
                    Some(if d0 < d1 { (d0, 0.0) } else { (d1, 1.0) })
                }
            }
            Stroke::Line { a, d, l, t0, t1, bow, .. } => {
                let (rx, ry) = (x - a.0, y - a.1);
                let t = (rx * d.0 + ry * d.1) / l;
                let perp = -rx * d.1 + ry * d.0;
                let tc = t.clamp(t0, t1);
                let off = bow * (PI * tc.clamp(0.0, 1.0)).sin();
                let along = (tc - t0) / (t1 - t0);
                let dist = (((t - tc) * l).powi(2) + (perp - off).powi(2)).sqrt();
                if dist > 60.0 {
                    return None;
                }
                Some((dist, along))
            }
        }
    }
}

/// The pentagram in chalk (white, its grains in alpha; the material's
/// colour tints it as the place's chalk is tinted)
fn pentagram_tex(salt: u32) -> Tex {
    canvas_tex_opt(PENT_PX, PENT_PX, false, move |ctx, w, h| {
        let seed = salt ^ 0x9e57;
        let mut r = Rng32(seed | 1);
        let mut strokes = vec![];
        // the circle, drawn twice round (an outer ring and an inner), each in
        // two strokes whose ends overlap where the hand went on
        for rad in [655.0, 585.0] {
            let (p1, p2) = (r.f() * 6.3, r.f() * 6.3);
            let mut a = r.f() * 6.3;
            for part in 0..2 {
                let span = PI + if part == 0 { 0.1 + 0.1 * r.f() } else { 0.05 + 0.12 * r.f() };
                let off = (r.f() - 0.5) * 6.0;
                strokes.push(Stroke::Arc { a0: a, span, rad: rad + off, p1, p2, width: 15.0 + 5.0 * r.f(), wob: r.f() * 6.0 });
                a += span - 0.08;
            }
        }
        // the star: its five points on the inner ring, each line drawn from a
        // point to the next but one, bowed a little, overshooting its ends
        let top = PI / 2.0 + (r.f() - 0.5) * 0.08;
        let pts5: Vec<(f64, f64)> = (0..5)
            .map(|k| {
                let q = top + f64::from(k) * 2.0 * PI / 5.0;
                let rr = 572.0 + (r.f() - 0.5) * 16.0;
                (rr * q.cos(), rr * q.sin())
            })
            .collect();
        for k in 0..5 {
            let (a, b) = (pts5[k * 2 % 5], pts5[(k * 2 + 2) % 5]);
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let l = (dx * dx + dy * dy).sqrt();
            let t0 = -(2.0 + 20.0 * r.f()) / l;
            let t1 = 1.0 + (6.0 + 30.0 * r.f()) / l;
            let bow = (r.f() - 0.5) * 22.0;
            strokes.push(Stroke::Line { a, d: (dx / l, dy / l), l, t0, t1, bow, width: 15.0 + 5.0 * r.f(), wob: r.f() * 6.0 });
        }
        // where feet crossed it: scuffs through the lines, the chalk smeared
        let scuffs: Vec<(f64, f64, f64, f64)> =
            (0..4).map(|_| ((r.f() - 0.5) * 1100.0, (r.f() - 0.5) * 1100.0, 40.0 + 70.0 * r.f(), r.f() * PI)).collect();
        let mm_px = PENT_SIZE * 1000.0 / w;
        fill(ctx, w, h, move |u, v| {
            let (x, y) = ((u - 0.5) * PENT_SIZE * 1000.0, (v - 0.5) * PENT_SIZE * 1000.0);
            let mut a = 0.0f64;
            let mut haze = 0.0f64;
            for st in &strokes {
                if let Some((d, along)) = st.at(x, y) {
                    let ends = (along / 0.04).min(1.0) * ((1.0 - along) / 0.06).min(1.0);
                    let wob = st.wob();
                    // pressed harder and lighter along it, and broken where it
                    // skipped over the pile
                    let press = (0.35 + 0.65 * vn(seed ^ 0x61, along * 18.0 + wob * 7.0, 0.5)) * (0.45 + 0.55 * ends);
                    let press = press * sstep(0.1, 0.24, vn(seed ^ 0x62, along * 30.0 + wob * 3.0, 1.5));
                    // its edge ragged where the stick rolled and the floor's pile caught it
                    let half = st.width() * 0.5 * (0.75 + 0.25 * ends) * (0.8 + 0.4 * vn(seed ^ 0x21, x / 5.0, y / 5.0));
                    if d < half + mm_px {
                        let edge = 1.0 - (d / half).min(1.0);
                        // the floor's tooth: grains caught on the pile's tips (1.4
                        // mm), in clumps (3 mm), skipping where the stick skipped
                        let tooth =
                            hash(seed, (x / 1.4).floor() as i32, (y / 1.4).floor() as i32) * 0.5 + vn(seed ^ 0x33, x / 3.0, y / 3.0) * 0.5;
                        let skip = 0.75 + 0.25 * vn(seed ^ 0x44, along * 140.0 + wob * 10.0, 0.5);
                        let k = press * skip * (0.9 + 0.5 * edge) - tooth * 0.85;
                        let grain = sstep(-0.05, 0.3, k) * sstep(half + mm_px, half - mm_px, d);
                        a = a.max(grain * (0.45 + 0.55 * press) * (0.7 + 0.3 * edge));
                    }
                    haze = haze.max(0.13 * (-d / 24.0).exp() * press * (0.6 + 0.4 * vn(seed ^ 0x12, x / 20.0, y / 20.0)));
                }
            }
            if a <= 0.0 && haze <= 0.004 {
                return [1.0, 1.0, 1.0, 0.0];
            }
            // scuffed: the chalk worn off in patches and smeared about them
            let mut keep = 1.0;
            for &(sx, sy, sr, sa) in &scuffs {
                let (dx, dy) = (x - sx, y - sy);
                let (ex, ey) = (dx * sa.cos() + dy * sa.sin(), (-dx * sa.sin() + dy * sa.cos()) * 2.2);
                let q = (ex * ex + ey * ey).sqrt() / sr;
                if q < 1.4 {
                    let n = fbm(seed ^ 0x51, x * 0.02, y * 0.02, 3);
                    let gone = (1.0 - sstep(0.55, 1.25, q + (n - 0.5) * 0.6)).clamp(0.0, 1.0);
                    keep = f64::min(keep, 1.0 - 0.85 * gone);
                    haze = haze.max(0.07 * gone * a.max(0.3));
                }
            }
            let lift = 0.78 + 0.22 * fbm(seed ^ 0x77, x * 0.004, y * 0.004, 2);
            let alpha = (a * keep * lift).max(haze);
            [1.0, 1.0, 1.0, alpha.clamp(0.0, 1.0)]
        });
    })
}

// R469 ouija board decal: 4% chance per level. A printed talking board of
// the old kind (no maker's name on it): a lithographed face on 1/8 in
// hardboard, 18 x 12 in, its letters in two arcs, its numerals and the
// words in a serif face, a sun by YES and a moon by NO; warped, its print
// rubbed through where the planchette went round and round, grimed at its
// edges, a corner knocked off. Its planchette a wooden teardrop on three
// felted legs, which drifts now and then as the old one did.
pub(super) fn place_ouija_boards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let bx = g.wx(x) + (rnd() - 0.5) * 2.0;
    let bz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let spin = rnd() * PI * 2.0;
    let salt = salt(bx, bz, 0x0b1);
    let board = group();
    put(&board, "Real__talkBoardCore", &lambert!({ map: &hardboard_tex(salt) }));
    put(
        &board,
        "Real__talkBoardFace",
        &lambert!({ map: &talk_board_tex(salt), polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -1 }),
    );
    board.position().set(bx, 0, bz);
    board.rotation().y = spin;
    g.world.add(&board);
    // the old planchette's offsets: where it lies on the board (in the
    // board's own frame now, so it always lies on the letters), its turn
    let lx = (rnd() - 0.5) * 0.30 * 0.8;
    let lz = (rnd() - 0.5) * 0.15 * 0.9;
    let pturn = rnd() * PI * 2.0;
    let (c, s) = (spin.cos(), spin.sin());
    let base_x = bx + lx * c + lz * s;
    let base_z = bz - lx * s + lz * c;
    let pl = group();
    put(&pl, "Real__talkPlanchette", &phong!({ map: &planchette_tex(salt), specular: 0x2a2620, shininess: 24 }));
    put(&pl, "Real__talkPlanchetteFelt", &lambert!({ color: 0x24261f }));
    pl.position().set(base_x, board_face_y(lx, lz), base_z);
    pl.rotation().y = pturn;
    g.world.add(&pl);
    // R484 subtle mysterious sway: the planchette slowly drifts as if moved
    // by unseen hands.
    let sway_phase_x = rnd() * PI * 2.0;
    let sway_phase_z = rnd() * PI * 2.0;
    let sway_period = 14.0 + rnd() * 10.0;
    g.animate(move |_g, _dt, t| {
        let mut p = pl.position();
        p.x = base_x + (t * (PI * 2.0 / sway_period) + sway_phase_x).sin() * 0.010;
        p.z = base_z + (t * (PI * 2.0 / sway_period * 1.3) + sway_phase_z).sin() * 0.008;
    });
}

/// The board (mm, `t18.py`): its size and how it has warped, as modelled
const BOARD: (f64, f64) = (457.0, 305.0);

/// The height of the board's printed face at (x, z) of its frame (m)
fn board_face_y(x: f64, z: f64) -> f64 {
    let (u, w) = (x * 1000.0 / (BOARD.0 / 2.0), z * 1000.0 / (BOARD.1 / 2.0));
    (3.46 + 2.6 * u * u + 0.9 * u * w + 0.6 * w * w) / 1000.0
}

/// The board's printed face (its top: u across, v up the board), 2.24 px a
/// mm: the lithograph in a serif face on a tan ground, then its years
fn talk_board_tex(salt: u32) -> Tex {
    let k = 1024.0 / BOARD.0;
    canvas_tex_opt(1024.0, (BOARD.1 * k).round(), false, move |ctx, w, h| {
        let ink = "rgb(38,28,22)";
        ctx.set_fill_style("rgb(184,160,118)");
        ctx.fill_rect(0, 0, w, h);
        ctx.save();
        ctx.scale(k, k);
        // the border: a double rule
        ctx.set_stroke_style(ink);
        ctx.set_line_width(1.6);
        ctx.stroke_rect(9.0, 9.0, BOARD.0 - 18.0, BOARD.1 - 18.0);
        ctx.set_line_width(0.6);
        ctx.stroke_rect(12.5, 12.5, BOARD.0 - 25.0, BOARD.1 - 25.0);
        ctx.set_fill_style(ink);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        // the sun by YES, the moon by NO
        ctx.begin_path();
        ctx.arc(38.0, 40.0, 11.0, 0, PI * 2.0);
        ctx.fill();
        ctx.set_line_width(1.4);
        for i in 0..16 {
            let a = f64::from(i) * PI / 8.0;
            let (r0, r1) = (14.0, if i % 2 == 0 { 22.0 } else { 19.0 });
            ctx.begin_path();
            ctx.move_to(38.0 + a.cos() * r0, 40.0 + a.sin() * r0);
            ctx.line_to(38.0 + a.cos() * r1, 40.0 + a.sin() * r1);
            ctx.stroke();
        }
        ctx.begin_path();
        ctx.arc(420.0, 40.0, 15.0, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style("rgb(184,160,118)");
        ctx.begin_path();
        ctx.arc(427.0, 35.0, 13.5, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style(ink);
        ctx.set_font("bold 22px serif");
        ctx.fill_text("YES", 82.0, 42.0);
        ctx.fill_text("NO", 378.0, 42.0);
        // the letters in two arcs
        ctx.set_font("bold 27px serif");
        let arc = |ctx: &mut Canvas, letters: &str, cy: f64, rad: f64, span: f64| {
            let n = letters.chars().count();
            for (i, ch) in letters.chars().enumerate() {
                let a = -span + 2.0 * span * i as f64 / (n - 1) as f64;
                ctx.save();
                ctx.translate(BOARD.0 / 2.0 + rad * a.sin(), cy - rad * a.cos());
                ctx.rotate(a);
                ctx.fill_text(&ch.to_string(), 0.0, 0.0);
                ctx.restore();
            }
        };
        arc(ctx, "ABCDEFGHIJKLM", 560.0, 438.0, 0.36);
        arc(ctx, "NOPQRSTUVWXYZ", 604.0, 428.0, 0.35);
        ctx.set_font("bold 24px serif");
        for (i, d) in "1234567890".chars().enumerate() {
            ctx.fill_text(&d.to_string(), 118.0 + 24.6 * i as f64, 232.0);
        }
        ctx.set_font("bold 21px serif");
        ctx.fill_text("GOOD BYE", BOARD.0 / 2.0, 272.0);
        ctx.restore();
        // its years: the tan gone uneven, the print rubbed through in a band
        // where the planchette went round, scratches, grime at the edges, a
        // tide line, dust; the corner knocked off to the hardboard
        let src = ctx.get_image_data(0, 0, w, h);
        let iw = w as usize;
        let mut img = ctx.create_image_data(w, h);
        let seed = salt ^ 0xb0a;
        let scratches: Vec<(f64, f64, f64, f64)> = {
            let mut r = Rng32(seed | 1);
            (0..9).map(|_| (r.f() * BOARD.0, r.f() * BOARD.1, r.f() * PI, 30.0 + 120.0 * r.f())).collect()
        };
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let (x, y) = ((i as f64 + 0.5) / k, (j as f64 + 0.5) / k);
                let o = (j * iw + i) * 4;
                let px = [f64::from(src.data[o]) / 255.0, f64::from(src.data[o + 1]) / 255.0, f64::from(src.data[o + 2]) / 255.0];
                let paper = [184.0 / 255.0, 160.0 / 255.0, 118.0 / 255.0];
                let inkness = (1.0 - (px[0] - 0.149) / (paper[0] - 0.149)).clamp(0.0, 1.0);
                // the ground mottled, the ink lifted where rubbed
                let n = fbm(seed, x * 0.03, y * 0.03, 4);
                let mut c = mix(paper, [0.62, 0.52, 0.37], n * 0.7);
                let (rx, ry) = ((x - BOARD.0 / 2.0) / 175.0, (y - 185.0) / 78.0);
                let rub = (1.0 - sstep(0.6, 1.15, (rx * rx + ry * ry).sqrt())) * (0.55 + 0.45 * vn(seed ^ 5, x * 0.4, y * 0.04));
                let ink_left = inkness * (1.0 - 0.55 * rub);
                c = mix(c, [0.16, 0.12, 0.09], ink_left * (0.85 + 0.15 * vn(seed ^ 6, x * 2.0, y * 2.0)));
                // rubbed to the brown under the print in the worst of it
                c = mix(c, [0.52, 0.42, 0.30], rub * 0.35 * sstep(0.55, 0.8, vn(seed ^ 7, x * 0.25, y * 0.9)));
                for &(sx, sy, sa, sl) in &scratches {
                    let (dx, dy) = (x - sx, y - sy);
                    let along = dx * sa.cos() + dy * sa.sin();
                    let across = (-dx * sa.sin() + dy * sa.cos()).abs();
                    if along.abs() < sl / 2.0 && across < 0.45 {
                        c = mix(c, [0.78, 0.72, 0.60], 0.55 * (1.0 - across / 0.45));
                    }
                }
                // grime at the edges, darker at the near edge where hands held it
                let edge = x.min(BOARD.0 - x).min(y).min(BOARD.1 - y);
                let grime =
                    (1.0 - sstep(2.0, 26.0, edge)) * (0.4 + 0.4 * fbm(seed ^ 8, x * 0.08, y * 0.08, 2)) + sstep(255.0, 305.0, y) * 0.12;
                c = mix(c, [0.24, 0.19, 0.13], grime);
                // a tide line where water stood on its far left
                let tq = ((x - 70.0).powi(2) / 1.6 + (y - 70.0).powi(2)).sqrt() / 92.0 + (vn(seed ^ 9, x * 0.05, y * 0.05) - 0.5) * 0.25;
                c = mix(
                    c,
                    [0.46, 0.36, 0.24],
                    sstep(0.84, 0.97, tq) * (1.0 - sstep(0.97, 1.03, tq)) * 0.6 + (1.0 - sstep(0.0, 0.97, tq)) * 0.08,
                );
                // the knocked corner (near right): the face gone to the board
                let cd =
                    ((x - (BOARD.0 - 4.0)).powi(2) + (y - (BOARD.1 - 6.0)).powi(2)).sqrt() + (vn(seed ^ 10, x * 0.5, y * 0.5) - 0.5) * 5.0;
                if cd < 12.0 {
                    c = mix(c, [0.33, 0.24, 0.15], 0.9);
                }
                // dust
                let d = 0.10 + 0.08 * fbm(seed ^ 11, x * 0.05, y * 0.05, 2);
                c = mix(c, [0.62, 0.60, 0.56], d);
                for kk in 0..3 {
                    out[i * 4 + kk] = (c[kk].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// The board's hardboard (its edges and its back): brown, pressed smooth
/// one side and screen-textured the other, grimed
fn hardboard_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x4bd;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 9.0, v * 9.0, 3);
            let c = mix([0.30, 0.22, 0.15], [0.22, 0.16, 0.11], n);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The planchette's wood (maple, varnished once, gone amber): its grain,
/// darker with handling round its rim, dust on top
fn planchette_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x91a;
        fill(ctx, w, h, |u, v| {
            let g = (vn(seed, u * 3.0, v * 40.0) * 0.6 + fbm(seed ^ 3, u * 8.0, v * 8.0, 2) * 0.4).clamp(0.0, 1.0);
            let mut c = mix([0.54, 0.43, 0.30], [0.43, 0.33, 0.22], g);
            let rim = (((u - 0.5).abs().max((v - 0.5).abs())) - 0.36).max(0.0) * 6.0;
            c = mix(c, [0.26, 0.18, 0.11], rim.min(0.7));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

// R467 small chain links: 8% chance per level. A short length of 3/8 in
// proof coil chain (5 to 8 welded links of 9.9 mm wire, 55 mm long) lying
// where it was dropped, every other link flat and the ones between on edge,
// leaning; galvanized gone dull with white rust, bare steel rusted, or
// black oiled steel blooming rust, by the old colour.
pub(super) fn place_small_chain_links(g: &mut Game, _dist: &[i32]) {
    g.pp.chain_positions.clear(); // R486 reset per level
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let chain_colors = [0x606870, 0x808088, 0x40484c];
    let finish: u32 = pick(&chain_colors);
    let n_links = 5 + (rnd() * 4.0).floor() as i32;
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 0xc4a);
    let mat = match finish {
        // hot-dip galvanized gone dull grey, white rust in its hollows
        0x808088 => phong!({ map: &chain_tex(salt, 0), specular: 0x262626, shininess: 18 }),
        // self-coloured steel, rusted
        0x606870 => lambert!({ map: &chain_tex(salt, 1) }),
        // black oiled, rust blooming through
        _ => phong!({ map: &chain_tex(salt, 2), specular: 0x1c1c1c, shininess: 14 }),
    };
    let chain = group();
    put(&chain, &format!("Real__chainLinks{}", n_links.clamp(5, 8)), &mat);
    chain.position().set(px, 0, pz);
    chain.rotation().y = turn;
    g.world.add(&chain);
    // R486 track for jangle audio.
    let cp = chain.pos();
    g.pp.chain_positions.push(v3(cp.x, 0.02, cp.z));
}

/// The chain's steel (box-mapped over the whole length): `kind` 0
/// galvanized, 1 rusted bare steel, 2 black oiled
fn chain_tex(salt: u32, kind: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x3c3 ^ kind.wrapping_mul(0x51);
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 7.0, v * 7.0, 3);
            let s = vn(seed ^ 4, u * 31.0, v * 31.0);
            let rust = [0.34, 0.17, 0.08];
            let c = match kind {
                0 => {
                    let mut c = mix([0.40, 0.41, 0.40], [0.30, 0.31, 0.31], n);
                    c = mix(c, [0.62, 0.61, 0.57], sstep(0.62, 0.8, s) * 0.7);
                    mix(c, rust, sstep(0.68, 0.85, n) * 0.5)
                }
                1 => {
                    let c = mix([0.30, 0.17, 0.09], [0.20, 0.12, 0.07], n);
                    mix(c, [0.26, 0.25, 0.24], sstep(0.55, 0.8, s) * 0.35)
                }
                _ => {
                    let c = mix([0.07, 0.07, 0.07], [0.11, 0.10, 0.09], n);
                    mix(c, rust, sstep(0.6, 0.82, fbm(seed ^ 6, u * 12.0, v * 12.0, 2)) * 0.8)
                }
            };
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The painters' small kit (noise, a fill on every core, a crackle, a salt
/// from where a thing is, a part placed); private to this batch
#[allow(dead_code)]
mod t18_kit {
    use game::prelude::*;

    /// A small stream of its own for a painter's layout (no level draw)
    pub(super) struct Rng32(pub(super) u32);
    impl Rng32 {
        pub(super) fn f(&mut self) -> f64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 17;
            self.0 ^= self.0 << 5;
            f64::from(self.0) / 4_294_967_296.0
        }
    }
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
    /// The distance (in cells) to the nearest edge of a Voronoi crackle of
    /// cells `size` across, and the cell's own value
    pub(super) fn crackle(seed: u32, x: f64, y: f64, size: f64) -> (f64, f64) {
        let (gx, gy) = (x / size, y / size);
        let (cx, cy) = (gx.floor() as i32, gy.floor() as i32);
        let (mut d1, mut d2, mut id) = (9.0f64, 9.0f64, 0.0);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (ix, iy) = (cx + dx, cy + dy);
                let px = f64::from(ix) + 0.1 + 0.8 * hash(seed, ix, iy);
                let py = f64::from(iy) + 0.1 + 0.8 * hash(seed ^ 0x55, ix, iy);
                let d = ((gx - px).powi(2) + (gy - py).powi(2)).sqrt();
                if d < d1 {
                    d2 = d1;
                    d1 = d;
                    id = hash(seed ^ 0x99, ix, iy);
                } else if d < d2 {
                    d2 = d;
                }
            }
        }
        (d2 - d1, id)
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
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
