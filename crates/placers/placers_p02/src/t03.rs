//! Batch `t03`'s props in this crate, made real (`docs/PROPS.md` P3): the
//! washroom's glass shelf with the bottles left on it, and the shower cap
//! dropped on the floor; their placers and what only they use
//! (`assets/src/blender/props/t03.py`, `t03.notes.md`).
use super::*;
use game::assets::props::prop_part_as_modelled;

// R572 shampoo bottle shelf: 5% chance per level, 2-3 bottles. A bathroom
// shelf of 8 mm toughened glass, 500 x 120 mm, in two chromed clamp brackets
// (Blender batch `t03`), the bottles left standing on it: oval HDPE bottles
// of shampoo, conditioner and shower gel with their print, Boston rounds of
// herbal and egg shampoo with paper labels (no maker's name: invented ones),
// dried runs down them from their caps; the glass dusty, limed where it was
// splashed, rings where other bottles stood; the chrome pitted.
pub(super) fn place_shampoo_shelves(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let shelf_group = group();
    let px = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.060);
    let pz = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.060);
    let salt = t03_kit::salt(px, pz);
    // the shelf: modelled with the wall at z = 0 and the glass's top at y = 0
    let shelf = group();
    shelf.position().set(0, T03_SHELF_TOP, -0.060);
    shelf_group.add(&shelf);
    let put = |grp: &Obj, name: &str, mat: &Mat| -> Option<Obj> {
        let m = mesh(prop_part_as_modelled(name)?, mat);
        grp.add(&m);
        Some(m)
    };
    put(
        &shelf,
        "Real__shampooShelfBrackets",
        &phong!({ map: &t03_kit::chrome_tex(salt), color: 0x6a6c6e, specular: 0xb4b4b4, shininess: 70 }),
    );
    put(&shelf, "Real__shampooShelfGlassEdge", &phong!({ color: 0x2f4a40, specular: 0x5a5a5a, shininess: 80 }));
    // 2-3 bottles standing on the glass
    let bottle_count = 2 + (rnd() * 2.0).floor() as i32;
    let mut kinds = vec![];
    for b in 0..bottle_count {
        let b_color = pick(&T03_BOTTLE_COLORS);
        let kind = T03_BOTTLE_COLORS.iter().position(|&k| k == b_color).unwrap_or(0);
        let b_h = 0.14 + rnd() * 0.06;
        // the old height draw sizes the bottle a little (uniformly, 0.92-1.08)
        // and turns it on the shelf as it was put down
        let f = (b_h - 0.14) / 0.06;
        let bottle = t03_bottle(kind, salt.wrapping_add(b as u32 * 0x9e37_79b9));
        bottle.scale().set_scalar(0.92 + 0.16 * f);
        bottle.rotation().y = (f - 0.5) * 0.5;
        let bx = -0.15 + b as f64 * 0.15;
        bottle.position().set(bx, T03_SHELF_TOP, 0);
        shelf_group.add(&bottle);
        kinds.push(kind);
    }
    // the glass, its rings where these bottles stand and where others stood
    if let Some(glass) = put(&shelf, "Real__shampooShelfGlass", &t03_kit::glass_mat(&t03_kit::shelf_glass_tex(salt, kinds))) {
        glass.set_render_order(2);
    }
    shelf_group.rotation().y = face_wall_yaw(c.dx, c.dz);
    shelf_group.position().set(px, 0, pz);
    g.world.add(&shelf_group);
}

/// The glass's top, as the old plank's (1.20 m, 2 cm thick)
const T03_SHELF_TOP: f64 = 1.21;
/// The old colour pool: which bottle each colour is (`t03_bottle`)
const T03_BOTTLE_COLORS: [u32; 5] = [0x40a0c0, 0xe08080, 0x80c060, 0xf0d040, 0xd050a0];

