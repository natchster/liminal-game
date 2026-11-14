//! `place_paper_airplanes`
use game::prelude::*;
use placers_kit::paint::rgba;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Paper airplane: 5% chance per level. A dart folded from a sheet of
// A4 (Blender batch r15: 297 mm long, its wings 61 mm each), flown into
// something nose first and left where it fell, its wings gone soft, on its
// keel and a wing tip; a typed page or a sheet of coloured paper, dusty.
// The old draws are kept call for call.
pub(crate) fn place_paper_airplanes(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    // the old colour draw: a printed white page, or blue, pink or green paper
    let paper_colors = [0xf8f4e8, 0xd0d8f0, 0xf0e0d0, 0xe8f0d8];
    let col = choice(&paper_colors);
    let look = paper_colors.iter().position(|&c| c == col).unwrap_or(0);
    let plane = group();
    let (px, pz) = jitter(g, x, z, 2.5);
    plane.position().set(px, 0, pz);
    plane.rotation().y = rnd() * PI * 2.0;
    // the old tilt's draw: it now lies on its keel and a wing, as it fell
    let _tilt = rnd();
    let salt = ((px * 977.0).floor() as i64 ^ ((pz * 1181.0).floor() as i64).rotate_left(13)) as u32;
    if let Some(geo) = prop_part_as_modelled("Real__paperPlane") {
        plane.add(&mesh(geo, &lambert!({ map: &paper_tex(look, salt), side: DoubleSide })));
    }
    g.world.add(&plane);
}

/// Copy paper (sRGB): white, blue, pink, green
const PAPER: [(f64, f64, f64); 4] = [(228.0, 224.0, 212.0), (186.0, 196.0, 212.0), (220.0, 198.0, 188.0), (202.0, 214.0, 188.0)];

const MEMO: [&str; 12] = [
    "Minutes of the staff meeting, Thursday 4 May",
    "Present: J. Hollis (chair), R. Dunn, S. Okafor, P. Marsh, L. Grant",
    "Apologies: T. Webb",
    "1. Minutes of the last meeting were agreed.",
    "2. Kitchen rota: the new rota will start on Monday.",
    "   Please wash your own cups.",
    "3. Stationery orders to be in by the 15th.",
    "4. The second floor printer is being repaired.",
    "5. Car park: the barrier will be fixed next week.",
    "6. Any other business: leaving collection for Margaret.",
    "Date of next meeting: Thursday 1 June, 10.00.",
    "",
];

fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: (f64, f64, f64), a: f64) {
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &rgba(c, a));
    gr.add_color_stop(1, &rgba(c, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// The sheet, seen from above (u across the wings, v from the tail down
/// to the nose): a typed page's lines on white (minutes of a meeting,
/// folded into a dart), else plain coloured paper; the floor's dust, the
/// nose scuffed grey where it hit
fn paper_tex(look: usize, salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0xda27);
        ctx.set_fill_style(rgba(PAPER[look], 1.0));
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..120 {
            let (x, y) = (r.next() * w, r.next() * h);
            ctx.set_fill_style(if r.next() < 0.5 { "rgba(255,252,244,0.08)" } else { "rgba(0,0,0,0.05)" });
            ctx.fill_rect(x, y, 3.0 + r.next() * 8.0, 3.0 + r.next() * 8.0);
        }
        if look == 0 {
            // the page's lines, turned as the folds took them
            ctx.save();
            ctx.translate(w / 2.0, h / 2.0);
            ctx.rotate(0.12 * (r.next() - 0.5));
            ctx.set_fill_style("rgba(40,38,36,0.72)");
            ctx.set_text_align("left");
            ctx.set_font("bold 9px Times New Roman");
            ctx.fill_text(MEMO[0], -110, -112);
            ctx.set_font("8px Times New Roman");
            let mut y = -96.0;
            for k in 1..40 {
                ctx.fill_text(MEMO[k % MEMO.len()], -110, y);
                y += 11.0;
            }
            ctx.restore();
        }
        // the nose (at the bottom middle) scuffed and grey where it hit
        soft(ctx, (w / 2.0, h - 6.0), 34.0, 0.7, 0.0, (110.0, 102.0, 90.0), 0.45);
        // creases' grime, the floor's dust
        ctx.set_line_width(0.8);
        for _ in 0..14 {
            let (x, y) = (r.next() * w, r.next() * h);
            let a = r.next() * PI;
            let l = 10.0 + r.next() * 40.0;
            ctx.set_stroke_style(format!("rgba(80,72,58,{:.2})", 0.08 + r.next() * 0.1));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        for _ in 0..20 {
            let at = (r.next() * w, r.next() * h);
            soft(ctx, at, 10.0 + r.next() * 30.0, 0.5 + 0.5 * r.next(), r.next() * 3.0, (110.0, 100.0, 84.0), 0.08 + 0.12 * r.next());
        }
        ctx.set_fill_style("rgba(120,112,98,0.16)");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..500 {
            let (x, y) = (r.next() * w, r.next() * h);
            ctx.set_fill_style(rgba((90.0, 82.0, 70.0), 0.15 + 0.25 * r.next()));
            ctx.fill_rect(x, y, 1.0, 1.0);
        }
    })
}
