//! Batch `t20`'s props in this crate, made real (`docs/PROPS.md` P3; the
//! Blender batch `assets/src/blender/props/t20.py`, its notes
//! `t20.notes.md`): the humidor with its cigars, and the hunter-case pocket
//! watch on its chain. Each keeps the old placer's draws, call for call;
//! the spilled cigars still rock where the old ones did, the watch keeps
//! its place (`pocket_watch_positions_r755`, its ticking). The old watch
//! cases glowed a little (an emissive): these are metal, lit by the room.
use super::*;
use game::visuals::pictures;
use t20_kit::*;

/// The humidor (t20.py `HU_*`): its outside, the case's height, the walls,
/// the floor inside, the lid's opening (radians); a cigar's length and radius
const HU_W: f64 = 0.260;
const HU_D: f64 = 0.190;
const HU_HB: f64 = 0.072;
const HU_T: f64 = 0.009;
const HU_FLOOR: f64 = 0.008;
const HU_LIN: f64 = 0.003;
const HU_OPEN: f64 = 1.72;
const CG_L: f64 = 0.140;
const CG_R: f64 = 0.00825;

// R746 humidor cigar box w/ scattered cigars (floor): 3% chance per level.
// A Victorian humidor (Blender batch `t20`): a veneered case on its plinth
// (mahogany, burr walnut, rosewood or ebonised: the old colour's draw),
// lined with Spanish cedar or zinc (the old lining's draw), the lid open
// on its brass hinges, a brass escutcheon and name plate; Coronas lying in
// it and spilled on the floor, their bands printed, a few rocking where
// they fell; now and then a steel cutter, a paper matchbook. The veneer's
// polish gone dull and crazed, the cedar pale, the cigars dried.
pub(super) fn place_humidors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // the old draws, call for call: the box's and the lining's colours
    let box_kind = ri(4);
    let lining_kind = ri(3);
    let cigar_count = 4 + (rnd() * 3.0).floor() as i32;
    struct Cigar {
        leaf: usize,
        scale: f64,
        dry: f64,
        yaw: f64,
        at: (f64, f64),
        inside: bool,
        band: usize,
        rock: Option<(f64, f64, f64)>,
    }
    let mut cigars: Vec<Cigar> = vec![];
    for i in 0..cigar_count {
        let leaf = ri(5);
        let len = 0.11 + rnd() * 0.03;
        let dry = rnd(); // the old radius's draw: how dried this one is
        let yaw = (rnd() - 0.5) * 1.2;
        let inside = i < 3;
        let at = (rnd(), rnd());
        let band = ri(4);
        let rock = if !inside && rnd() < 0.5 {
            let phase = rnd() * PI * 2.0;
            let freq = 0.35 + rnd() * 0.4;
            let amp = 0.05 + rnd() * 0.05;
            Some((phase, freq, amp))
        } else {
            None
        };
        // the old length's draw scales the whole cigar a little (0.92-1.06)
        cigars.push(Cigar { leaf, scale: 0.92 + (len - 0.11) / 0.03 * 0.14, dry, yaw, at, inside, band, rock });
    }
    let cutter = rnd() < 0.4;
    let matchbook = if rnd() < 0.3 { Some((rnd() - 0.5) * 0.5) } else { None };
    let px = g.wx(x) + (rnd() - 0.5) * 1.6;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.6;
    let turn = rnd() * PI * 2.0;
    let hm = group();
    hm.position().set(px, 0, pz);
    hm.rotation().y = turn;
    let vary = vary(px, pz, turn);
    let veneer = phong!({ map: &hu_veneer_tex(vary, box_kind), specular: 0x1e1a16, shininess: 22 });
    put(&hm, "Real__humidorBox", &veneer);
    let lining = if lining_kind == 1 {
        phong!({ map: &hu_zinc_tex(vary), specular: 0x2a2a2a, shininess: 18 })
    } else {
        lambert!({ map: &hu_cedar_tex(vary, lining_kind == 2) })
    };
    put(&hm, "Real__humidorLining", &lining);
    let brass = phong!({ color: 0x6a5530, specular: 0x4a3c22, shininess: 32 });
    put(&hm, "Real__humidorBrass", &brass);
    // the lid open on its hinges
    let lid = group();
    lid.position().set(0, HU_HB, -HU_D * 0.5);
    lid.rotation().x = -HU_OPEN;
    hm.add(&lid);
    put(&lid, "Real__humidorLid", &veneer);
    put(&lid, "Real__humidorLidLining", &lining);
    put(&lid, "Real__humidorLidBrass", &brass);
    // the cigars: three lying in it (each on the one under it where they
    // cross), the rest spilled on the floor beside it
    let mut laid: Vec<(f64, f64, f64, f64)> = vec![];
    let mut rocking: Vec<(Obj, f64, f64, f64, f64)> = vec![];
    for c in &cigars {
        let half = CG_L * 0.5 * c.scale;
        let (ex, ez) = (c.yaw.cos().abs() * half + CG_R, c.yaw.sin().abs() * half + CG_R);
        let (cx, cz, mut y) = if c.inside {
            let (iw, id) = (HU_W * 0.5 - HU_T - HU_LIN - 0.002, HU_D * 0.5 - HU_T - HU_LIN - 0.002);
            let cx = ((c.at.0 - 0.5) * (HU_W - 0.030)).clamp(-(iw - ex).max(0.0), (iw - ex).max(0.0));
            let cz = ((c.at.1 - 0.5) * (HU_D - 0.030)).clamp(-(id - ez).max(0.0), (id - ez).max(0.0));
            (cx, cz, HU_FLOOR + 0.002 + CG_R * c.scale * 1.015 + 0.0008)
        } else {
            // on the floor: its radius and its slight bend under it
            (HU_W * 0.5 + 0.02 + half * 0.86 + c.at.0 * 0.10, (c.at.1 - 0.5) * 0.12, CG_R * c.scale * 1.015 + 0.0008)
        };
        // on the one under it where they cross
        for &(ox, oz, oy, oyaw) in &laid {
            if (oy - y).abs() < 0.001 && hu_cross((cx, cz, c.yaw, half), (ox, oz, oyaw, half)) {
                y = oy + 2.0 * CG_R * c.scale;
            }
        }
        laid.push((cx, cz, y, c.yaw));
        let cg = group();
        cg.position().set(cx, y, cz);
        cg.rotation().y = c.yaw;
        let body = group();
        body.rotation().z = -PI / 2.0;
        body.scale().set(c.scale, c.scale, c.scale);
        cg.add(&body);
        put(&body, "Real__humidorCigar", &lambert!({ map: &hu_leaf_tex(c.leaf, c.dry) }));
        put(&body, "Real__humidorCigarBand", &lambert!({ map: &hu_band_tex(c.band) }));
        put(&body, "Real__humidorCigarFoot", &lambert!({ map: &hu_filler_tex() }));
        hm.add(&cg);
        if let Some((phase, freq, amp)) = c.rock {
            rocking.push((cg.clone(), c.yaw, phase, freq, amp));
        }
    }
    // R747 cigar rocking animator: the spilled ones still rock where they fell
    if !rocking.is_empty() {
        g.animate(move |_g, _dt, t| {
            for (cg, base, phase, freq, amp) in &rocking {
                cg.rotation().y = base + (t * freq + phase).sin() * amp;
            }
        });
    }
    // a steel guillotine cutter beside it
    if cutter {
        let ct = group();
        ct.position().set(-HU_W * 0.5 - 0.045, 0, 0.05);
        ct.rotation().y = PI * 0.5 + (hash(vary, 3, 1) - 0.5) * 0.8;
        put(&ct, "Real__humidorCutter", &phong!({ map: &hu_steel_tex(vary), specular: 0x3a3a3a, shininess: 40 }));
        hm.add(&ct);
    }
    // a paper matchbook, turned as the old one's draw
    if let Some(r) = matchbook {
        let mb = group();
        mb.position().set(HU_W * 0.5 + 0.035, 0, -HU_D * 0.5 - 0.035);
        mb.rotation().y = r;
        put(&mb, "Real__humidorMatchbook", &lambert!({ map: &hu_matchbook_tex(vary) }));
        hm.add(&mb);
    }
    g.world.add(&hm);
}

