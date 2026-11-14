//! `place_safety_glasses`
use game::prelude::*;
use placers_kit::noise::vn;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;
use placers_kit::place::seed_of;

// Safety glasses: 8% chance per level. A pair of clear wraparound
// safety spectacles (Blender family `small_things_dropped_or_left`: 125 mm across, one polycarbonate
// lens with its side shields and nose notch, nylon temples 132 mm), put down
// open on the lens's foot and the temples' tips, or folded and lying on
// their back; the lens scratched and hazed, a thumb's smear on it.
pub(crate) fn place_safety_glasses(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let glasses = group();
    // where it lies and its turn: the old draws, made as before
    scatter(g, &glasses, x, z, 2.4);
    let p = glasses.pos();
    // which pair, from where it lies (no draw): folded or open, a grey lens
    // now and then, the temples' nylon
    let seed = seed_of(&[p.x, p.z]);
    let folded = seed % 5 < 2;
    let grey = (seed >> 4).is_multiple_of(4);
    let nylon = [0x1b1b1d, 0x26303c, 0x3a3d40][((seed >> 7) % 3) as usize];
    let (lens_n, frame_n) = if folded {
        ("Real__safetyGlassesFoldedLens", "Real__safetyGlassesFoldedFrame")
    } else {
        ("Real__safetyGlassesOpenLens", "Real__safetyGlassesOpenFrame")
    };
    if let Some(geo) = prop_part_as_modelled(frame_n) {
        glasses.add(&mesh(geo, phong!({ color: nylon, specular: 0x2a2a2a, shininess: 34 })));
    }
    if let Some(geo) = prop_part_as_modelled(lens_n) {
        let lens = mesh(geo, lens_mat(&lens_dirt_tex(seed), if grey { 0.42 } else { 0.0 }));
        lens.set_render_order(2);
        glasses.add(&lens);
    }
    g.world.add(&glasses);
}

/// Polycarbonate as the windows' glass (`Patch::Glass`): the room seen in
/// it by Fresnel, its dirt from the map's grey; `tint` darkens a grey
/// lens (the view through it dimmed)
fn lens_mat(map: &Tex, tint: f32) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x8a8a8a, shininess: 180, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "safety_lens".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = 0.55;
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // the dirt: how much of the map's grey, its colour (linear)
        f[8] = 2.2;
        f[9] = 0.19;
        f[10] = 0.185;
        f[11] = 0.17;
        // a grey lens: a body of its own, dark
        f[12] = tint;
        f[13] = 0.08;
    }
    m
}

/// What years did to the lens (grey: how much dirt): a haze, dust
/// settled in blotches, a thumb's smear, and scratches, fine and many,
/// a few long from being dropped on their face; different on every pair
fn lens_dirt_tex(seed: u32) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        let s = seed;
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = (i as f64 / w, j as f64 / h);
                let d = vn(s, x * 6.0, y * 3.0) * 0.55 + vn(s ^ 9, x * 25.0, y * 12.0) * 0.45;
                let smear = (vn(s ^ 77, x * 2.5, y * 4.0) - 0.62).max(0.0) * 1.6;
                let v = ((0.20 + 0.34 * d + smear) * 255.0).min(255.0) as u8;
                out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        let mut r = SplitMix64::new(seed, 0x51f7);
        ctx.set_line_cap("round");
        // fine scratches, mostly one way (wiped on a sleeve)
        for _ in 0..90 {
            let (x0, y0) = (r.range(0.0, w), r.range(0.0, h));
            let a = r.range(-0.4, 0.4) + if r.next() < 0.3 { 1.3 } else { 0.0 };
            let l = r.range(6.0, 40.0);
            ctx.set_stroke_style(format!("rgba(235,235,235,{:.2})", r.range(0.25, 0.6)));
            ctx.set_line_width(r.range(0.5, 1.1));
            ctx.begin_path();
            ctx.move_to(x0, y0);
            ctx.line_to(x0 + a.cos() * l, y0 + a.sin() * l);
            ctx.stroke();
        }
        // a few long ones, from a fall
        for _ in 0..4 {
            let (x0, y0) = (r.range(0.0, w), r.range(0.0, h));
            let a = r.range(0.0, PI);
            let l = r.range(50.0, 140.0);
            ctx.set_stroke_style("rgba(250,250,250,0.75)");
            ctx.set_line_width(1.4);
            ctx.begin_path();
            ctx.move_to(x0, y0);
            ctx.quadratic_curve_to(
                x0 + a.cos() * l * 0.5 + r.range(-8.0, 8.0),
                y0 + a.sin() * l * 0.5 + r.range(-8.0, 8.0),
                x0 + a.cos() * l,
                y0 + a.sin() * l,
            );
            ctx.stroke();
        }
        // a thumb's print where it was picked up: soft whorls
        let (tx, ty) = (r.range(0.15, 0.85) * w, r.range(0.25, 0.75) * h);
        ctx.set_line_width(1.0);
        for k in 0..9 {
            let rr = 3.0 + k as f64 * 2.2;
            ctx.set_stroke_style(format!("rgba(210,210,205,{:.2})", 0.30 - k as f64 * 0.025));
            ctx.begin_path();
            ctx.ellipse(tx, ty, rr * 1.25, rr, 0.5, 0, 6.3);
            ctx.stroke();
        }
    })
}