/// One bottle of `kind` standing at its foot's centre: the old colours were a
/// pale blue anti-dandruff shampoo, a rose conditioner (oval HDPE), a green
/// herbal shampoo, an amber egg shampoo (Boston round PET), a plum shower gel
/// (oval)
fn t03_bottle(kind: usize, salt: u32) -> Obj {
    let grp = group();
    let round = kind == 2 || kind == 3;
    let (body_col, cap_col, spec, shine) = [
        (0x9fb6be, 0xd8d5cb, 0x2a2a2a, 30),
        (0xc49a96, 0xa4706f, 0x2a2a2a, 30),
        (0x3d6a3c, 0xdcd8cc, 0x6a6a6a, 70),
        (0x9a6420, 0x1d1c1c, 0x6a6a6a, 70),
        (0x76506a, 0xa8a8a6, 0x2a2a2a, 30),
    ][kind.min(4)];
    let (body, label, cap) = if round {
        ("Real__shampooRoundBody", "Real__shampooRoundLabel", "Real__shampooRoundCap")
    } else {
        ("Real__shampooOvalBody", "Real__shampooOvalLabel", "Real__shampooOvalCap")
    };
    let grime = t03_kit::bottle_grime_tex(salt, round);
    if let Some(geo) = prop_part_as_modelled(body) {
        grp.add(&mesh(geo, phong!({ map: &grime, color: body_col, specular: spec, shininess: shine })));
    }
    if let Some(geo) = prop_part_as_modelled(cap) {
        grp.add(&mesh(geo, phong!({ map: &grime, color: cap_col, specular: 0x303030, shininess: 40 })));
    }
    if let Some(geo) = prop_part_as_modelled(label) {
        let lab = mesh(
            geo,
            lambert!({ map: &t03_kit::label_tex(kind, salt), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
        );
        if round {
            // the wrap's print is painted round its middle (u = 0.5): to the front
            lab.rotation().y = PI;
        }
        grp.add(&lab);
    }
    grp
}

// R578 shower cap: 6% chance per level. A reusable PEVA shower cap (Blender
// batch `t03`) dropped on the floor opening down: the crown fallen in on
// itself in creases, spilled over its gathered elastic in two lobes, the
// frill flopped out round it; printed (small flowers, dots) or plain, or a
// hotel's clear disposable one, its old colour; yellowed, dust in it, mildew
// spotting where it stayed damp.
pub(super) fn place_shower_caps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let cap_colors = [0xffffff, 0xf0d0e0, 0x8080e0, 0xf0f0a0, 0xe0e0e0];
    let col = pick(&cap_colors);
    let kind = cap_colors.iter().position(|&k| k == col).unwrap_or(0);
    let cap = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let salt = t03_kit::salt(px, pz);
    if let Some(geo) = prop_part_as_modelled("Real__showerCapFilm") {
        let tex = t03_kit::cap_print_tex(kind, salt);
        let mat = if kind == 4 {
            // clear polyethylene, gone milky
            phong!({ map: &tex, color: 0xdadcd6, specular: 0x404040, shininess: 50, transparent: true, opacity: 0.72, side: DoubleSide })
        } else {
            // PEVA's satin
            phong!({ map: &tex, color: 0xffffff, specular: 0x242424, shininess: 22, side: DoubleSide })
        };
        cap.add(&mesh(geo, &mat));
    }
    cap.position().set(px, 0, pz);
    cap.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cap);
}

/// The shelf's and the cap's painters and materials
mod t03_kit {
    use super::*;

    /// A placed thing's own number, from where it stands (no draw): its
    /// painters start their strokes from it, so no two floors share them
    pub(super) fn salt(px: f64, pz: f64) -> u32 {
        ((px * 1013.0).floor() as i64 ^ ((pz * 733.0).floor() as i64).rotate_left(17)) as u32
    }

    /// Its own stream starts the same on every floor: the salt moves where
    fn skip(salt: u32) {
        for _ in 0..(salt % 61) {
            let _ = rnd();
        }
    }