/// Do two cigars (middle x, z, turn, half length) cross, seen from above?
fn hu_cross(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    let seg = |c: (f64, f64, f64, f64)| ((c.0 - c.2.cos() * c.3, c.1 + c.2.sin() * c.3), (c.0 + c.2.cos() * c.3, c.1 - c.2.sin() * c.3));
    let (p, q) = seg(a);
    let (r, s) = seg(b);
    // the closest the two segments come, sampled
    let mut best = f64::MAX;
    for i in 0..=16 {
        let t = i as f64 / 16.0;
        let (x, z) = (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t);
        for j in 0..=16 {
            let u = j as f64 / 16.0;
            let (x2, z2) = (r.0 + (s.0 - r.0) * u, r.1 + (s.1 - r.1) * u);
            best = best.min(((x - x2).powi(2) + (z - z2).powi(2)).sqrt());
        }
    }
    best < 2.0 * CG_R
}

/// The case's veneer as its faces are mapped: mahogany's ribbon figure,
/// burr walnut's knots and swirls, rosewood's dark streaks, or ebonised;
/// the French polish gone dull and crazed, rubbed through on its edges,
/// dust on its top
fn hu_veneer_tex(vary: u32, kind: usize) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0x7e0 ^ kind as u32;
        fill(ctx, w, h, |u, v| {
            let mut o = match kind {
                0 => {
                    let ribbon = ((u * 70.0 + fbm(seed, u * 3.0, v * 12.0, 3) * 5.0).sin() * 0.5 + 0.5).powf(2.0);
                    [0.33, 0.15, 0.08].map(|c| c * (0.85 + 0.25 * ribbon))
                }
                1 => {
                    let burl = fbm(seed, u * 14.0, v * 14.0, 4);
                    let eye = ss(0.72, 0.8, vn(seed ^ 3, u * 30.0, v * 30.0));
                    [0.40, 0.26, 0.14].map(|c| c * (0.7 + 0.6 * burl) * (1.0 - 0.5 * eye))
                }
                2 => wood(seed, u, v, [0.17, 0.08, 0.05], 8.0, 60.0),
                _ => {
                    let g = fbm(seed, u * 8.0, v * 40.0, 3);
                    [0.05, 0.045, 0.04].map(|c| c * (0.85 + 0.3 * g))
                }
            };
            let d = crack(seed ^ 5, u * 90.0, v * 90.0);
            o = o.map(|c| c * (1.0 - 0.2 * (1.0 - ss(0.0, 0.04, d))));
            let rub = ss(0.65, 0.85, fbm(seed ^ 7, u * 10.0, v * 10.0, 3)) * (1.0 - ss(0.05, 0.15, u.min(1.0 - u).min(v).min(1.0 - v)));
            o = mix(o, [0.30, 0.22, 0.15], rub * 0.4);
            let dust = 0.08 + 0.12 * fbm(seed ^ 9, u * 7.0, v * 7.0, 3);
            opaque(mix(o, [0.43, 0.40, 0.35], dust))
        });
    })
}

