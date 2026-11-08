//! Batch `t11`'s lodge trophies, made real (`docs/PROPS.md` P3; the
//! Blender batch `assets/src/blender/props/t11.py`, its notes
//! `t11.notes.md`): a roe buck's antlers on their skull plate and oak
//! shield, a trout skin-mounted on its oval board. Each keeps the old
//! placer's draws, call for call, and hangs where the old one hung.
use crate::{face_yaw, pick, wall_tiles};
use game::prelude::*;
use t11_kit::*;

/// The antlers' place on the trophy (`t11.py` `TROPHY_SKULL`): the burrs'
/// bases above the shield's middle, out from the wall on the skull plate
const TROPHY_ANTLERS: [f64; 3] = [0.0, 0.052, 0.046];
/// The pair's turn about x on the trophy: the skull's (its frontal to the
/// room, `TROPHY_TURN` = 90 degrees) less the antlers' lean off the frontal
/// (`ROE_BETA`, 52 degrees)
const TROPHY_ANTLER_TURN: f64 = 0.663_225;

// Mounted deer antler pair: 3% chance per level. A roe buck's
// antlers, a six-pointer, on the skull plate they grew from (the frontal
// bones sawn through the orbits, boiled and bleached), screwed from behind
// to a carved oak shield (15 x 22 cm) and hung on the wall: the burrs and
// the beams' pearling dark with old sap, the tines' points rubbed white,
// the bone gone ivory, dust on all of it.
pub fn place_mounted_antlers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let (wx, wz) = (g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    let k = salt(wx, wz, 11);
    let mount = group();
    let trophy = group();
    trophy.position().y = 1.80;
    put(&trophy, "Real__roeTrophyShield", &phong!({ map: &oak_tex(k, 128, 192), specular: 0x141210, shininess: 16 }));
    put(&trophy, "Real__roeTrophySkull", &lambert!({ map: &bone_tex(k) }));
    let antlers = group();
    antlers.position().set(TROPHY_ANTLERS[0], TROPHY_ANTLERS[1], TROPHY_ANTLERS[2]);
    antlers.rotation().x = TROPHY_ANTLER_TURN;
    put(&antlers, "Real__roeAntlers", &lambert!({ map: &antler_tex(k) }));
    put(&antlers, "Real__roeAntlerTips", &phong!({ color: 0xcfc3a8, specular: 0x262420, shininess: 30 }));
    trophy.add(&antlers);
    mount.add(&trophy);
    mount.rotation().y = face_yaw(dx, dz);
    mount.position().set(wx, 0, wz);
    g.world.add(&mount);
}

// Mounted trophy fish: 3% chance per level. A 30 cm trout as a
// taxidermist mounted one for fifty years: the skin over a carved body, its
// show side out, its back flat to a walnut-stained oval board with an ogee
// edge, the tail swept out off it, the fins spread, a glass eye; painted
// to its kind (the old colour pick: a rainbow, a brook trout, a brown or a
// lake trout), lacquered, the lacquer gone amber, dust on its back.
pub fn place_mounted_fish(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    // the old body colour's pick: the kind of trout
    let kind = pick(&[0usize, 1, 2, 3]);
    let (wx, wz) = (g.wx(x) + dx as f64 * (CELL * 0.5 - 0.017), g.wz(z) + dz as f64 * (CELL * 0.5 - 0.017));
    let k = salt(wx, wz, 12);
    let mount = group();
    let fish = group();
    fish.position().y = 1.65;
    put(&fish, "Real__trophyFishPlaque", &phong!({ map: &walnut_tex(k), specular: 0x161412, shininess: 20 }));
    // the skin's paint under its lacquer: a little gloss, gone dull with dust
    put(&fish, "Real__trophyFishBody", &phong!({ map: &trout_tex(k, kind), specular: 0x2a2824, shininess: 34 }));
    put(&fish, "Real__trophyFishFins", &phong!({ map: &trout_fin_tex(k, kind), specular: 0x2a2824, shininess: 34 }));
    // a glass eye: it catches the light, it never makes any
    put(&fish, "Real__trophyFishEye", &phong!({ map: &fish_eye_tex(), specular: 0x8a8a8a, shininess: 110 }));
    mount.add(&fish);
    mount.rotation().y = face_yaw(dx, dz);
    mount.position().set(wx, 0, wz);
    g.world.add(&mount);
}