    // ---- painting kit (per-pixel, pure: safe for `par_rows`) ----
    fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right,
    /// v up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
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
    /// Rubs a painted print: specks and smears taken out of it where hands
    /// and water went (its own stream)
    fn rub(ctx: &mut Canvas, w: f64, h: f64, n: usize, size: f64, edge_bias: bool) {
        ctx.set_global_composite_operation("destination-out");
        for _ in 0..n {
            let (mut x, y) = (rnd() * w, rnd() * h);
            if edge_bias && rnd() < 0.5 {
                x = if rnd() < 0.5 { rnd() * w * 0.15 } else { w - rnd() * w * 0.15 };
            }
            ctx.set_global_alpha(0.12 + 0.45 * rnd());
            ctx.begin_path();
            ctx.ellipse(x, y, size * (0.3 + rnd()), size * (0.15 + 0.4 * rnd()), rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        ctx.set_global_alpha(1.0);
        ctx.set_global_composite_operation("source-over");
    }

    /// Clear glass as the windows' (`Patch::Glass`): the room seen in it by
    /// Fresnel, its dirt from the map's grey
    pub(super) fn glass_mat(map: &Tex) -> Mat {
        let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 220, transparent: true, depthWrite: false });
        {
            let mut mm = m.m();
            mm.map = Some(map.clone());
            mm.patch = Patch::Glass;
            mm.look_settled = true;
            mm.name = "shampoo_shelf_glass".into();
        }
        {
            let pu = m.r().patch_uniforms.clone();
            let f = &mut pu.borrow_mut().f;
            f[0] = 0.40;
            f[4] = 0.5;
            f[5] = 0.6;
            f[6] = 0.3;
            f[7] = 0.0;
            f[8] = 1.0;
            f[9] = 0.21;
            f[10] = 0.20;
            f[11] = 0.18;
            f[12] = 0.0;
            f[13] = 0.07;
        }
        m
    }

    /// The shelf's glass seen from above (u along it, v from its front to the
    /// wall): dust, lime where it was splashed, rings where bottles stood
    /// (where the ones left stand, and two that were taken)
    pub(super) fn shelf_glass_tex(salt: u32, kinds: Vec<usize>) -> Tex {
        canvas_tex_opt(256, 64, false, move |ctx, w, h| {
            let seed = salt ^ 0x5e1f;
            // the rings, in metres on the glass (x along it, z out from the
            // wall): each bottle's foot (an oval's 76 x 45 mm, a round's 58)
            let mut rings: Vec<(f64, f64, f64, f64)> = kinds
                .iter()
                .enumerate()
                .map(|(b, &k)| {
                    let (a, c) = if k == 2 || k == 3 { (0.029, 0.029) } else { (0.037, 0.022) };
                    (-0.15 + 0.15 * b as f64, 0.06, a, c)
                })
                .collect();
            rings.push((-0.21 + 0.03 * hash(seed, 1, 0), 0.05 + 0.02 * hash(seed, 2, 0), 0.029, 0.029));
            rings.push((0.19 + 0.04 * hash(seed, 3, 0), 0.055 + 0.02 * hash(seed, 4, 0), 0.037, 0.022));
            fill(ctx, w, h, |u, v| {
                // the glass's top: u along its 0.5 m, v from its front (0) to
                // the wall (1), its 0.12 m
                let (x, z) = (u * 0.5 - 0.25, 0.01 + (1.0 - v) * 0.12);
                // dust thicker toward the wall, less where hands reached
                let d = fbm(seed, u * 14.0, v * 4.0, 4);
                let mut dirt = 0.16 + 0.30 * d + 0.18 * v;
                // lime: splashes dried white in blotches
                dirt += 0.35 * sstep(0.62, 0.78, fbm(seed ^ 9, u * 22.0, v * 6.0, 3));
                for &(rx, rz, ra, rc) in &rings {
                    let r = (((x - rx) / ra).powi(2) + ((z - rz) / rc).powi(2)).sqrt();
                    // a dried ring at its foot's edge, cleaner inside it
                    dirt += 0.45 * (-((r - 1.0) / 0.08).powi(2)).exp() - 0.14 * sstep(1.0, 0.75, r);
                }
                let g = dirt.clamp(0.0, 1.0);
                [g, g, g, 1.0]
            });
        })
    }

