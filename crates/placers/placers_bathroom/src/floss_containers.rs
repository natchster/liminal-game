//! `place_floss_containers`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::vn;
use placers_kit::paint::opaque as rgb;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;
use placers_kit::place::skip;

// Dental floss container: 8% chance per level.
pub(crate) fn place_floss_containers(g: &mut Game, _dist: &[i32]) {
    // A 50 m dental floss dispenser (batch `r18`): a moulded polypropylene box
    // and its flip lid, the steel cutter in the lid, lying on its back; its
    // printed label rubbed, the end of the floss left out of it and lying
    // curled on the floor (r18.notes.md)
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    // the old colour draw: which of four (a body and its lid)
    let body_colors = [0xe0e0e0, 0x4080c0, 0xc0c0f0, 0xf0e0f0];
    let c = choice(&body_colors);
    let kind = body_colors.iter().position(|&k| k == c).unwrap_or(0);
    let (body_col, lid_col) = COLOURS[kind];
    let salt = (x as u32).wrapping_mul(0x9e37_79b9) ^ (z as u32).wrapping_mul(0x85eb_ca6b) ^ kind as u32;
    let floss = group();
    let put = |name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            floss.add(&mesh(geo, mat));
        }
    };
    let scuffs = scuff_tex(salt);
    put("Real__flossBody", &phong!({ map: &scuffs, color: body_col, specular: 0x262626, shininess: 34 }));
    put("Real__flossLid", &phong!({ map: &scuffs, color: lid_col, specular: 0x262626, shininess: 34 }));
    put("Real__flossCutter", &phong!({ color: 0x8a8a86, specular: 0x8a8a8a, shininess: 60 }));
    put("Real__flossThread", &lambert!({ color: 0xd2cfc4 }));
    // the label printed on its face
    let label = mesh(
        plane_geo!(0.030, 0.026),
        &lambert!({ map: &label_tex(kind, salt), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    label.rotation().x = -PI / 2.0;
    label.position().set(0.0, 0.0201, -0.0085);
    floss.add(&label);
    let (px, pz) = jitter(g, x, z, 2.4);
    floss.position().set(px, 0, pz);
    floss.rotation().y = rnd() * PI * 2.0;
    g.world.add(&floss);
}

/// Body and lid (sRGB hex): white and teal, blue and white, white and
/// lavender, white and pink, each gone a little yellow
const COLOURS: [(u32, u32); 4] = [(0xd6d3c8, 0x4e7f78), (0x4f7394, 0xd6d3c8), (0xd6d3c8, 0x7f7aa0), (0xd8d2c8, 0xa98690)];

/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right,
/// v up, both 0..1, as the texture is laid) gives straight sRGB and alpha
fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
    let (iw, _) = (w as usize, h as usize);
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

/// The plastic's wear (a grey to multiply its colour by): scuffs, grime
/// in its corners, a dull film
fn scuff_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xf1055;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 6.0, v * 6.0, 3);
            let g = 0.82 + 0.12 * n - (vn(seed ^ 1, u * 50.0, v * 50.0) - 0.75).max(0.0) * 1.2;
            rgb([g, g * 0.98, g * 0.95])
        });
    })
}

/// The printed label (its own maker's: no real one): DENTAL FLOSS, waxed
/// mint, 50 m, a band of the lid's colour; rubbed at its edges
fn label_tex(kind: usize, salt: u32) -> Tex {
    canvas_tex_opt(120, 104, false, move |ctx, w, h| {
        skip(salt);
        ctx.clear_rect(0, 0, w, h);
        let band = ["#3f6e68", "#3d5f7e", "#6a668c", "#8e6c76"][kind.min(3)];
        ctx.set_fill_style(band);
        ctx.fill_rect(0, 0, w, 26);
        ctx.set_fill_style("#f2efe6");
        ctx.set_font("bold 15px Helvetica Neue");
        ctx.set_text_align("center");
        ctx.fill_text_max("DENTAL FLOSS", w / 2.0, 19.0, w - 8.0);
        ctx.set_fill_style("#2a2826");
        ctx.set_font("13px Helvetica Neue");
        ctx.fill_text_max("waxed \u{b7} mint", w / 2.0, 50.0, w - 8.0);
        ctx.set_font("bold 22px Helvetica Neue");
        ctx.fill_text_max("50 m", w / 2.0, 82.0, w - 8.0);
        ctx.set_font("9px Helvetica Neue");
        ctx.fill_text_max("54.6 yd", w / 2.0, 98.0, w - 8.0);
        // rubbed off where thumbs went, and scratched
        ctx.set_global_composite_operation("destination-out");
        for _ in 0..40 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_global_alpha(0.15 + 0.4 * rnd());
            ctx.begin_path();
            ctx.ellipse(x, y, 2.0 + rnd() * 6.0, 1.0 + rnd() * 3.0, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        ctx.set_global_alpha(0.6);
        ctx.fill_rect(0, h - 3.0, w, 3);
    })
}