/// Spanish cedar (fresh-cut pink-tan, or darkened by the years and the
/// cigars' oils): its straight grain, stained where the cigars lay
fn hu_cedar_tex(vary: u32, dark: bool) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0xced;
        let base = if dark { [0.34, 0.20, 0.12] } else { [0.56, 0.36, 0.24] };
        fill(ctx, w, h, |u, v| {
            let o = wood(seed, u, v, base, 10.0, 50.0);
            let stain = ss(0.6, 0.8, fbm(seed ^ 3, u * 5.0, v * 9.0, 3));
            opaque(o.map(|c| c * (1.0 - 0.18 * stain)))
        });
    })
}

/// A zinc liner gone dull grey, its solder seams, white bloom
fn hu_zinc_tex(vary: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = vary ^ 0x21c;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 6.0, v * 6.0, 3);
            let bloom = ss(0.62, 0.8, fbm(seed ^ 3, u * 10.0, v * 10.0, 3));
            opaque(mix([0.38, 0.39, 0.40].map(|c| c * (0.8 + 0.3 * n)), [0.62, 0.62, 0.60], bloom * 0.4))
        });
    })
}

/// A cigar's wrapper (u round it, v along it, foot to head), in one of the
/// old five shades (claro to oscuro), dried (`dry`: paler, its veins
/// standing): the leaf's veins across it, the cap darker; shared by every
/// cigar of a shade and dryness
fn hu_leaf_tex(kind: usize, dry: f64) -> Tex {
    let tone = [[0.36, 0.22, 0.12], [0.25, 0.14, 0.08], [0.30, 0.18, 0.09], [0.44, 0.30, 0.17], [0.20, 0.10, 0.06]][kind];
    let dry_k = (dry * 3.0).floor() as u32;
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = 0x1ea + kind as u32 * 17 + dry_k;
        let d = 0.15 * dry_k as f64;
        fill(ctx, w, h, |u, v| {
            let vein = ss(0.9, 0.98, (u * 9.0 + v * 22.0 + fbm(seed, u * 4.0, v * 8.0, 2) * 2.0).sin() * 0.5 + 0.5) * (0.25 + d);
            let n = fbm(seed ^ 3, u * 8.0, v * 24.0, 3);
            let mut o = tone.map(|c| c * (0.8 + 0.35 * n) * (1.0 - vein));
            o = o.map(|c| c * (1.0 - 0.25 * ss(0.92, 0.99, v)));
            opaque(mix(o, [0.46, 0.40, 0.32], 0.06 + d * 0.5))
        });
    })
}

