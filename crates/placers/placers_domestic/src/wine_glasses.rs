//! `place_wine_glasses`
use game::prelude::*;
use placers_kit::baking;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// A wine glass left on the floor (Blender batch `r16`): a stemmed 8 oz
// glass of the Libbey Citation class (171 mm tall), the wine in it long gone
// down to a dark crust in the bottom of the bowl, dried rings where it stood,
// a lipstick print on the rim now and then; or knocked over, lying on its
// foot's rim and its bowl's belly, the wine dried into the carpet in front
// of its mouth. 5% chance per level.
pub(crate) fn place_wine_glasses(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // which wine it was: red, an older red, rosé, white (the old colours)
    let wine_colors = [0x601020, 0x400820, 0xa02040, 0xd0a040];
    let wine_color = pick(&wine_colors);
    let wine = wine_colors.iter().position(|&c| c == wine_color).unwrap_or(0);
    let is_spilled = rnd() < 0.35;
    // standing: how full it was left (the old fill level, 65-75% of the old
    // bowl), now the dried ring's height; knocked over: the old tilt's draw,
    // how far the wine ran
    let (level, run) = if !is_spilled { (0.65 + rnd() * 0.10, 0.0) } else { (0.0, (rnd() - 0.5) * 0.3) };
    // (35%) a lipstick print on the rim; the draw is made either way
    let lipstick = rnd() < 0.35 && !is_spilled;
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let yaw = rnd() * PI * 2.0;
    // what is painted differs floor to floor: from where it stands (no draw)
    let salt = ((px * 1013.0).floor() as i64 ^ ((pz * 733.0).floor() as i64).rotate_left(17)) as u32;
    let grp = group();
    if !is_spilled {
        let tide = WINE_TIDE.0 + (WINE_TIDE.1 - WINE_TIDE.0) * ((level - 0.65) / 0.10).clamp(0.0, 1.0);
        if let Some(geo) = prop_part_as_modelled("Real__wineGlassDregs") {
            grp.add(&mesh(geo, phong!({ color: WINE_DREGS[wine], specular: 0x3a3436, shininess: 90 })));
        }
        if let Some(geo) = prop_part_as_modelled("Real__wineGlassFoot") {
            // the foot writes depth: the placement measures the glass by it
            // (a glass that writes none is not there to it, and the dregs
            // alone would read as hovering)
            let foot = mesh(geo, wine_glass_mat(&wine_glass_dust_tex(), 0.5));
            foot.mat().depthWrite(true);
            grp.add(&foot);
        }
        if let Some(geo) = prop_part_as_modelled("Real__wineGlassBowl") {
            let bowl = mesh(geo.clone(), wine_glass_mat(&wine_glass_dust_tex(), 0.45));
            bowl.set_render_order(2);
            grp.add(&bowl);
            // the wine's film and rings on the glass, and the lipstick
            let film = mesh(
                geo,
                phong!({
                    map: &wine_glass_film_tex(wine, tide, lipstick, salt),
                    color: 0xffffff,
                    specular: 0x2a2a2a,
                    shininess: 60,
                    transparent: true,
                    depthWrite: false
                }),
            );
            film.set_render_order(3);
            grp.add(&film);
        }
    } else if let Some(geo) = prop_part_as_modelled("Real__wineGlassLying") {
        let lying = mesh(geo, wine_glass_mat(&wine_glass_dust_tex(), 0.55));
        lying.set_render_order(2);
        grp.add(&lying);
    }
    grp.position().set(px, 0, pz);
    grp.rotation().y = yaw;
    g.world.add(&grp);
    if is_spilled {
        // where it ran out of the mouth (+x of the lying glass) and dried
        // into the floor: a stain, flat on it, its own object as the old pool
        let k = 1.0 + run * 1.6;
        let stain = mesh(
            plane_geo!(0.24 * k, 0.18 * k),
            lambert!({
                map: &wine_stain_tex(wine, salt),
                color: 0xffffff,
                transparent: true,
                depthWrite: false,
                polygonOffset: true,
                polygonOffsetFactor: -2,
                polygonOffsetUnits: -2
            }),
        );
        let reach = 0.10 + 0.04 * k;
        stain.rotation().x = -PI / 2.0;
        stain.rotation().z = rnd() * PI * 2.0;
        stain.position().set(px + yaw.cos() * reach, 0.0012, pz - yaw.sin() * reach);
        g.world.add(&stain);
    }
}

/// Where the wine stood in the bowl (`Real__wineGlassBowl`, y in metres):
/// the old fill level's range, a third to a half of the bowl
const WINE_TIDE: (f64, f64) = (0.100, 0.122);

/// The bowl part's height (its bounds: from where the stem joins, 77 mm, to
/// the rim) and the dregs' top (domestic/kitchen_and_hearth_left_behind.py)
const WINE_BOWL_Y: (f64, f64) = (0.0770, 0.1710);

