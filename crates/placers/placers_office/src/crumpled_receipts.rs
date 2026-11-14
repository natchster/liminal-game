//! `place_crumpled_receipts`
use crate::shared::salt;
use crate::shared::soft;
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Crumpled receipt: 10% chance per level. A till receipt (80 mm
// thermal paper) balled in a hand and dropped: family `everyday_litter`'s balled sheet
// (`place_scrap_papers`), the size a strip of receipt makes of it, its
// thermal print gone brown and faded. The old draws are kept call for call.
pub(crate) fn place_crumpled_receipts(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let receipt = group();
    scatter(g, &receipt, x, z, 2.4);
    let salt = salt(&receipt);
    // a strip of 80 x 200 mm is a quarter of A4: its ball a little over half
    // as big (a crumpled ball's size goes as its area to the 0.4)
    let (name, k) = if salt.is_multiple_of(3) { ("Real__paperWad", 0.55) } else { ("Real__paperBall", 0.57) };
    let geo = prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(0.040, 0.025, 0.040));
    let ball = mesh(geo, &lambert!({ map: &receipt_tex(salt) }));
    ball.scale().set(k, k, k);
    receipt.add(&ball);
    g.world.add(&receipt);
}

// --------------------------------------------------------- the receipt
const TILL: [&str; 22] = [
    "TAKEAWAY",
    "TABLE 4",
    "COFFEE WHITE      1.80",
    "TEA               1.20",
    "TOAST & JAM       1.60",
    "SOUP OF THE DAY   2.95",
    "ROLL BUTTERED     0.85",
    "WATER 500ML       0.90",
    "2 x BISCUIT       0.60",
    "SCONE             1.40",
    "SUBTOTAL          9.75",
    "TOTAL             9.75",
    "CASH             10.00",
    "CHANGE            0.25",
    "ITEMS 7",
    "THANK YOU",
    "VAT INCLUDED",
    "REG 02  OP 11",
    "14:22",
    "No. 004718",
    "--------------------",
    "********************",
];

/// A till receipt balled up (laid on the ball by its box faces): the
/// thermal paper's cool white gone cream, patches of its print turned
/// every way, faded brown where the heat print has aged, the creases'
/// grime, the floor's dust
fn receipt_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0x7ec1);
        ctx.set_fill_style("#e2ded2");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..200 {
            let (x, y) = (r.next() * w, r.next() * h);
            let v = 214.0 + r.next() * 26.0;
            ctx.set_fill_style(format!("rgba({v:.0},{:.0},{:.0},0.16)", v - 3.0, v - 12.0));
            ctx.fill_rect(x, y, 3.0 + r.next() * 9.0, 3.0 + r.next() * 9.0);
        }
        // the print, on facets of the strip turned their own ways (its
        // back is blank: some facets have none)
        ctx.set_font("bold 9px Courier New");
        ctx.set_text_align("left");
        for _ in 0..14 {
            let (cx, cy) = (r.next() * w, r.next() * h);
            let rad = 18.0 + r.next() * 26.0;
            ctx.save();
            ctx.begin_path();
            let n = 5 + (r.next() * 3.0) as usize;
            for k in 0..n {
                let t = (k as f64 + r.next() * 0.4) / n as f64 * PI * 2.0;
                let rr = rad * (0.6 + 0.4 * r.next());
                if k == 0 {
                    ctx.move_to(cx + t.cos() * rr, cy + t.sin() * rr);
                } else {
                    ctx.line_to(cx + t.cos() * rr, cy + t.sin() * rr);
                }
            }
            ctx.close_path();
            ctx.clip();
            ctx.translate(cx, cy);
            ctx.rotate(r.next() * PI * 2.0);
            if r.next() < 0.75 {
                let fade = 0.35 + 0.4 * r.next();
                let mut k = (r.next() * TILL.len() as f64) as usize;
                let mut y = -rad;
                while y < rad + 10.0 {
                    ctx.set_fill_style(format!("rgba(74,62,52,{fade:.2})"));
                    ctx.fill_text(TILL[k % TILL.len()], -rad - 4.0 + r.next() * 6.0, y);
                    k += 1;
                    y += 10.5;
                }
            }
            ctx.restore();
        }
        // crease lines: grime in the valleys
        ctx.set_line_width(0.8);
        for _ in 0..44 {
            let (x, y) = (r.next() * w, r.next() * h);
            let a = r.next() * PI;
            let l = 8.0 + r.next() * 36.0;
            ctx.set_stroke_style(format!("rgba(72,64,52,{:.2})", 0.08 + r.next() * 0.14));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        for _ in 0..18 {
            let at = (r.next() * w, r.next() * h);
            soft(ctx, at, 8.0 + r.next() * 20.0, 0.5 + r.next() * 0.4, r.next() * 3.0, (96.0, 86.0, 66.0), 0.08 + r.next() * 0.14);
        }
    })
}