/// A cigar band (u round it, v across it): one of four printed designs
/// for the old four band colours, gold leaf and ink on paper, faded
fn hu_band_tex(kind: usize) -> Tex {
    canvas_tex_opt(256, 64, false, move |ctx, w, h| {
        let (ground, ink, word) = [
            ("#b8954e", "#6a1c18", "FLOR FINA"),
            ("#8a2a2a", "#c9a85a", "HABANA"),
            ("#2c3e5a", "#c4a35a", "LA PREFERIDA"),
            ("#c8b26a", "#3a2a18", "REGALIA"),
        ][kind];
        ctx.set_fill_style(ground);
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style(ink);
        ctx.fill_rect(0, 6, w, 3);
        ctx.fill_rect(0, h - 9.0, w, 3);
        // the medallion in front, the name either side of it
        ctx.begin_path();
        ctx.arc(w * 0.5, h * 0.5, 18, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style(ground);
        ctx.begin_path();
        ctx.arc(w * 0.5, h * 0.5, 13, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style(ink);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 13px Times New Roman");
        ctx.fill_text(word, w * 0.24, h * 0.52);
        ctx.fill_text(word, w * 0.76, h * 0.52);
        ctx.set_font("bold 14px Times New Roman");
        ctx.fill_text("H", w * 0.5, h * 0.52);
        // faded, browned at its edges
        let src = ctx.get_image_data(0, 0, w, h);
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let k = (j * n + i) * 4;
                let (u, v) = (i as f64 / w, j as f64 / h);
                let e = 1.0 - ss(0.0, 0.15, v.min(1.0 - v));
                let mut o = [0, 1, 2].map(|c| f64::from(src.data[k + c]) / 255.0);
                o = mix(o, [0.55, 0.48, 0.38], 0.25 + 0.1 * fbm(kind as u32, u * 10.0, v * 3.0, 2));
                o = o.map(|c| c * (1.0 - 0.3 * e));
                for c in 0..3 {
                    out[i * 4 + c] = (o[c].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A cut foot's filler: the folded leaves' ends, brown and dry
fn hu_filler_tex() -> Tex {
    canvas_tex_opt(32, 32, false, move |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let n = fbm(0xf11, u * 9.0, v * 9.0, 3);
            let fold = ss(0.4, 0.6, vn(0xf12, u * 14.0, v * 14.0));
            opaque([0.30, 0.19, 0.11].map(|c| c * (0.7 + 0.5 * n) * (0.8 + 0.3 * fold)))
        });
    })
}

/// Steel gone dull, a bloom of rust in its corners
fn hu_steel_tex(vary: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = vary ^ 0x57e;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 7.0, v * 7.0, 3);
            let rust = ss(0.66, 0.8, fbm(seed ^ 3, u * 9.0, v * 9.0, 3));
            opaque(mix([0.30, 0.30, 0.31].map(|c| c * (0.8 + 0.3 * n)), [0.32, 0.18, 0.10], rust * 0.6))
        });
    })
}