    /// Chrome on zinc, years in a bathroom's damp: dulled, pitted, white
    /// bloom and a little rust where the plating went (a grey to multiply by)
    pub(super) fn chrome_tex(salt: u32) -> Tex {
        canvas_tex_opt(64, 64, false, move |ctx, w, h| {
            let seed = salt ^ 0xc407;
            fill(ctx, w, h, |u, v| {
                let n = fbm(seed, u * 6.0, v * 6.0, 3);
                let pit = (vn(seed ^ 3, u * 48.0, v * 48.0) - 0.78).max(0.0) * 3.5;
                let bloom = sstep(0.66, 0.8, fbm(seed ^ 5, u * 10.0, v * 10.0, 2)) * 0.35;
                let g = 0.80 + 0.15 * n - pit + bloom;
                let rust = sstep(0.75, 0.85, fbm(seed ^ 7, u * 9.0, v * 9.0, 2));
                [g * (1.0 + 0.10 * rust), g * (1.0 - 0.06 * rust), g * (1.0 - 0.22 * rust), 1.0]
            });
        })
    }

    /// A bottle's plastic: mottled, a scum ring dried at its foot, dried runs
    /// of what it held down its front from the cap (a grey to multiply by)
    pub(super) fn bottle_grime_tex(salt: u32, round: bool) -> Tex {
        canvas_tex_opt(64, 128, false, move |ctx, w, h| {
            let seed = salt ^ 0xb0771e;
            let runs: Vec<(f64, f64, f64)> =
                (0..3).map(|k| (0.25 + 0.5 * hash(seed, k, 1), 0.35 + 0.35 * hash(seed, k, 2), 0.010 + 0.012 * hash(seed, k, 3))).collect();
            let top = if round { 0.80 } else { 0.85 };
            fill(ctx, w, h, |u, v| {
                let n = fbm(seed, u * 5.0, v * 9.0, 3);
                let mut g = 0.80 + 0.14 * n;
                // the scum at the foot, its tide line
                g -= 0.22 * sstep(0.10, 0.02, v) + 0.10 * (-((v - 0.095) / 0.012).powi(2)).exp();
                // runs from the shoulder, drying thinner as they went
                for &(ru, end, wd) in &runs {
                    if v < top && v > end {
                        let t = (top - v) / (top - end);
                        let wob = 0.01 * (v * 40.0 + ru * 9.0).sin();
                        let d = ((u - ru - wob).abs() / (wd * (1.0 - 0.5 * t))).min(2.0);
                        g -= 0.12 * (1.0 - d).max(0.0) * (1.0 - 0.4 * t);
                    }
                }
                // dust on what faced up
                g += 0.10 * sstep(top - 0.05, top + 0.08, v) * fbm(seed ^ 2, u * 20.0, v * 20.0, 2);
                [g, g * 0.99, g * 0.96, 1.0]
            });
        })
    }