/// Quartersawn oak under an old dark stain and varnish (u across, v up the
/// shield): the grain's straight lines, the rays' pale flecks across them,
/// the stain uneven, the varnish crazed, dust in the pores
fn oak_tex(salt: u32, w: u32, h: u32) -> Tex {
    canvas_tex_opt(w, h, false, move |ctx, w, h| {
        let seed = salt ^ 0x0a4;
        fill(ctx, w, h, |u, v| {
            let base = [0.33, 0.22, 0.13];
            let grain = fbm(seed, u * 46.0 + fbm(seed ^ 13, u * 3.0, v * 6.0, 2) * 4.0, v * 3.0, 3);
            let mut c = mix(base, [0.20, 0.12, 0.07], sstep(0.45, 0.72, grain) * 0.5);
            // the rays: short pale flakes across the grain
            let ray = vn(seed ^ 3, u * 30.0, v * 160.0);
            c = mix(c, [0.46, 0.33, 0.20], sstep(0.78, 0.92, ray) * sstep(0.45, 0.6, fbm(seed ^ 5, u * 8.0, v * 8.0, 2)) * 0.6);
            // the stain uneven, lighter where hands rubbed the edges
            let blot = fbm(seed ^ 7, u * 5.0, v * 5.0, 3);
            c = mix(c, [0.40, 0.28, 0.17], (blot - 0.5).max(0.0) * 0.8);
            let edge = (u.min(1.0 - u)).min(v.min(1.0 - v));
            c = mix(c, [0.44, 0.31, 0.19], (1.0 - sstep(0.0, 0.06, edge)) * 0.35);
            // the varnish crazed: fine dark cracks
            let cr = (vn(seed ^ 9, u * 90.0, v * 120.0) - 0.5).abs();
            c = mix(c, [0.10, 0.06, 0.03], (1.0 - sstep(0.0, 0.03, cr)) * 0.35);
            // dust in the pores and over it
            let d = fbm(seed ^ 11, u * 60.0, v * 80.0, 2);
            c = mix(c, [0.48, 0.45, 0.40], sstep(0.55, 0.8, d) * 0.25 + 0.08);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The skull plate's bone: boiled and bleached, gone ivory and yellowed in
/// its pits, stained brown up toward the pedicles where the antlers' sap
/// ran, dust
fn bone_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x0b1;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.70, 0.65, 0.54];
            let n = fbm(seed, u * 6.0, v * 6.0, 3);
            c = mix(c, [0.62, 0.53, 0.39], sstep(0.4, 0.75, n) * 0.7);
            // toward the pedicles (the top of the plate as it hangs)
            c = mix(c, [0.48, 0.37, 0.25], sstep(0.5, 0.95, v) * 0.55 * (0.6 + 0.4 * n));
            // grime in the sutures and the orbits' edges
            let su = (fbm(seed ^ 7, u * 9.0, v * 4.0, 2) - 0.5).abs();
            c = mix(c, [0.36, 0.30, 0.22], (1.0 - sstep(0.0, 0.03, su)) * 0.5);
            let pit = vn(seed ^ 3, u * 40.0, v * 40.0);
            c = mix(c, [0.52, 0.47, 0.38], sstep(0.75, 0.9, pit) * 0.6);
            let d = vn(seed ^ 5, u * 70.0, v * 70.0);
            c = mix(c, [0.62, 0.60, 0.55], sstep(0.82, 0.95, d) * 0.5);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A roe's antler (v up the beam): the burr and the pearls dark with old
/// sap, their crowns rubbed paler, the gutters between them darker, the
/// beam browner and smoother toward the tines
fn antler_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x0c7;
        fill(ctx, w, h, |u, v| {
            let low = 1.0 - sstep(0.15, 0.6, v);
            let mut c = mix([0.40, 0.30, 0.20], [0.20, 0.14, 0.09], low);
            // the gutters: dark streaks along the beam
            let gut = fbm(seed, u * 34.0, v * 3.0, 3);
            c = mix(c, [0.10, 0.07, 0.05], sstep(0.55, 0.75, gut) * 0.6);
            // the pearls' crowns rubbed pale
            let pearl = vn(seed ^ 3, u * 60.0, v * 110.0);
            c = mix(c, [0.62, 0.54, 0.42], sstep(0.72, 0.9, pearl) * (0.35 + 0.4 * low));
            let d = vn(seed ^ 5, u * 90.0, v * 160.0);
            c = mix(c, [0.55, 0.52, 0.46], sstep(0.85, 0.97, d) * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A walnut-stained board (u along it, v across): the grain along, the
/// ogee edge routed round it darker where the stain soaked into the cut,
/// the varnish worn on the edge's lip, dust
fn walnut_tex(salt: u32) -> Tex {
    canvas_tex_opt(192, 96, false, move |ctx, w, h| {
        let seed = salt ^ 0x0d3;
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 2.0 - 1.0, v * 2.0 - 1.0);
            let r = (x * x + y * y).sqrt();
            let grain = fbm(seed, u * 3.0, v * 30.0 + fbm(seed ^ 1, u * 4.0, v * 4.0, 2) * 3.0, 3);
            let mut c = mix([0.30, 0.19, 0.11], [0.17, 0.10, 0.06], sstep(0.4, 0.72, grain));
            c = mix(c, [0.36, 0.24, 0.14], (fbm(seed ^ 3, u * 6.0, v * 6.0, 2) - 0.5).max(0.0) * 0.8);
            // the routed edge
            c = mix(c, [0.14, 0.08, 0.05], sstep(0.90, 0.95, r) * 0.6);
            c = mix(c, [0.42, 0.30, 0.19], sstep(0.955, 0.975, r) * (1.0 - sstep(0.985, 1.0, r)) * 0.5);
            let d = fbm(seed ^ 5, u * 70.0, v * 35.0, 2);
            c = mix(c, [0.48, 0.45, 0.40], sstep(0.55, 0.8, d) * 0.22 + 0.07);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The trout's body (`t11.py`): where each point of the texture lies on
/// the fish, and how deep the body is there
mod trout {
    /// The body part's bounds in x and y (`Real__trophyFishBody`)
    pub const X0: f64 = -0.148_98;
    pub const X1: f64 = 0.149_271;
    pub const Y0: f64 = -0.046_066;
    pub const Y1: f64 = 0.066_925;
    const SNOUT: f64 = -0.150;
    const BODY: f64 = 0.255;
    const H: f64 = 0.0330;
    const DEPTH: [(f64, f64); 12] = [
        (0.0, 0.08),
        (0.03, 0.30),
        (0.08, 0.52),
        (0.16, 0.74),
        (0.24, 0.88),
        (0.36, 1.0),
        (0.5, 0.96),
        (0.64, 0.80),
        (0.78, 0.56),
        (0.88, 0.44),
        (0.95, 0.42),
        (1.0, 0.44),
    ];

    fn prof(s: f64) -> f64 {
        for w in DEPTH.windows(2) {
            let ((s0, v0), (s1, v1)) = (w[0], w[1]);
            if s >= s0 && s <= s1 {
                let t = (s - s0) / (s1 - s0);
                let t = t * t * (3.0 - 2.0 * t);
                return v0 + (v1 - v0) * t;
            }
        }
        DEPTH[DEPTH.len() - 1].1
    }
    /// At (u, v) of the body's texture: s along the body (0 the snout, 1
    /// the tail's root, past it the tail), q up its depth (-1 the belly's
    /// edge, 1 the back's), and x, y in metres
    pub fn at(u: f64, v: f64) -> (f64, f64, f64, f64) {
        let x = X0 + (X1 - X0) * u;
        let y = Y0 + (Y1 - Y0) * v;
        let s = (x - SNOUT) / BODY;
        let sc = s.clamp(0.0, 1.0);
        let yc = 0.0025 * (std::f64::consts::PI * sc).sin() - 0.002 * sc;
        let h = H * prof(sc);
        (s, (y - yc) / h.max(1e-4), x, y)
    }
}

/// The colours of each kind of trout (straight sRGB): back, flank, belly
const TROUT: [[[f64; 3]; 3]; 4] = [
    // rainbow: olive-grey back, silver flank, white belly
    [[0.30, 0.35, 0.29], [0.66, 0.66, 0.62], [0.84, 0.82, 0.77]],
    // brook: dark olive back, olive flank, orange belly
    [[0.21, 0.24, 0.15], [0.45, 0.44, 0.30], [0.76, 0.50, 0.30]],
    // brown: olive-brown back, golden flank, cream belly
    [[0.33, 0.28, 0.17], [0.64, 0.54, 0.33], [0.84, 0.78, 0.60]],
    // lake: grey-green back, grey flank, white belly
    [[0.25, 0.28, 0.27], [0.42, 0.45, 0.42], [0.84, 0.84, 0.79]],
];

/// Spots on a jittered grid: how much of a spot of radius `r` (cells of
/// `cell` m) covers (x, y), and the spot's own random value
fn spots(seed: u32, x: f64, y: f64, cell: f64, r: f64, keep: f64) -> (f64, f64) {
    let (gx, gy) = (x / cell, y / cell);
    let (ix, iy) = (gx.floor() as i32, gy.floor() as i32);
    let mut best = (0.0, 0.0);
    for j in -1..=1 {
        for i in -1..=1 {
            let (cx, cy) = (ix + i, iy + j);
            let pick = hash(seed, cx, cy);
            if pick > keep {
                continue;
            }
            let ox = hash(seed ^ 0x51, cx, cy);
            let oy = hash(seed ^ 0x93, cx, cy);
            let rr = r * (0.6 + 0.6 * hash(seed ^ 0x77, cx, cy));
            let d = ((gx - (cx as f64 + ox)).powi(2) + (gy - (cy as f64 + oy)).powi(2)).sqrt() / rr;
            let a = 1.0 - sstep(0.75, 1.05, d);
            if a > best.0 {
                best = (a, hash(seed ^ 0x3c, cx, cy));
            }
        }
    }
    best
}

/// The trout's skin as a taxidermist painted it (u along the fish, v up
/// it): its kind's back, flank and belly, its spots (a rainbow's black
/// specks and pink band, a brook trout's pale worm marks, yellow spots and
/// red ones ringed blue, a brown's black and red spots ringed pale, a lake
/// trout's cream spots), the gill cover, the jaw's line, the lateral line,
/// fine scales; the fins' rays; the lacquer gone amber, cracked, dust
fn trout_tex(salt: u32, kind: usize) -> Tex {
    canvas_tex_opt(384, 144, false, move |ctx, w, h| {
        let seed = salt ^ 0x0e5;
        let [back, flank, belly] = TROUT[kind];
        fill(ctx, w, h, |u, v| {
            let (s, q, x, y) = trout::at(u, v);
            let fin = !(0.0..=1.0).contains(&s) || q.abs() > 1.0;
            let mut c;
            if fin {
                // the fins: rays (along the tail; up the dorsal and anal),
                // the membrane between them darker
                let ray = if s > 0.97 { (y * 900.0).sin() } else { (x * 700.0 + y * 120.0).sin() };
                c = mix(flank, back, 0.72);
                c = [c[0] * 0.9, c[1] * 0.9, c[2] * 0.88];
                c = mix(c, [0.16, 0.13, 0.10], (0.5 - 0.5 * ray) * 0.4);
                match kind {
                    0 | 2 => {
                        let (sp, _) = spots(seed ^ 1, x, y, 0.006, 0.28, 0.6);
                        c = mix(c, [0.07, 0.06, 0.05], sp * 0.85);
                        if kind == 2 && s > 0.75 && s < 0.86 && q > 1.0 {
                            // the brown's adipose fin, tipped orange
                            c = mix(c, [0.70, 0.40, 0.20], sstep(1.15, 1.35, q) * 0.8);
                        }
                    }
                    1 => {
                        // the brook's dorsal and tail marbled; its anal fin
                        // red with a white edge and a black stripe
                        if q < -1.0 && s < 0.9 {
                            c = [0.62, 0.32, 0.20];
                            let lead = s - 0.66;
                            c = mix(c, [0.08, 0.07, 0.06], sstep(0.010, 0.016, lead) * (1.0 - sstep(0.024, 0.03, lead)));
                            c = mix(c, [0.86, 0.84, 0.78], 1.0 - sstep(0.006, 0.012, lead));
                        } else {
                            let m = (fbm(seed ^ 3, x * 260.0, y * 260.0, 2) - 0.5).abs();
                            c = mix(c, [0.08, 0.07, 0.05], (1.0 - sstep(0.04, 0.1, m)) * 0.8);
                        }
                    }
                    _ => {
                        let (sp, _) = spots(seed ^ 2, x, y, 0.007, 0.3, 0.55);
                        c = mix(c, [0.76, 0.74, 0.62], sp * 0.7);
                        if q < -1.0 && s < 0.9 {
                            c = mix(c, [0.86, 0.84, 0.78], 1.0 - sstep(0.004, 0.009, s - 0.66));
                        }
                    }
                }
                // a fin is thin: the board's dark shows through it a little
                c = mix(c, [0.20, 0.15, 0.10], 0.12);
            } else {
                // back to belly
                c = mix(belly, flank, sstep(-0.75, -0.2, q));
                c = mix(c, back, sstep(0.15, 0.7, q));
                // fine scales: a faint diamond net on the flank
                let (a, b) = (x * 520.0 + y * 520.0, x * 520.0 - y * 520.0);
                let net = (a.sin() * b.sin()).abs();
                c = mix(c, [0.0, 0.0, 0.0], (1.0 - sstep(0.0, 0.25, net)) * 0.08 * sstep(0.2, 0.4, s));
                match kind {
                    0 => {
                        // the pink band along the flank, the black specks
                        let band = 1.0 - sstep(0.12, 0.32, (q - 0.02).abs());
                        c = mix(c, [0.68, 0.44, 0.44], band * 0.7 * sstep(0.12, 0.3, s));
                        let (sp, _) = spots(seed ^ 1, x, y, 0.0055, 0.22, 0.7);
                        c = mix(c, [0.07, 0.06, 0.05], sp * 0.85 * sstep(-0.2, 0.3, q) * sstep(0.05, 0.15, s));
                    }
                    1 => {
                        // worm marks over the back, yellow spots, red spots
                        // ringed pale blue on the flank
                        let m = (fbm(seed ^ 3, x * 220.0, y * 220.0, 3) - 0.5).abs();
                        c = mix(c, [0.62, 0.60, 0.40], (1.0 - sstep(0.02, 0.06, m)) * sstep(0.2, 0.5, q) * 0.8);
                        let (sp, r) = spots(seed ^ 4, x, y, 0.009, 0.3, 0.7);
                        let fl = (1.0 - sstep(0.3, 0.6, q.abs())) * sstep(0.18, 0.28, s);
                        if r < 0.22 {
                            let (ring, _) = spots(seed ^ 4, x, y, 0.009, 0.5, 0.7);
                            c = mix(c, [0.46, 0.52, 0.62], ring * fl * 0.7);
                            c = mix(c, [0.74, 0.28, 0.20], sp * fl);
                        } else {
                            c = mix(c, [0.76, 0.68, 0.38], sp * fl * 0.8);
                        }
                        // the belly's white edge
                        c = mix(c, [0.88, 0.86, 0.80], 1.0 - sstep(-0.95, -0.82, q));
                    }
                    2 => {
                        // black spots over back and flank, red ones along
                        // the lateral line, each ringed pale
                        let (sp, r) = spots(seed ^ 5, x, y, 0.010, 0.3, 0.75);
                        let on = sstep(-0.45, -0.1, q) * sstep(0.08, 0.2, s);
                        let red = r < 0.3 && q.abs() < 0.35;
                        let (ring, _) = spots(seed ^ 5, x, y, 0.010, 0.45, 0.75);
                        c = mix(c, [0.86, 0.80, 0.64], ring * on * 0.6);
                        c = mix(c, if red { [0.66, 0.26, 0.16] } else { [0.08, 0.07, 0.05] }, sp * on);
                    }
                    _ => {
                        // pale spots all over
                        let (sp, _) = spots(seed ^ 2, x, y, 0.007, 0.32, 0.6);
                        c = mix(c, [0.76, 0.74, 0.62], sp * 0.75 * sstep(-0.6, -0.2, q) * sstep(0.05, 0.12, s));
                    }
                }
                // the head: darker crown, the gill cover's edge, the jaw
                if s < 0.25 {
                    c = mix(c, back, sstep(0.0, 0.6, q) * (1.0 - sstep(0.1, 0.25, s)) * 0.6);
                    if kind == 0 {
                        c = mix(c, [0.66, 0.46, 0.46], (1.0 - sstep(0.0, 0.5, (q + 0.15).abs())) * sstep(0.10, 0.16, s) * 0.6);
                    }
                    let edge = 0.205 + 0.032 * (1.0 - q.abs()).max(0.0).powi(2);
                    c = mix(c, [0.10, 0.08, 0.06], (1.0 - sstep(0.0, 0.006, (s - edge).abs())) * 0.6 * (1.0 - sstep(0.4, 0.7, q)));
                    let jaw = (q + 0.35 - s * 0.6).abs();
                    c = mix(c, [0.08, 0.06, 0.05], (1.0 - sstep(0.0, 0.08, jaw)) * (1.0 - sstep(0.08, 0.13, s)) * 0.8);
                }
                // the lateral line
                let ll = (q - 0.05 - 0.1 * (s - 0.5)).abs();
                c = mix(c, [0.20, 0.18, 0.14], (1.0 - sstep(0.0, 0.035, ll)) * sstep(0.22, 0.3, s) * 0.3);
            }
            // the lacquer gone amber, a little darker
            c = [c[0] * 0.92, c[1] * 0.88, c[2] * 0.74];
            // its cracks, and dust settled on the back and the fins' tops
            let cr = (vn(seed ^ 9, u * 70.0, v * 30.0) - 0.5).abs();
            c = mix(c, [0.12, 0.10, 0.08], (1.0 - sstep(0.0, 0.02, cr)) * 0.4);
            let d = vn(seed ^ 11, u * 300.0, v * 110.0);
            let up = sstep(0.4, 1.0, q.min(1.6));
            c = mix(c, [0.52, 0.50, 0.46], sstep(0.7, 0.95, d) * (0.15 + 0.2 * up) + 0.04 + 0.06 * up);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The pectoral and pelvic fins: rays along their length, its kind's colour
/// (a brook trout's red with the white leading edge and the black stripe;
/// a lake trout's white edge), the lacquer amber, dust
fn trout_fin_tex(salt: u32, kind: usize) -> Tex {
    canvas_tex_opt(96, 32, false, move |ctx, w, h| {
        let seed = salt ^ 0x0f1;
        let [back, flank, _] = TROUT[kind];
        fill(ctx, w, h, |u, v| {
            let ray = (v * 70.0 + u * 6.0).sin();
            let mut c = match kind {
                0 => [0.66, 0.56, 0.52],
                1 => [0.72, 0.38, 0.24],
                2 => mix(flank, [0.70, 0.56, 0.36], 0.5),
                _ => mix(flank, back, 0.3),
            };
            c = mix(c, [0.18, 0.14, 0.11], (0.5 - 0.5 * ray) * 0.3);
            if kind == 1 || kind == 3 {
                // the leading edge (the fin's top, as they are spread)
                let e = 1.0 - v;
                if kind == 1 {
                    c = mix(c, [0.08, 0.07, 0.06], sstep(0.12, 0.2, e) * (1.0 - sstep(0.3, 0.38, e)));
                }
                c = mix(c, [0.86, 0.84, 0.78], 1.0 - sstep(0.08, 0.16, e));
            }
            c = [c[0] * 0.92, c[1] * 0.88, c[2] * 0.74];
            let d = vn(seed, u * 120.0, v * 40.0);
            c = mix(c, [0.52, 0.50, 0.46], sstep(0.75, 0.95, d) * 0.4 + 0.05);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A taxidermist's glass fish eye (on the sphere's map, its front at u =
/// 0.25, v = 0.5): the black pupil, the gold iris round it, the eye's dark
/// back
fn fish_eye_tex() -> Tex {
    canvas_tex_opt(64, 32, false, move |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let (du, dv) = ((u - 0.25) * std::f64::consts::TAU, (v - 0.5) * std::f64::consts::PI);
            let a = (du * du + dv * dv).sqrt();
            let mut c = [0.16, 0.14, 0.11];
            c = mix(c, [0.62, 0.50, 0.26], 1.0 - sstep(0.95, 1.05, a));
            c = mix(c, [0.40, 0.30, 0.14], sstep(0.55, 0.9, a) * (1.0 - sstep(0.95, 1.05, a)) * 0.6);
            c = mix(c, [0.03, 0.03, 0.03], 1.0 - sstep(0.42, 0.48, a));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The painting kit of batch `t11`'s painters (as batches `t08`'s and
/// `t09`'s): value noise and a per-pixel fill (pure, so `par_rows` may run
/// it on every core), a salt from where a thing is, a part placed.
#[allow(dead_code)]
mod t11_kit {
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