/// A paper matchbook's cover as its top is mapped: printed in red and
/// cream, SAFETY MATCHES, the striker at its foot, rubbed and stained
fn hu_matchbook_tex(vary: u32) -> Tex {
    let size = (0.038, 0.047);
    pictures::sheet_tex(size, 3000.0, pictures::Age::engraving(vary), vary, |ctx, w, h| {
        ctx.set_fill_style("#7a2420");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#d8c8a0");
        ctx.fill_rect(w * 0.08, h * 0.22, w * 0.84, h * 0.46);
        ctx.set_fill_style("#7a2420");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font(&format!("bold {:.0}px Helvetica", h * 0.11));
        ctx.fill_text("SAFETY", w * 0.5, h * 0.36);
        ctx.fill_text("MATCHES", w * 0.5, h * 0.52);
        ctx.set_fill_style("#d8c8a0");
        ctx.set_font(&format!("{:.0}px Helvetica", h * 0.06));
        ctx.fill_text("CLOSE COVER BEFORE STRIKING", w * 0.5, h * 0.12);
        // the striker strip at the foot, worn by strikes
        ctx.set_fill_style("#3a3028");
        ctx.fill_rect(0, h * 0.84, w, h * 0.12);
    })
}

// ================================================= the pocket watch

/// The watch (t20.py `PW_*`): the dial's face (y), the cover's hinge line
/// (x, y), where the chain leaves the bow (x), the seconds' subdial (z)
const PW_DIAL_Y: f64 = 0.0090;
const PW_HINGE: (f64, f64) = (-0.0256, 0.0106);
const PW_CHAIN_X: f64 = 0.0505;
const PW_SUB_Z: f64 = 0.0098;
/// The open cover's turn past upright, its rim down on the floor (t20.py
/// `PW_COVER_DROP`: the part is modelled open)
const PW_COVER_DROP: f64 = 0.174;