    /// The print of each bottle (no maker's name: invented ones), rubbed
    /// where hands held it and gone where water stood on it. The ovals'
    /// fronts are printed on the plastic (the rest of the map clear); the
    /// rounds' paper labels go round them, their print round u = 0.5
    pub(super) fn label_tex(kind: usize, salt: u32) -> Tex {
        if kind == 2 || kind == 3 {
            canvas_tex_opt(368, 140, false, move |ctx, w, h| {
                skip(salt);
                ctx.clear_rect(0, 0, w, h);
                // the paper: the front three fifths, its edges a little ragged
                let (x0, x1) = (w * 0.2, w * 0.8);
                let (paper, ink, band) = if kind == 2 { ("#e4e0cc", "#2f4a2a", "#5f7f45") } else { ("#ece2c2", "#4a2a12", "#b07a22") };
                ctx.set_fill_style(paper);
                ctx.fill_rect(x0, 2, x1 - x0, h - 4.0);
                ctx.set_fill_style(band);
                ctx.fill_rect(x0, 12, x1 - x0, 6);
                ctx.fill_rect(x0, h - 20.0, x1 - x0, 6);
                ctx.set_fill_style(ink);
                ctx.set_text_align("center");
                // the front: the middle third (what faces you of a round)
                let (cx, mw) = (w / 2.0, w * 0.32);
                if kind == 2 {
                    ctx.set_font("bold 30px Gill Sans");
                    ctx.fill_text_max("Meadow", cx, 56.0, mw);
                    ctx.set_font("13px Gill Sans");
                    ctx.fill_text_max("HERBAL SHAMPOO", cx, 76.0, mw);
                    ctx.set_font("10px Gill Sans");
                    ctx.fill_text_max("camomile \u{b7} nettle \u{b7} rosemary", cx, 92.0, mw);
                    ctx.fill_text_max("for normal hair", cx, 104.0, mw);
                } else {
                    ctx.set_font("bold 21px Copperplate");
                    ctx.fill_text_max("BRIGHTWELL", cx, 52.0, mw);
                    ctx.set_font("italic 15px Times New Roman");
                    ctx.fill_text_max("egg & lemon shampoo", cx, 74.0, mw);
                    ctx.set_font("10px Times New Roman");
                    ctx.fill_text_max("for dull and lifeless hair", cx, 92.0, mw);
                    ctx.fill_text_max("with real egg protein", cx, 104.0, mw);
                }
                ctx.set_font("bold 11px Helvetica");
                ctx.fill_text_max("250 ml \u{212e}", cx, h - 26.0, 80.0);
                // the sides' small print: what is in it, and how to use it
                ctx.set_font("6px Helvetica");
                ctx.set_text_align("left");
                let small = [
                    "Ingredients: Aqua, Sodium",
                    "Laureth Sulfate, Cocamide",
                    "DEA, Sodium Chloride,",
                    "Parfum, Citric Acid,",
                    "Methylparaben, CI 19140.",
                ];
                for (i, l) in small.iter().enumerate() {
                    ctx.fill_text_max(l, x0 + 5.0, 34.0 + i as f64 * 8.0, w * 0.16 - 8.0);
                }
                let how = ["Wet hair, massage", "into a lather, rinse.", "Repeat if needed.", "Avoid the eyes.", "Store below 25 \u{b0}C."];
                for (i, l) in how.iter().enumerate() {
                    ctx.fill_text_max(l, x1 - w * 0.16 + 3.0, 34.0 + i as f64 * 8.0, w * 0.16 - 8.0);
                }
                // the paper gone: water at its foot, a corner torn, rubbed
                ctx.set_global_composite_operation("destination-out");
                ctx.begin_path();
                ctx.move_to(x1 - 18.0 - rnd() * 10.0, h);
                ctx.line_to(x1 + 1.0, h - 14.0 - rnd() * 16.0);
                ctx.line_to(x1 + 1.0, h);
                ctx.fill();
                ctx.set_global_composite_operation("source-over");
                rub(ctx, w, h, 50, 7.0, true);
                // a tide where it got wet, brown at its line
                ctx.set_global_alpha(0.35);
                ctx.set_stroke_style("rgba(120,90,50,0.8)");
                ctx.set_line_width(1.5);
                ctx.begin_path();
                let ty = h - 30.0 - rnd() * 20.0;
                ctx.move_to(x0, ty);
                let mut xx = x0;
                while xx < x1 {
                    xx += 8.0;
                    ctx.line_to(xx, ty + (rnd() - 0.5) * 6.0);
                }
                ctx.stroke();
                ctx.set_global_alpha(1.0);
            })
        } else {
            canvas_tex_opt(112, 220, false, move |ctx, w, h| {
                skip(salt);
                ctx.clear_rect(0, 0, w, h);
                let cx = w / 2.0;
                ctx.set_text_align("center");
                match kind {
                    0 => {
                        ctx.set_fill_style("#1e3550");
                        ctx.set_font("bold 25px Futura");
                        ctx.fill_text_max("clearwell", cx, 42.0, w - 10.0);
                        ctx.fill_rect(14, 52, w - 28.0, 2);
                        ctx.set_font("bold 12px Helvetica");
                        ctx.fill_text_max("ANTI-DANDRUFF", cx, 74.0, w - 10.0);
                        ctx.set_font("bold 17px Helvetica");
                        ctx.fill_text_max("SHAMPOO", cx, 93.0, w - 10.0);
                        ctx.set_font("10px Helvetica");
                        ctx.fill_text_max("with zinc pyrithione", cx, 110.0, w - 10.0);
                        ctx.fill_text_max("normal to greasy hair", cx, 123.0, w - 10.0);
                    }
                    1 => {
                        ctx.set_fill_style("#4a1e26");
                        ctx.set_font("30px Optima");
                        ctx.fill_text_max("Linden", cx, 44.0, w - 10.0);
                        ctx.set_font("italic 13px Optima");
                        ctx.fill_text_max("conditioner", cx, 64.0, w - 10.0);
                        ctx.set_font("10px Optima");
                        ctx.fill_text_max("rose & almond", cx, 86.0, w - 10.0);
                        ctx.fill_text_max("for dry, coloured hair", cx, 99.0, w - 10.0);
                        ctx.begin_path();
                        ctx.arc(cx, 120.0, 6.0, 0, 7);
                        ctx.set_stroke_style("#4a1e26");
                        ctx.set_line_width(1.2);
                        ctx.stroke();
                    }
                    _ => {
                        ctx.set_fill_style("#e6dee4");
                        ctx.set_font("italic 28px Optima");
                        ctx.fill_text_max("Velour", cx, 46.0, w - 10.0);
                        ctx.set_font("bold 11px Helvetica");
                        ctx.fill_text_max("BATH & SHOWER", cx, 70.0, w - 10.0);
                        ctx.set_font("bold 15px Helvetica");
                        ctx.fill_text_max("CREME GEL", cx, 87.0, w - 10.0);
                        ctx.set_font("10px Helvetica");
                        ctx.fill_text_max("black orchid", cx, 104.0, w - 10.0);
                    }
                }
                ctx.set_font("bold 11px Helvetica");
                ctx.fill_text_max("400 ml \u{212e}", cx, h - 14.0, w - 10.0);
                ctx.set_font("6px Helvetica");
                let small = [
                    "Aqua, Sodium Laureth Sulfate,",
                    "Cocamidopropyl Betaine, Glycerin,",
                    "Parfum, Sodium Chloride, Citric",
                    "Acid, Sodium Benzoate. Rinse off.",
                ];
                for (i, l) in small.iter().enumerate() {
                    ctx.fill_text_max(l, cx, h - 58.0 + i as f64 * 8.0, w - 8.0);
                }
                rub(ctx, w, h, 60, 6.0, true);
            })
        }
    }

