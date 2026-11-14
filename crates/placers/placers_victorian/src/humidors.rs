//! `place_humidors`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use game::visuals::pictures;
use placers_kit::geometry::put;
use placers_kit::noise::crack;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix_unclamped as mix;
use placers_kit::noise::sstep as ss;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::paint::opaque;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;
use placers_kit::place::vary;

// Humidor cigar box w/ scattered cigars (floor): 3% chance per level.
// A Victorian humidor (Blender batch `t20`): a veneered case on its plinth
// (mahogany, burr walnut, rosewood or ebonised: the old colour's draw),
// lined with Spanish cedar or zinc (the old lining's draw), the lid open
// on its brass hinges, a brass escutcheon and name plate; Coronas lying in
// it and spilled on the floor where they fell, their bands printed; now
// and then a steel cutter, a paper matchbook. The veneer's
// polish gone dull and crazed, the cedar pale, the cigars dried.
pub(crate) fn place_humidors(g: &mut Game, _dist: &[i32]) {
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
        // keeps the old draws of a spilled cigar's rocking (50%: its phase,
        // speed and swing): it lies still now
        if !inside && rnd() < 0.5 {
            let _ = rnd();
            let _ = rnd();
            let _ = rnd();
        }
        // the old length's draw scales the whole cigar a little (0.92-1.06)
        cigars.push(Cigar { leaf, scale: 0.92 + (len - 0.11) / 0.03 * 0.14, dry, yaw, at, inside, band });
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

/// Wood seen along its grain (u along it): the growth rings' streaks and
/// the figure, `base` its colour (sRGB)
fn wood(seed: u32, u: f64, v: f64, base: [f64; 3], along: f64, across: f64) -> [f64; 3] {
    let warp = fbm(seed ^ 0x1d, u * along * 0.15, v * across * 0.05, 3);
    let ring = ((v * across + warp * 6.0).sin() * 0.5 + 0.5).powf(3.0);
    let streak = fbm(seed, u * along * 0.2, v * across, 3);
    let fine = vn(seed ^ 3, u * along * 3.0, v * across * 6.0);
    base.map(|c| c * (0.82 + 0.28 * streak + 0.06 * fine - 0.14 * ring))
}