// R751 open pocket watch on chain (floor): 4% chance per level. A 16-size
// hunter-case pocket watch of about 1900 (Blender batch `t20`): gold-filled,
// coin silver, rose gold or gunmetal (the old case's draw), its front cover
// open on its hinge, a white enamel dial with Roman numerals and a seconds
// subdial, blued spade hands stopped at a time (the old hands' draws); its
// Albert chain of curb links trailing to a T-bar or a winding key, or
// broken off; now and then an inscription engraved inside its cover. The
// case worn through at its edges, scratched, the enamel crazed.
pub(super) fn place_pocket_watches(g: &mut Game, _dist: &[i32]) {
    g.pp.pocket_watch_positions_r755.clear(); // R755 reset per level
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // the old draws, call for call: the case, the hands, the chain's end and
    // what hangs there, the engraving, where it lies and how it is turned
    let case_kind = ri(4);
    let hour_draw = rnd();
    let min_draw = rnd();
    let chain_end_x = PW_CHAIN_X + 0.10 + rnd() * 0.08;
    let chain_end_z = -0.03 - rnd() * 0.04;
    let chain_end_roll = rnd();
    let engraved = rnd() < 0.3;
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let turn = rnd() * PI * 2.0;
    let pw = group();
    pw.position().set(px, 0, pz);
    pw.rotation().y = turn;
    let vary = vary(px, pz, turn);
    let case = pw_case_mat(vary, case_kind);
    put(&pw, "Real__pocketWatchCase", &case);
    put(&pw, "Real__pocketWatchDial", &lambert!({ map: &pw_dial_tex(vary) }));
    if let Some(m) = put(&pw, "Real__pocketWatchCrystal", &pictures::glass_mat(vary, 0.35)) {
        m.set_render_order(2);
    }
    // the hands stopped at a time: the minutes from the old minute hand's
    // draw, the hour from the old hour hand's, the hour hand where it is at
    // that minute; the seconds anywhere
    let minutes = (min_draw * 60.0).floor();
    let hours = (hour_draw * 12.0).floor();
    let hour_ang = (hours + minutes / 60.0) / 12.0 * PI * 2.0;
    let min_ang = minutes / 60.0 * PI * 2.0;
    let sec_ang = hash(vary, 7, 7) * PI * 2.0;
    let blued = phong!({ color: 0x141a2a, specular: 0x30384a, shininess: 60 });
    for (name, ang, y, zc) in [
        ("Real__pocketWatchHour", hour_ang, PW_DIAL_Y + 0.0003, 0.0),
        ("Real__pocketWatchMinute", min_ang, PW_DIAL_Y + 0.0007, 0.0),
        ("Real__pocketWatchSecond", sec_ang, PW_DIAL_Y + 0.0003, PW_SUB_Z),
    ] {
        let hg = group();
        hg.position().set(0, y, zc);
        hg.rotation().y = -ang;
        put(&hg, name, &blued);
        pw.add(&hg);
    }
    // the front cover open on its hinge at IX, its rim down on the floor
    put(&pw, "Real__pocketWatchCover", &case);
    // 30% (the old draw): an inscription engraved inside the cover
    if engraved {
        let hinge = group();
        hinge.position().set(PW_HINGE.0, PW_HINGE.1, 0);
        hinge.rotation().z = PI + PW_COVER_DROP;
        pw.add(&hinge);
        let ins = mesh(
            plane_geo!(0.030, 0.022),
            lambert!({ map: &pw_inscription_tex(vary), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
        );
        // on the inside of the lid (its hollow faces down when shut, up now)
        ins.rotation().x = PI / 2.0;
        ins.rotation().z = PI / 2.0;
        ins.position().set(-PW_HINGE.0, 0.0134 - PW_HINGE.1 - 0.0001, 0);
        hinge.add(&ins);
    }
    // the Albert chain from the bow: curb links along a gentle curve
    let chain = group();
    pw.add(&chain);
    let (x0, x1, z1) = (PW_CHAIN_X, chain_end_x, chain_end_z);
    let at = |t: f64| (x0 + t * (x1 - x0), t * z1 + (t * PI).sin() * 0.008);
    // its length, then the links every 4.4 mm along it
    let mut len = 0.0;
    let mut prev = at(0.0);
    for i in 1..=64 {
        let p = at(i as f64 / 64.0);
        len += ((p.0 - prev.0).powi(2) + (p.1 - prev.1).powi(2)).sqrt();
        prev = p;
    }
    let links = (len / 0.0044).round().max(4.0) as usize;
    for i in 0..links {
        let t = (i as f64 + 0.5) / links as f64;
        let (lx, lz) = at(t);
        let (ax, az) = at((t + 0.01).min(1.0));
        let (bx, bz) = at((t - 0.01).max(0.0));
        let l = group();
        l.position().set(lx, if i % 2 == 0 { 0.0 } else { 0.0003 }, lz);
        l.rotation().y = -(az - bz).atan2(ax - bx);
        put(&l, "Real__pocketWatchLink", &case);
        chain.add(&l);
    }
    // what hangs at its end: a T-bar (60%), a winding key (30%), or nothing
    let (ex, ez) = at(1.0);
    let (bx, bz) = at(0.98);
    let dir = (ez - bz).atan2(ex - bx);
    let end = group();
    end.rotation().y = -dir;
    end.position().set(ex, 0, ez);
    if chain_end_roll < 0.60 {
        end.position().set(ex + dir.cos() * 0.0040, 0, ez + dir.sin() * 0.0040);
        put(&end, "Real__pocketWatchTBar", &case);
    } else if chain_end_roll < 0.90 {
        end.position().set(ex + dir.cos() * 0.0110, 0, ez + dir.sin() * 0.0110);
        put(&end, "Real__pocketWatchKey", &case);
    }
    pw.add(&end);
    g.world.add(&pw);
    let p = pw.pos();
    g.pp.pocket_watch_positions_r755.push(v3(p.x, 0, p.z)); // R755
}

/// The case's metal, as the old four: gold-filled, coin silver, rose gold,
/// gunmetal; worn and scratched
fn pw_case_mat(vary: u32, kind: usize) -> Mat {
    let (base, spec, shin) = [
        ([0.52, 0.38, 0.17], 0x6a5530, 50),
        ([0.42, 0.42, 0.41], 0x5a5a5a, 50),
        ([0.50, 0.32, 0.24], 0x6a4a3a, 50),
        ([0.10, 0.10, 0.11], 0x3a3a40, 40),
    ][kind];
    phong!({ map: &pw_metal_tex(vary, base, kind), specular: spec, shininess: shin })
}

/// A watch case's metal as it is mapped: its tarnish, the fine scratches
/// of a pocket, the gold worn through to the base metal at the edges, a
/// silver's black in its hollows, a gunmetal's blue worn grey
fn pw_metal_tex(vary: u32, base: [f64; 3], kind: usize) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x3a7;
        fill(ctx, w, h, |u, v| {
            let k = tarnish(seed, u, v, 6.0);
            let mut o = base.map(|c| c * k);
            let scratch = ss(0.94, 0.985, vn(seed ^ 3, u * 4.0 + v * 70.0, v * 4.0 - u * 50.0));
            o = mix(o, base.map(|c| (c * 1.5).min(0.8)), scratch * 0.5);
            let worn = ss(0.68, 0.82, fbm(seed ^ 5, u * 9.0, v * 9.0, 3));
            o = match kind {
                0 => mix(o, [0.42, 0.33, 0.20], worn * 0.6),
                1 => o.map(|c| c * (1.0 - 0.5 * worn)),
                2 => mix(o, [0.40, 0.30, 0.20], worn * 0.5),
                _ => mix(o, [0.30, 0.30, 0.30], worn * 0.6),
            };
            opaque(o)
        });
    })
}

