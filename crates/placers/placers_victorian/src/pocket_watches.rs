//! `place_pocket_watches`
use game::prelude::*;
use game::visuals::pictures;
use placers_kit::geometry::put;
use placers_kit::noise::crack;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::paint::opaque;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;
use placers_kit::place::vary;

// Open pocket watch on chain (floor): 4% chance per level. A 16-size
// hunter-case pocket watch of about 1900 (Blender family `a_studys_things`): gold-filled,
// coin silver, rose gold or gunmetal (the old case's draw), its front cover
// open on its hinge, a white enamel dial with Roman numerals and a seconds
// subdial, blued spade hands stopped at a time (the old hands' draws); its
// Albert chain of curb links trailing to a T-bar or a winding key, or
// broken off; now and then an inscription engraved inside its cover. The
// case worn through at its edges, scratched, the enamel crazed.
pub(crate) fn place_pocket_watches(g: &mut Game, _dist: &[i32]) {
    g.pp.hunter_watch_positions.clear(); // reset per level
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
    let at = |t: f64| (x0 + t * (x1 - x0), t * z1 + (t * PI).dsin() * 0.008);
    // its length, then the links every 4.4 mm along it
    let mut len = 0.0;
    let mut prev = at(0.0);
    for i in 1..=64 {
        let p = at(i as f64 / 64.0);
        len += ((p.0 - prev.0).dpowi(2) + (p.1 - prev.1).dpowi(2)).sqrt();
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
        l.rotation().y = -(az - bz).datan2(ax - bx);
        put(&l, "Real__pocketWatchLink", &case);
        chain.add(&l);
    }
    // what hangs at its end: a T-bar (60%), a winding key (30%), or nothing
    let (ex, ez) = at(1.0);
    let (bx, bz) = at(0.98);
    let dir = (ez - bz).datan2(ex - bx);
    let end = group();
    end.rotation().y = -dir;
    end.position().set(ex, 0, ez);
    if chain_end_roll < 0.60 {
        end.position().set(ex + dir.dcos() * 0.0040, 0, ez + dir.dsin() * 0.0040);
        put(&end, "Real__pocketWatchTBar", &case);
    } else if chain_end_roll < 0.90 {
        end.position().set(ex + dir.dcos() * 0.0110, 0, ez + dir.dsin() * 0.0110);
        put(&end, "Real__pocketWatchKey", &case);
    }
    pw.add(&end);
    g.world.add(&pw);
    let p = pw.pos();
    g.pp.hunter_watch_positions.push(v3(p.x, 0, p.z));
}

/// The watch (victorian/a_studys_things.py `PW_*`): the dial's face (y), the cover's hinge line
/// (x, y), where the chain leaves the bow (x), the seconds' subdial (z)
const PW_DIAL_Y: f64 = 0.0090;

const PW_HINGE: (f64, f64) = (-0.0256, 0.0106);

const PW_CHAIN_X: f64 = 0.0505;

const PW_SUB_Z: f64 = 0.0098;

/// The open cover's turn past upright, its rim down on the floor (victorian/a_studys_things.py
/// `PW_COVER_DROP`: the part is modelled open)
const PW_COVER_DROP: f64 = 0.174;

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
            let scratch = sstep(0.94, 0.985, vn(seed ^ 3, u * 4.0 + v * 70.0, v * 4.0 - u * 50.0));
            o = mix_unclamped(o, base.map(|c| (c * 1.5).min(0.8)), scratch * 0.5);
            let worn = sstep(0.68, 0.82, fbm(seed ^ 5, u * 9.0, v * 9.0, 3));
            o = match kind {
                0 => mix_unclamped(o, [0.42, 0.33, 0.20], worn * 0.6),
                1 => o.map(|c| c * (1.0 - 0.5 * worn)),
                2 => mix_unclamped(o, [0.40, 0.30, 0.20], worn * 0.5),
                _ => mix_unclamped(o, [0.30, 0.30, 0.30], worn * 0.6),
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
            let line = 1.0 - sstep(0.0, 0.012, d);
            let reach = sstep(0.55, 0.7, fbm(seed ^ 5, u * 3.0, v * 3.0, 2));
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
            ctx.move_to(cx + a.dsin() * 19.6 * mm, cy - a.dcos() * 19.6 * mm);
            ctx.line_to(cx + a.dsin() * (19.6 - l) * mm, cy - a.dcos() * (19.6 - l) * mm);
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
            ctx.translate(cx + a.dsin() * 15.3 * mm, cy - a.dcos() * 15.3 * mm);
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
            let l = if k % 15 == 0 {
                1.2
            } else if k % 5 == 0 {
                0.9
            } else {
                0.5
            };
            ctx.begin_path();
            ctx.move_to(sx + a.dsin() * 5.0 * mm, sy - a.dcos() * 5.0 * mm);
            ctx.line_to(sx + a.dsin() * (5.0 - l) * mm, sy - a.dcos() * (5.0 - l) * mm);
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

/// Tarnished metal's mottle (a multiplier near 1)
fn tarnish(seed: u32, u: f64, v: f64, scale: f64) -> f64 {
    0.8 + 0.3 * fbm(seed, u * scale, v * scale, 3) - 0.18 * sstep(0.6, 0.8, fbm(seed ^ 9, u * scale * 0.6, v * scale * 0.6, 3))
}
