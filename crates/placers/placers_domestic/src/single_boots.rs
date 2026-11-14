//! `place_single_boots`
use crate::shared::r14a_add;
use crate::shared::r14a_css;
use crate::shared::r14a_leather_tex;
use crate::shared::r14a_mix;
use crate::shared::r14a_rgb;
use crate::shared::r14a_soft;
use crate::shared::r14a_wander;
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter_xz_turned as scatter_xz;

// Single worn boot: 8% chance per level. A 6 in lace-up work boot left on
// the floor, standing or fallen on its side (Blender batch r14a: the shaft
// and padded collar, the quarters lapped over the vamp, the lug sole, the
// lace undone from its hooks).
pub(crate) fn place_single_boots(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // the old colours stand for a work boot's leathers: black, dark brown
    // oiled, wheat nubuck faded, oxblood; (leather, lining, sole, laces, metal)
    let boot_colors = [0x201a10, 0x503020, 0x6a3820, 0x2a1810];
    let cols = match choice(&boot_colors) {
        0x201a10 => (0x2e2b28, 0x2a2522, 0x262422, 0x2a2724, 0x3c3832),
        0x503020 => (0x4e3626, 0x3a2c22, 0x2a2826, 0x7a6448, 0x4a4236),
        0x6a3820 => (0x8a6a48, 0x5a4836, 0x3a332c, 0x8c7a5c, 0x5c5040),
        _ => (0x46282a, 0x34262a, 0x262422, 0x3a2c24, 0x3c3832),
    };
    // 25% fallen over: as it lies on its outer side (modelled so)
    let tipped = rnd() < 0.25;
    let boot = group();
    scatter_xz(g, &boot, x, z, 2.4);
    let p = boot.pos();
    let vary = ((p.x * 7919.0).abs() as u32) ^ ((p.z * 104_729.0).abs() as u32).rotate_left(11);
    boot.add(&r14a_footwear("boot", if tipped { "Side" } else { "" }, cols, vary));
    g.world.add(&boot);
}

/// The shoe or boot of batch r14a, `kind` "shoe" or "boot": `var` "R", "L",
/// "Side" (the right one fallen on its outer side, as it lies) or "" (the
/// boot standing). Colours sRGB: the leather, the lining, the sole's
/// rubber or leather, the laces, the eyelets' and hooks' metal.
fn r14a_footwear(kind: &str, var: &str, cols: (u32, u32, u32, u32, u32), vary: u32) -> Obj {
    let (leather, lining, sole, lace, metal) = cols;
    let grp = group();
    let name = |p: &str| format!("Real__{kind}{p}{var}");
    let boot = kind == "boot";
    let upper = lambert!({ color: 0xffffff, map: &r14a_leather_tex(leather, 256, vary) });
    r14a_add(&grp, &name("Upper"), &upper);
    r14a_add(&grp, &name("Lining"), &lambert!({ color: lining }));
    let edge = lambert!({ color: 0xffffff, map: &r14a_sole_tex(sole, false, vary ^ 0x91) });
    r14a_add(&grp, &name("Sole"), &edge);
    let heel =
        if boot || var == "Side" { edge.clone() } else { lambert!({ color: 0xffffff, map: &r14a_sole_tex(0x3e2c22, true, vary ^ 0x93) }) };
    r14a_add(&grp, &name("Heel"), &heel);
    // the tread shows only on one that has fallen over
    let tread = if var == "Side" {
        lambert!({ color: 0xffffff, map: &r14a_tread_tex(boot, sole, vary ^ 0x97) })
    } else {
        lambert!({ color: sole })
    };
    r14a_add(&grp, &name("Tread"), &tread);
    r14a_add(&grp, &name("Laces"), &lambert!({ color: lace }));
    let met = phong!({ color: metal, specular: 0x4a4a46, shininess: 40 });
    r14a_add(&grp, &name(if boot { "Hooks" } else { "Eyelets" }), &met);
    grp
}

/// A sole's edge and welt (or `stack`, a heel of leather lifts on a rubber
/// top piece, its layers across): inked and burnished, scuffed pale, dirt.
fn r14a_sole_tex(base: u32, stack: bool, vary: u32) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let c = r14a_rgb(base);
            ctx.set_fill_style(&r14a_css(c, 1.0));
            ctx.fill_rect(0, 0, w, h);
            if stack {
                // the lifts (each 4-5 mm of the heel's 22), the top piece below
                let mut y = 0.0;
                while y < h * 0.76 {
                    ctx.set_fill_style(&r14a_css((c.0 * 0.55, c.1 * 0.5, c.2 * 0.48), 0.7));
                    ctx.fill_rect(0, y, w, 1.0);
                    ctx.set_fill_style(&r14a_css(r14a_mix(c, (150.0, 120.0, 96.0), 0.3), 0.3));
                    ctx.fill_rect(0, y + 1.0, w, 1.0);
                    y += h * 0.19;
                }
                ctx.set_fill_style("rgb(40,38,36)");
                ctx.fill_rect(0, h * 0.77, w, h * 0.23);
            }
            for _ in 0..40 {
                ctx.set_fill_style(&r14a_css((c.0 * 0.7, c.1 * 0.68, c.2 * 0.66), 0.2));
                ctx.fill_rect(rnd() * w, rnd() * h, 4.0 + rnd() * 18.0, 1.0);
            }
            for _ in 0..22 {
                let crust = r14a_mix(c, (150.0, 140.0, 126.0), 0.5);
                r14a_soft(ctx, (rnd() * w, rnd() * h), 2.0 + rnd() * 6.0, 0.4, (rnd() - 0.5) * 0.5, crust, 0.35);
            }
            for _ in 0..10 {
                r14a_soft(ctx, (rnd() * w, rnd() * h), 8.0 + rnd() * 20.0, 0.5, 0.0, (110.0, 102.0, 90.0), 0.12);
            }
        })
    })
}