/// The enamel dial (43 mm) as its face is mapped: white enamel gone
/// ivory, the minute track, Roman numerals in black, the seconds' subdial
/// at VI, a maker's line; hairline cracks crazing the enamel
fn pw_dial_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0xd1;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 5.0, v * 5.0, 3);
            let mut o = [0.86, 0.83, 0.74].map(|c| c * (0.94 + 0.06 * n));
            let d = crack(seed ^ 3, u * 9.0, v * 9.0);
            let line = 1.0 - ss(0.0, 0.012, d);
            let reach = ss(0.55, 0.7, fbm(seed ^ 5, u * 3.0, v * 3.0, 2));
            o = o.map(|c| c * (1.0 - 0.45 * line * reach));
            opaque(o)
        });
        let (cx, cy) = (w / 2.0, h / 2.0);
        // the dial is 43.2 mm across the canvas's 256 px
        let mm = w / 43.2;
        ctx.set_stroke_style("rgba(20,18,16,0.95)");
        ctx.set_fill_style("rgba(20,18,16,0.95)");
        ctx.set_line_width(0.8);
        ctx.begin_path();
        ctx.arc(cx, cy, 19.6 * mm, 0, PI * 2.0);
        ctx.stroke();
        ctx.begin_path();
        ctx.arc(cx, cy, 18.4 * mm, 0, PI * 2.0);
        ctx.stroke();
        for k in 0..60 {
            let a = k as f64 * PI / 30.0;
            let l = if k % 5 == 0 { 1.6 } else { 1.2 };
            ctx.set_line_width(if k % 5 == 0 { 1.4 } else { 0.7 });
            ctx.begin_path();
            ctx.move_to(cx + a.sin() * 19.6 * mm, cy - a.cos() * 19.6 * mm);
            ctx.line_to(cx + a.sin() * (19.6 - l) * mm, cy - a.cos() * (19.6 - l) * mm);
            ctx.stroke();
        }
        const NUM: [&str; 12] = ["XII", "I", "II", "III", "IIII", "V", "VI", "VII", "VIII", "IX", "X", "XI"];
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font(&format!("{:.0}px Times New Roman", 4.4 * mm));
        for (k, t) in NUM.iter().enumerate() {
            if k == 6 {
                continue; // the subdial is where VI would be
            }
            let a = k as f64 * PI / 6.0;
            ctx.save();
            ctx.translate(cx + a.sin() * 15.3 * mm, cy - a.cos() * 15.3 * mm);
            ctx.rotate(a);
            ctx.scale(0.78, 1.0);
            ctx.fill_text(t, 0, 0);
            ctx.restore();
        }
        // the seconds' subdial at VI
        let (sx, sy) = (cx, cy + 9.8 * mm);
        ctx.set_line_width(0.7);
        ctx.begin_path();
        ctx.arc(sx, sy, 5.0 * mm, 0, PI * 2.0);
        ctx.stroke();
        for k in 0..60 {
            let a = k as f64 * PI / 30.0;
            let l = if k % 15 == 0 { 1.2 } else if k % 5 == 0 { 0.9 } else { 0.5 };
            ctx.begin_path();
            ctx.move_to(sx + a.sin() * 5.0 * mm, sy - a.cos() * 5.0 * mm);
            ctx.line_to(sx + a.sin() * (5.0 - l) * mm, sy - a.cos() * (5.0 - l) * mm);
            ctx.stroke();
        }
        ctx.set_font(&format!("{:.0}px Times New Roman", 1.7 * mm));
        ctx.fill_text("WARRANTED", cx, cy - 6.5 * mm);
        // grime round the edge, under the bezel
        let gr = ctx.create_radial_gradient(cx, cy, 15.0 * mm, cx, cy, 21.6 * mm);
        gr.add_color_stop(0, "rgba(70,56,36,0)");
        gr.add_color_stop(1, "rgba(70,56,36,0.35)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, 0, w, h);
    })
}

/// An inscription engraved inside the cover, in a copperplate hand: a
/// present's words, its date; the cuts dark with old polish
fn pw_inscription_tex(vary: u32) -> Tex {
    canvas_tex_opt(180, 132, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(vary, 0x1a5);
        let names = ["J. H.", "W. T. R.", "A. M.", "E. C. B.", "H. L."];
        let froms = ["from his loving wife", "from his father", "for faithful service", "from M.", "from his friends"];
        let name = names[(r.next() * names.len() as f64) as usize];
        let from = froms[(r.next() * froms.len() as f64) as usize];
        let year = 1884 + (r.next() * 30.0) as i32;
        ctx.set_fill_style("rgba(40,30,18,0.75)");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("italic 20px Snell Roundhand");
        ctx.fill_text(name, w / 2.0, h * 0.28);
        ctx.set_font("italic 15px Snell Roundhand");
        ctx.fill_text(from, w / 2.0, h * 0.52);
        ctx.fill_text(&format!("{year}"), w / 2.0, h * 0.74);
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
