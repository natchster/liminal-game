//! `place_magnetic_letters`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Magnetic letters: 10% chance per level. 4-8 plastic fridge letters
// spilled on the floor (Blender family `toys_on_the_floor`): moulded capitals 38 mm tall,
// a round magnet set in each one's back, in the faded colours of an
// alphabet set; random letters, no word. Some landed on their backs.
pub(crate) fn place_magnetic_letters(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let Some((x, z)) = pick_open(g) else { return };
    let letter_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0, 0xe07020];
    // the old colours as an alphabet set's polypropylene, faded
    let plastic: [u32; 6] = [0x983634, 0x35568a, 0xc29a3c, 0x3f7a43, 0x725089, 0xb96a30];
    let wear = fridge_letter_tex();
    let mut mats: Vec<Option<Mat>> = vec![None; plastic.len()];
    let magnet_mat = lambert!({ color: 0x3b3936 });
    let grp = group();
    // where each letter was drawn (the old spacing test runs on these), and
    // where each lies, its reach, its letter
    let mut drawn: Vec<(f64, f64)> = vec![];
    let mut lying: Vec<(f64, f64, f64)> = vec![];
    let mut used = [false; 26];
    let n_letters = 4 + (rnd() * 5.0).floor() as i32;
    for _ in 0..n_letters {
        let col = pick_color(&letter_colors);
        let k = letter_colors.iter().position(|&c| c == col).unwrap_or(0);
        // Keep clear of the tiles already down (half-diagonal 2 cm); widen the
        // scatter a little on each retry when it gets crowded.
        let (mut lx, mut lz) = (0.0, 0.0);
        for tries in 0..30 {
            let spread = 1.0 + tries as f64 * 0.06;
            lx = (rnd() - 0.5) * 0.18 * spread;
            lz = (rnd() - 0.5) * 0.12 * spread;
            if drawn.iter().all(|&(px, pz)| hypot2(px - lx, pz - lz) >= 0.040) {
                break;
            }
        }
        drawn.push((lx, lz));
        let ry = rnd() * PI * 2.0;
        // which letter, and whether it landed on its back, from where it fell
        // (no new draw); no letter twice
        let mut li = ((((lx + 0.5) * 9973.0 + (lz + 0.5) * 7919.0).fract().abs()) * 26.0) as usize % 26;
        while used[li] {
            li = (li + 7) % 26;
        }
        used[li] = true;
        let ch = (b'A' + li as u8) as char;
        let on_back = (ry * 13.7).fract() < 0.2;
        let Some(geo) = prop_part_as_modelled(&format!("Real__fridgeLetter{ch}")) else { continue };
        let bb = geo.bounding_box();
        let reach = 0.5 * hypot2(bb.max.x - bb.min.x, bb.max.z - bb.min.z);
        // clear of the letters already down: pushed off them where they fell
        let (mut qx, mut qz) = (lx, lz);
        for _ in 0..8 {
            for &(ax, az, ar) in &lying {
                let (dx, dz) = (qx - ax, qz - az);
                let d = hypot2(dx, dz);
                let need = reach + ar + 0.003;
                if d < need {
                    let (ux, uz) = if d > 1e-6 { (dx / d, dz / d) } else { (1.0, 0.0) };
                    qx += ux * (need - d);
                    qz += uz * (need - d);
                }
            }
        }
        lying.push((qx, qz, reach));
        let mat = mats[k].get_or_insert_with(|| phong!({ color: plastic[k], map: &wear, specular: 0x222222, shininess: 30 })).clone();
        let letter = group();
        let inner = group();
        inner.add(&mesh(geo, &mat));
        if let Some(mg) = prop_part_as_modelled(&format!("Real__fridgeLetterMagnet{ch}")) {
            inner.add(&mesh(mg, &magnet_mat));
        }
        if on_back {
            inner.rotation().z = PI;
            inner.position().y = bb.max.y;
        }
        letter.add(&inner);
        letter.position().set(qx, 0, qz);
        letter.rotation().y = ry;
        grp.add(&letter);
    }
    scatter_pos(g, &grp, x, z, 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

/// The letters' wear (multiplies each one's colour): the moulded plastic
/// scuffed pale where it was handled, scratched, grime and grit in it
fn fridge_letter_tex() -> Tex {
    canvas_tex(128, 128, |ctx, w, h| {
        ctx.set_fill_style("#e8e3da");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..12 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 10.0 + rnd() * 24.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, if rnd() < 0.5 { "rgba(255,252,244,0.30)" } else { "rgba(66,58,46,0.26)" });
            gr.add_color_stop(1, "rgba(0,0,0,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        ctx.set_line_width(0.8);
        for _ in 0..30 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 3.0 + rnd() * 12.0;
            ctx.set_stroke_style(if rnd() < 0.7 { "rgba(252,248,240,0.4)" } else { "rgba(44,38,32,0.3)" });
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.dcos() * l, y + a.dsin() * l);
            ctx.stroke();
        }
        for _ in 0..90 {
            ctx.set_fill_style(format!("rgba(50,44,36,{:.2})", 0.25 + rnd() * 0.35));
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
    })
}