/// The tread of one fallen on its side, as it lies (u from the toe, at u =
/// 0, back to the heel; v from the outer edge, on the floor, up to the inner
/// one): a shoe's leather sole worn dark at the ball and frayed at the toe,
/// paler in the waist where it never touched, the heel's rubber top piece
/// worn at its back; a boot's lugs, worn grey on their faces, grit in the
/// grooves.
fn r14a_tread_tex(boot: bool, base: u32, vary: u32) -> Tex {
    canvas_tex(256, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            // the heel's breast, as modelled (domestic/things_worn_and_dropped.py): the shoe's at 78% of
            // the tread from the toe, the boot's at 77%
            let br = if boot { 0.771 } else { 0.781 };
            if !boot {
                let leather = (122.0, 96.0, 70.0);
                ctx.set_fill_style(&r14a_css(leather, 1.0));
                ctx.fill_rect(0, 0, w, h);
                // worn: the ball dark and smooth, the toe frayed paler
                r14a_soft(ctx, (w * 0.34, h * 0.5), w * 0.22, 0.6, 0.0, (64.0, 54.0, 44.0), 0.6);
                r14a_soft(ctx, (w * 0.04, h * 0.5), w * 0.07, 1.4, 0.0, (150.0, 132.0, 108.0), 0.5);
                // the waist, never on the floor: paler, the size stamped
                r14a_soft(ctx, (w * 0.66, h * 0.5), w * 0.12, 0.6, 0.0, (160.0, 132.0, 100.0), 0.45);
                ctx.save();
                ctx.translate(w * 0.66, h * 0.5);
                ctx.rotate(-PI / 2.0);
                ctx.set_fill_style("rgba(60,46,34,0.55)");
                ctx.set_font("bold 11px Helvetica");
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.fill_text("9", 0, -6);
                ctx.set_font("7px Helvetica");
                ctx.fill_text("LEATHER", 0, 7);
                ctx.restore();
                for _ in 0..140 {
                    r14a_wander(ctx, (rnd() * w * br, rnd() * h), 3.0 + rnd() * 9.0, (rnd() - 0.5) * 6.3, (70.0, 58.0, 46.0), 0.25, 0.6);
                }
                // the heel's top piece, black rubber, worn at the back
                ctx.set_fill_style("rgb(44,41,38)");
                ctx.fill_rect(w * br, 0, w * (1.0 - br), h);
                r14a_soft(ctx, (w * 1.0, h * 0.35), w * 0.06, 1.8, 0.0, (96.0, 88.0, 78.0), 0.6);
                for k in 0..5 {
                    ctx.set_fill_style("rgba(120,112,100,0.7)");
                    ctx.fill_rect(w * (br + 0.03 + 0.035 * (k as f64 % 2.0)), h * (0.2 + 0.15 * k as f64), 2.0, 2.0);
                }
            } else {
                let c = r14a_rgb(base);
                let groove = (c.0 * 0.55, c.1 * 0.55, c.2 * 0.55);
                ctx.set_fill_style(&r14a_css(groove, 1.0));
                ctx.fill_rect(0, 0, w, h);
                // lugs in rows across the sole, chevroned; few in the waist
                let face = r14a_mix(c, (120.0, 114.0, 106.0), 0.35);
                let mut x = 6.0;
                let mut row = 0;
                while x < w - 4.0 {
                    let waist = x / w > 0.56 && x / w < br;
                    if x / w > br - 0.012 && x / w < br + 0.012 {
                        x += 6.0;
                        continue;
                    }
                    let n = if waist { 2 } else { 4 };
                    for k in 0..n {
                        let y = h * (0.12 + 0.76 * (k as f64 + 0.5 * (row % 2) as f64) / n as f64);
                        ctx.save();
                        ctx.translate(x, y);
                        ctx.rotate(if k * 2 < n { 0.5 } else { -0.5 });
                        ctx.set_fill_style(&r14a_css(face, 1.0));
                        ctx.fill_rect(-4.0, -9.0, 8.0, 18.0);
                        ctx.restore();
                    }
                    x += if waist { 16.0 } else { 11.0 };
                    row += 1;
                }
                // the faces worn grey, grit in the grooves
                r14a_soft(ctx, (w * 0.34, h * 0.5), w * 0.2, 0.6, 0.0, (130.0, 124.0, 114.0), 0.25);
                r14a_soft(ctx, (w * 0.92, h * 0.5), w * 0.1, 0.8, 0.0, (130.0, 124.0, 114.0), 0.25);
                for _ in 0..260 {
                    ctx.set_fill_style(&r14a_css((150.0, 140.0, 124.0), 0.35));
                    ctx.fill_rect(rnd() * w, rnd() * h, 1.0, 1.0);
                }
            }
        })
    })
}