    /// The cap's PEVA seen from above: its print (or plain; or the clear
    /// film's crinkle), yellowed, dust in it, mildew spotting in clusters
    pub(super) fn cap_print_tex(kind: usize, salt: u32) -> Tex {
        canvas_tex_opt(256, 256, false, move |ctx, w, h| {
            skip(salt);
            let base = ["#ddd8cc", "#d6b4bc", "#8e8cb0", "#ddd6a6", "#cfd2cc"][kind.min(4)];
            ctx.set_fill_style(base);
            ctx.fill_rect(0, 0, w, h);
            match kind {
                0 => {
                    // small faded blue flowers, scattered
                    for _ in 0..90 {
                        let (x, y) = (rnd() * w, rnd() * h);
                        ctx.set_fill_style("rgba(92,112,150,0.75)");
                        for p in 0..5 {
                            let a = p as f64 * 1.2566 + rnd() * 0.2;
                            ctx.begin_path();
                            ctx.ellipse(x + a.cos() * 2.6, y + a.sin() * 2.6, 2.2, 1.4, a, 0, 7);
                            ctx.fill();
                        }
                        ctx.set_fill_style("rgba(190,160,80,0.8)");
                        ctx.begin_path();
                        ctx.arc(x, y, 1.1, 0, 7);
                        ctx.fill();
                    }
                }
                1 => {
                    // white polka dots on a half-drop grid
                    ctx.set_fill_style("rgba(238,232,226,0.9)");
                    let s = 18.0;
                    let mut j = 0;
                    while (j as f64) * s < h + s {
                        let mut i = 0;
                        while (i as f64) * s < w + s {
                            let x = i as f64 * s + if j % 2 == 1 { s / 2.0 } else { 0.0 };
                            ctx.begin_path();
                            ctx.arc(x, j as f64 * s, 3.4, 0, 7);
                            ctx.fill();
                            i += 1;
                        }
                        j += 1;
                    }
                }
                3 => {
                    // small daisies
                    for _ in 0..70 {
                        let (x, y) = (rnd() * w, rnd() * h);
                        ctx.set_fill_style("rgba(245,242,228,0.85)");
                        for p in 0..8 {
                            let a = p as f64 * 0.785;
                            ctx.begin_path();
                            ctx.ellipse(x + a.cos() * 3.0, y + a.sin() * 3.0, 2.4, 0.9, a, 0, 7);
                            ctx.fill();
                        }
                        ctx.set_fill_style("rgba(200,150,50,0.9)");
                        ctx.begin_path();
                        ctx.arc(x, y, 1.4, 0, 7);
                        ctx.fill();
                    }
                }
                _ => {}
            }
            // the wear, pixel by pixel over the print
            let seed = salt ^ 0x5ca9;
            let mut img = ctx.get_image_data(0, 0, w, h);
            let iw = w as usize;
            let clear = kind == 4;
            par_rows(&mut img.data, iw * 4, |j, out| {
                for i in 0..iw {
                    let (u, v) = ((i as f64 + 0.5) / w, (j as f64 + 0.5) / h);
                    let n = fbm(seed, u * 8.0, v * 8.0, 4);
                    // yellowed and dirty in blotches, dust settled
                    let dirt = 0.06 + 0.18 * sstep(0.45, 0.8, n);
                    // mildew: black specks in clusters
                    let clus = sstep(0.62, 0.78, fbm(seed ^ 3, u * 5.0, v * 5.0, 2));
                    let speck = sstep(0.80, 0.90, vn(seed ^ 4, u * 120.0, v * 120.0)) * clus;
                    // the clear film's crinkle catches the light in lines
                    let crink =
                        if clear { sstep(0.85, 1.0, 1.0 - (fbm(seed ^ 6, u * 14.0, v * 14.0, 3) - 0.5).abs() * 2.0) * 0.25 } else { 0.0 };
                    for k in 0..3 {
                        let c = f64::from(out[i * 4 + k]) / 255.0;
                        let tint = [0.92, 0.86, 0.70][k];
                        let mut c2 = c * (1.0 - dirt) + c * tint * dirt * 0.85;
                        c2 = c2 * (1.0 - 0.85 * speck) + 0.05 * speck;
                        c2 += crink;
                        out[i * 4 + k] = (c2.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
            });
            ctx.put_image_data(&img, 0, 0);
        })
    }
}