const WINE_DREGS_TOP: f64 = 0.0905;

/// What each wine leaves in the glass: dried crust (sRGB)
const WINE_DREGS: [u32; 4] = [0x2a0c12, 0x1e0a0e, 0x4a1a1e, 0x6a4a1c];

/// Its dried film and rings (r, g, b) and how strong
const WINE_FILM: [(u8, u8, u8, f64); 4] = [(92, 26, 38, 0.55), (70, 20, 30, 0.6), (140, 64, 70, 0.4), (150, 118, 52, 0.32)];

/// Smooth value noise in [0, 1]
fn wine_noise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (fx, fy) = (x - xf, y - yf);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xf as i32, yf as i32);
    let (a, b) = (baking::hash(seed, xi, yi), baking::hash(seed, xi + 1, yi));
    let (c, d) = (baking::hash(seed, xi, yi + 1), baking::hash(seed, xi + 1, yi + 1));
    a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}

/// Clear glass as the windows' and the mirror's (`Patch::Glass`): the room
/// seen in it by Fresnel, bright at its edges where it is seen at a graze,
/// clear square on; its dust (the map's grey, `dust` of it) lit as dust
fn wine_glass_mat(map: &Tex, dust: f32) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 260, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "wine_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it, and nothing lit behind it
        f[0] = 0.32;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // dust: how much of the map's grey, its colour (linear)
        f[8] = dust;
        f[9] = 0.20;
        f[10] = 0.19;
        f[11] = 0.17;
        f[12] = 0.0;
        // two faces of glass: 4% each, square on
        f[13] = 0.07;
    }
    m
}

/// Dust and handling on glass (grey: how much): a haze, heavier blotches,
/// smears. Shared by every part of the glass.
fn wine_glass_dust_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                let n = w as usize;
                let mut img = ctx.create_image_data(w, h);
                par_rows(&mut img.data, n * 4, |j, out| {
                    for i in 0..n {
                        let (x, y) = (i as f64 / 128.0, j as f64 / 128.0);
                        let d = wine_noise(71, x * 7.0, y * 7.0) * 0.6 + wine_noise(72, x * 29.0, y * 29.0) * 0.4;
                        let smear = (wine_noise(73, x * 3.0, y * 11.0) - 0.6).max(0.0) * 1.5;
                        let v = ((0.18 + 0.45 * d + smear) * 255.0).min(255.0) as u8;
                        out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
                    }
                });
                ctx.put_image_data(&img, 0, 0);
            })
        })
        .clone()
    })
}

/// The wine on the bowl, round (u, from +z toward +x) and up (v, over the
/// bowl part's height): its dried film from the crust in the bottom up to
/// the ring where it stood, its tears run down from that ring, fainter rings
/// where it went down in steps; fingerprints' grease; and now and then a
/// lipstick print on the rim. Transparent where there is none.
fn wine_glass_film_tex(wine: usize, tide: f64, lipstick: bool, salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let vy = |y: f64| (1.0 - (y - WINE_BOWL_Y.0) / (WINE_BOWL_Y.1 - WINE_BOWL_Y.0)) * h; // canvas y of a height
        let (fr, fg, fb, fa) = WINE_FILM[wine];
        let seed = salt ^ 0x5eed;
        let mut r = SplitMix64::new(salt, 0x7716);
        // the rings it left going down: the top one strongest
        let rings: Vec<(f64, f64)> = (0..3).map(|k| (tide - k as f64 * r.range(0.004, 0.009), [1.0, 0.45, 0.3][k])).collect();
        let (y_top, y_bot) = (vy(tide), vy(WINE_DREGS_TOP));
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = j as f64;
            for i in 0..n {
                let u = i as f64 / w;
                // the film below the ring, thicker toward the bottom and in
                // the tears that ran down
                let wob = (wine_noise(seed ^ 3, u * 22.0, 0.5) - 0.5) * 3.0;
                let mut film = 0.0;
                if y > y_top + wob {
                    let t = ((y - y_top) / (y_bot - y_top).max(1.0)).clamp(0.0, 1.0);
                    let tears = (wine_noise(seed ^ 5, u * 70.0, y / h * 2.5) - 0.62).max(0.0) * 3.0;
                    film = 0.22 + 0.4 * t * t + tears * 0.5;
                }
                for &(ry, k) in &rings {
                    let dy = y - (vy(ry) + wob * k);
                    film += k * 1.2 * (-(dy * dy) / 2.2).exp() * (0.7 + 0.3 * wine_noise(seed ^ 7, u * 30.0, 0.0));
                }
                let a = (film * fa).min(0.92);
                out[i * 4..i * 4 + 4].copy_from_slice(&[fr, fg, fb, (a * 255.0) as u8]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // fingerprints where it was held: soft ovals of grease, ridged
        for _ in 0..3 {
            let (cx, cy) = ((0.3 + 0.4 * r.next()) * w, (0.35 + 0.3 * r.next()) * h);
            ctx.save();
            ctx.translate(cx, cy);
            ctx.rotate(r.range(-0.6, 0.6));
            ctx.scale(1.0, 1.35);
            for k in 0..7 {
                ctx.set_stroke_style(&format!("rgba(196,198,190,{:.3})", 0.16 + 0.06 * (k % 2) as f64));
                ctx.set_line_width(1.0);
                ctx.begin_path();
                ctx.ellipse(0, 0, 3.0 + k as f64 * 1.6, 2.4 + k as f64 * 1.3, 0, 0, PI * 2.0);
                ctx.stroke();
            }
            ctx.restore();
        }
        if lipstick {
            // a lower lip's print on the rim, outside, its crease lines,
            // faded and gone brown-pink
            let cx = (0.18 + 0.08 * r.next()) * w;
            let cy = vy(0.1675);
            ctx.save();
            ctx.translate(cx, cy);
            ctx.scale(1.0, 0.55);
            let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, 16);
            gr.add_color_stop(0, "rgba(140,52,62,0.7)");
            gr.add_color_stop(0.7, "rgba(140,52,62,0.5)");
            gr.add_color_stop(1, "rgba(140,52,62,0)");
            ctx.set_fill_style(gr);
            ctx.begin_path();
            ctx.ellipse(0, 0, 16, 9, 0, 0, PI);
            ctx.fill();
            ctx.set_stroke_style("rgba(214,200,196,0.35)");
            ctx.set_line_width(0.8);
            for k in -5..=5 {
                ctx.begin_path();
                ctx.move_to(k as f64 * 2.6, 0.5);
                ctx.line_to(k as f64 * 2.4 + r.range(-0.6, 0.6), 7.5 - (k as f64).abs() * 0.6);
                ctx.stroke();
            }
            ctx.restore();
        }
    })
}

/// Wine run out of a glass and dried into the floor: a soaked patch of a
/// few lobes round its middle, darker at its edge where it dried last (the
/// ring), its edge wavering a little; a drop or two flung past it.
fn wine_stain_tex(wine: usize, salt: u32) -> Tex {
    canvas_tex_opt(256, 192, false, move |ctx, w, h| {
        let (fr, fg, fb, fa) = WINE_FILM[wine];
        let mut r = SplitMix64::new(salt, 0x57a1);
        let seed = salt ^ 0x2bad;
        // the lobes: (x, y, radius) in px, round the middle
        let mut lobes = vec![(w * 0.5, h * 0.5, h * 0.28)];
        for _ in 0..4 {
            let a = r.range(0.0, PI * 2.0);
            let d = h * r.range(0.08, 0.18);
            lobes.push((w * 0.5 + a.cos() * d * 1.3, h * 0.5 + a.sin() * d, h * r.range(0.14, 0.22)));
        }
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = j as f64;
            for i in 0..n {
                let x = i as f64;
                // the stain's field: a smooth union of its lobes (> 0 inside)
                let mut f: f64 = 0.0;
                for &(cx, cy, cr) in &lobes {
                    let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() / cr;
                    f += (1.0 - d * d).max(0.0).powi(2);
                }
                let edge =
                    f - 0.25 + 0.06 * (wine_noise(seed, x / 26.0, y / 26.0) - 0.5) + 0.02 * (wine_noise(seed ^ 1, x / 6.0, y / 6.0) - 0.5);
                if edge <= 0.0 {
                    out[i * 4 + 3] = 0;
                    continue;
                }
                // the ring where it dried last, and the soaked middle
                let ring = (-(edge / 0.035).powi(2)).exp();
                let body = (edge / 0.08).min(1.0) * (0.5 + 0.25 * wine_noise(seed ^ 3, x / 18.0, y / 18.0));
                let a = (body * 0.75 + ring * 0.55).min(1.0) * fa * 1.3;
                out[i * 4..i * 4 + 4].copy_from_slice(&[fr, fg, fb, (a.min(0.8) * 255.0) as u8]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // a few drops flung past it
        for _ in 0..5 {
            let a = r.range(0.0, PI * 2.0);
            let (x, y) = (w * 0.5 + a.cos() * w * r.range(0.3, 0.45), h * 0.5 + a.sin() * h * r.range(0.3, 0.45));
            let rr = r.range(1.2, 3.0);
            ctx.set_fill_style(&format!("rgba({fr},{fg},{fb},{:.2})", fa));
            ctx.begin_path();
            ctx.arc(x, y, rr, 0, PI * 2.0);
            ctx.fill();
        }
    })
}
