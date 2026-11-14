//! `place_pinned_photos`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Pinned photos: 12% chance per level, 1 polaroid pinned to a wall.
pub(crate) fn place_pinned_photos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let cands = wall_candidates(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let texs = photo_textures();
    let tex = texs[(rnd() * texs.len() as f64).floor() as usize].clone();
    // Per-weather color tint via material multiply.
    let photo_tint = photo_tint_by_weather(g.weather_key());
    let photo_mat = lambert!({ map: tex, color: photo_tint, transparent: true, side: DoubleSide });
    let photo = mesh(plane_geo!(0.14, 0.14), &photo_mat);
    let pin_mat = lambert!({ color: 0x2a1a10 });
    let pin = mesh(pg("PinnedPhotos__pin", box_geo!(0.008, 0.008, 0.006)), &pin_mat);
    // photo 2 mm off the wall; the pin's flange (back, 6 mm deep) on its front
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.002);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.002);
    let photo_height = 1.5 + rnd() * 0.35;
    if c.dx.abs() == 1 {
        let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        photo.rotation().y = rot;
        pin.rotation().y = rot;
        let jit_z = (rnd() - 0.5) * 1.2;
        photo.position().set(wall_x, photo_height, wall_z + jit_z);
        pin.position().set(wall_x - c.dx as f64 * 0.003, photo_height + 0.056, wall_z + jit_z);
    } else {
        let rot = if c.dz > 0 { PI } else { 0.0 };
        photo.rotation().y = rot;
        pin.rotation().y = rot;
        let jit_x = (rnd() - 0.5) * 1.2;
        photo.position().set(wall_x + jit_x, photo_height, wall_z);
        pin.position().set(wall_x + jit_x, photo_height + 0.056, wall_z - c.dz as f64 * 0.003);
    }
    // Slight tilt as if pinned crookedly.
    photo.rotation().z = (rnd() - 0.5) * 0.15;
    g.world.add(&photo);
    g.world.add(&pin);
    // 5% chance: cracked-glass overlay atop the photo.
    if rnd() < 0.05 {
        let crack_mat = basic_mat!({ map: cracked_glass_texture(), transparent: true, depthWrite: false, side: DoubleSide });
        let crack = mesh(plane_geo!(0.14, 0.14), &crack_mat);
        crack.position().copy(&photo.pos());
        crack.rotation().copy(&photo.rot());
        crack.position().x -= c.dx as f64 * 0.001;
        crack.position().z -= c.dz as f64 * 0.001;
        g.world.add(&crack);
        // 2-3 tiny floor shards beneath cracked photo.
        let shard_mat = basic_mat!({ color: shard_color_by_weather(g.weather_key()), transparent: true, opacity: 0.8, depthWrite: false, side: DoubleSide });
        let sh_count = 2 + (rnd() * 2.0).floor() as i32;
        let pp = photo.pos();
        for _sh in 0..sh_count {
            let shard = mesh(plane_geo!(0.010 + rnd() * 0.015, 0.014 + rnd() * 0.016), &shard_mat);
            shard.rotation().x = -PI / 2.0;
            shard.rotation().z = rnd() * PI * 2.0;
            let sh_ang = rnd() * PI * 2.0;
            let sh_rad = 0.06 + rnd() * 0.20;
            shard.position().set(pp.x + sh_ang.cos() * sh_rad, 0.003, pp.z + sh_ang.sin() * sh_rad);
            g.world.add(&shard);
        }
    }
}

/// Photo tint per weather
fn photo_tint_by_weather(key: &str) -> u32 {
    by_weather(key, 0x8ea0b4, 0xd0b070, 0xa06060, 0xa89870, 0xc4b898, 0xffffff)
}

#[derive(Clone, Copy)]
enum PhotoKind {
    Shapes(&'static [[f64; 4]]),
    Silhouette,
    Eyes,
    Doorway,
    NightSky,
    StarCluster,
    RainWindow,
    Hallway,
    MirrorRefl,
    FoundFootage,
    ClockFace,
    TvStatic,
    Negative,
    Halo,
}

/// Photo textures (16 variants)
fn photo_textures() -> Vec<Tex> {
    thread_local! { static V: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    V.with(|c| {
        c.get_or_init(|| {
            with_native(|| {
                use PhotoKind::*;
                let variants: [(&str, PhotoKind); 16] = [
                    ("#c8b898", Shapes(&[[24.0, 20.0, 24.0, 32.0], [50.0, 20.0, 24.0, 32.0]])), // 2 vague figures
                    ("#8a8878", Shapes(&[[10.0, 40.0, 76.0, 20.0]])),                           // horizon line
                    ("#a8988a", Shapes(&[[36.0, 22.0, 24.0, 40.0]])),                           // single central subject
                    ("#b0a8a0", Silhouette),
                    ("#4a4038", Eyes),
                    ("#2a2620", Doorway),
                    ("#0a0a12", NightSky),
                    ("#100815", StarCluster),
                    ("#3a4048", RainWindow),
                    ("#3a3020", Hallway),
                    ("#28241e", MirrorRefl),
                    ("#0a0a08", FoundFootage),
                    ("#3a3630", ClockFace),
                    ("#1a1a1a", TvStatic),
                    ("#c8c0b8", Negative),
                    ("#100a10", Halo),
                ];
                let mut textures = vec![];
                for &(bg, kind) in variants.iter() {
                    textures.push(canvas_tex_opt(96, 96, false, |ctx, w, h| {
                        ctx.set_fill_style(bg);
                        ctx.fill_rect(0, 0, w, h);
                        // Grunge speckle.
                        for _ in 0..30 {
                            ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.05 + rnd() * 0.05));
                            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
                        }
                        ctx.set_fill_style("#2a1a12");
                        match kind {
                            Silhouette => {
                                ctx.begin_path();
                                ctx.arc(w / 2.0, 32, 10, 0, PI * 2.0);
                                ctx.fill();
                                ctx.begin_path();
                                ctx.ellipse(w / 2.0, 66, 16, 26, 0, 0, PI * 2.0);
                                ctx.fill();
                            }
                            Doorway => {
                                ctx.set_fill_style("#d8c898");
                                ctx.fill_rect(30, 22, 36, 52);
                                ctx.set_stroke_style("#1a1610");
                                ctx.set_line_width(4);
                                ctx.stroke_rect(30, 22, 36, 52);
                                ctx.set_fill_style("#3a2818");
                                ctx.fill_rect(45, 44, 6, 22);
                            }
                            NightSky => {
                                ctx.set_fill_style("#e8e4c8");
                                for _ in 0..40 {
                                    let sx = rnd() * w;
                                    let sy = rnd() * h;
                                    let s_size = if rnd() < 0.15 { 2.0 } else { 1.0 };
                                    ctx.set_global_alpha(0.4 + rnd() * 0.5);
                                    ctx.fill_rect(sx, sy, s_size, s_size);
                                }
                                ctx.set_global_alpha(1);
                                ctx.set_fill_style("#f0eee0");
                                ctx.begin_path();
                                ctx.arc(68, 28, 7, 0, PI * 2.0);
                                ctx.fill();
                                ctx.set_fill_style("rgba(240,238,224,0.25)");
                                ctx.begin_path();
                                ctx.arc(68, 28, 11, 0, PI * 2.0);
                                ctx.fill();
                            }
                            Halo => {
                                let (cx, cy) = (w / 2.0, h / 2.0);
                                for r in 0..5 {
                                    let rad = 8.0 + r as f64 * 8.0;
                                    let alpha = 0.65 - r as f64 * 0.11;
                                    ctx.set_stroke_style(&format!("rgba(220,200,160,{})", alpha));
                                    ctx.set_line_width(1.6);
                                    ctx.begin_path();
                                    ctx.arc(cx, cy, rad, 0, PI * 2.0);
                                    ctx.stroke();
                                }
                                ctx.set_fill_style("rgba(240,220,180,0.85)");
                                ctx.begin_path();
                                ctx.arc(cx, cy, 3, 0, PI * 2.0);
                                ctx.fill();
                                let grd = ctx.create_radial_gradient(cx, cy, 12, cx, cy, 55);
                                grd.add_color_stop(0, "rgba(0,0,0,0)");
                                grd.add_color_stop(1, "rgba(0,0,0,0.4)");
                                ctx.set_fill_style(&grd);
                                ctx.fill_rect(0, 0, w, h);
                            }
                            Negative => {
                                ctx.set_fill_style("#f0e8d8");
                                ctx.begin_path();
                                ctx.arc(w / 2.0, 32, 10, 0, PI * 2.0);
                                ctx.fill();
                                ctx.begin_path();
                                ctx.ellipse(w / 2.0, 66, 16, 26, 0, 0, PI * 2.0);
                                ctx.fill();
                                for _ in 0..40 {
                                    ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.10 + rnd() * 0.15));
                                    ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
                                }
                                ctx.set_fill_style("rgba(30,20,15,0.55)");
                                for s in 0..6 {
                                    let sx = 6.0 + s as f64 * 15.0;
                                    ctx.fill_rect(sx, 2, 5, 4);
                                    ctx.fill_rect(sx, h - 6.0, 5, 4);
                                }
                            }
                            TvStatic => {
                                for _ in 0..800 {
                                    let sx = rnd() * w;
                                    let sy = rnd() * h;
                                    let gray = 40.0 + rnd() * 200.0;
                                    ctx.set_fill_style(&format!("rgba({},{},{},{})", gray, gray, gray, 0.4 + rnd() * 0.5));
                                    ctx.fill_rect(sx, sy, 1, 1);
                                }
                                ctx.set_fill_style("rgba(0,0,0,0.35)");
                                ctx.fill_rect(0, 40.0 + rnd() * 20.0, w, 4);
                            }
                            ClockFace => {
                                let (cx, cy) = (w / 2.0, h / 2.0);
                                ctx.set_fill_style("#d0c8b8");
                                ctx.begin_path();
                                ctx.arc(cx, cy, 34, 0, PI * 2.0);
                                ctx.fill();
                                ctx.set_stroke_style("#1a1408");
                                ctx.set_line_width(2);
                                for i in 0..12 {
                                    let a = (i as f64 / 12.0) * PI * 2.0 - PI / 2.0;
                                    ctx.begin_path();
                                    ctx.move_to(cx + a.cos() * 28.0, cy + a.sin() * 28.0);
                                    ctx.line_to(cx + a.cos() * 33.0, cy + a.sin() * 33.0);
                                    ctx.stroke();
                                }
                                ctx.set_stroke_style("#0a0805");
                                ctx.set_line_width(3);
                                let hour_ang = ((3.0 / 12.0) + 3.0 / 720.0) * PI * 2.0 - PI / 2.0;
                                ctx.begin_path();
                                ctx.move_to(cx, cy);
                                ctx.line_to(cx + hour_ang.cos() * 16.0, cy + hour_ang.sin() * 16.0);
                                ctx.stroke();
                                ctx.set_line_width(2);
                                let min_ang = (3.0 / 60.0) * PI * 2.0 - PI / 2.0;
                                ctx.begin_path();
                                ctx.move_to(cx, cy);
                                ctx.line_to(cx + min_ang.cos() * 24.0, cy + min_ang.sin() * 24.0);
                                ctx.stroke();
                                ctx.set_fill_style("#0a0805");
                                ctx.begin_path();
                                ctx.arc(cx, cy, 2.5, 0, PI * 2.0);
                                ctx.fill();
                            }
                            FoundFootage => {
                                for _ in 0..400 {
                                    let gx = rnd() * w;
                                    let gy = rnd() * h;
                                    let gv = 40.0 + rnd() * 120.0;
                                    ctx.set_fill_style(&format!("rgba({},{},{},{})", gv, gv, gv * 0.9, 0.10 + rnd() * 0.30));
                                    ctx.fill_rect(gx, gy, 1, 1);
                                }
                                let band_y = 20.0 + rnd() * 50.0;
                                ctx.set_fill_style("rgba(200,200,180,0.15)");
                                ctx.fill_rect(0, band_y, w, 3);
                                ctx.set_fill_style("rgba(15,10,10,0.55)");
                                ctx.begin_path();
                                ctx.ellipse(w / 2.0 + (rnd() - 0.5) * 12.0, h - 26.0, 14, 22, 0, 0, PI * 2.0);
                                ctx.fill();
                                ctx.set_fill_style("rgba(15,10,10,0.45)");
                                ctx.begin_path();
                                ctx.arc(w / 2.0 + (rnd() - 0.5) * 12.0, h - 50.0, 6, 0, PI * 2.0);
                                ctx.fill();
                                ctx.set_fill_style("rgba(180,180,160,0.05)");
                                let mut y = 0.0;
                                while y < h {
                                    ctx.fill_rect(0, y, w, 1);
                                    y += 3.0;
                                }
                            }
                            MirrorRefl => {
                                let (cx, cy) = (w / 2.0, h / 2.0);
                                let frames = [
                                    (12.0, 12.0, w - 24.0, h - 24.0, "#5a5044"),
                                    (22.0, 22.0, w - 44.0, h - 44.0, "#3e3a30"),
                                    (32.0, 32.0, w - 64.0, h - 64.0, "#22201c"),
                                ];
                                for &(fx, fy, fw, fh, col) in frames.iter() {
                                    ctx.set_stroke_style(col);
                                    ctx.set_line_width(2);
                                    ctx.stroke_rect(fx, fy, fw, fh);
                                }
                                ctx.set_fill_style("#0a0805");
                                ctx.begin_path();
                                ctx.arc(cx, cy - 4.0, 4, 0, PI * 2.0);
                                ctx.fill();
                                ctx.begin_path();
                                ctx.ellipse(cx, cy + 8.0, 6, 10, 0, 0, PI * 2.0);
                                ctx.fill();
                            }
                            Hallway => {
                                let (cx, cy) = (w / 2.0, h / 2.0);
                                ctx.set_stroke_style("#1a1408");
                                ctx.set_line_width(2);
                                for (ex, ey) in [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)] {
                                    ctx.begin_path();
                                    ctx.move_to(ex, ey);
                                    ctx.line_to(cx, cy);
                                    ctx.stroke();
                                }
                                let glow_pool =
                                    ["rgba(220,180,120,0.55)", "rgba(140,180,220,0.45)", "rgba(220,80,80,0.55)", "rgba(230,220,140,0.60)"];
                                let glow_inner = glow_pool[(rnd() * glow_pool.len() as f64).floor() as usize];
                                // the same colour with its alpha set to 0
                                let glow_outer = match glow_inner.rfind(',') {
                                    Some(i) => format!("{},0)", &glow_inner[..i]),
                                    None => glow_inner.to_string(),
                                };
                                let grd = ctx.create_radial_gradient(cx, cy, 4, cx, cy, 28);
                                grd.add_color_stop(0, glow_inner);
                                grd.add_color_stop(1, &glow_outer);
                                ctx.set_fill_style(&grd);
                                ctx.fill_rect(0, 0, w, h);
                                ctx.set_fill_style("#0a0805");
                                ctx.begin_path();
                                ctx.arc(cx, cy, 3, 0, PI * 2.0);
                                ctx.fill();
                            }
                            RainWindow => {
                                ctx.set_fill_style("#1a2028");
                                ctx.fill_rect(w / 2.0 - 2.0, 8, 4, h - 16.0);
                                ctx.fill_rect(8, h / 2.0 - 2.0, w - 16.0, 4);
                                ctx.set_stroke_style("#1a2028");
                                ctx.set_line_width(3);
                                ctx.stroke_rect(6, 6, w - 12.0, h - 12.0);
                                ctx.set_stroke_style("rgba(200,220,240,0.35)");
                                ctx.set_line_width(0.8);
                                for _ in 0..30 {
                                    let rx = rnd() * w;
                                    let ry0 = rnd() * (h - 20.0);
                                    let ry_len = 6.0 + rnd() * 18.0;
                                    ctx.begin_path();
                                    ctx.move_to(rx, ry0);
                                    ctx.line_to(rx - 1.0, ry0 + ry_len);
                                    ctx.stroke();
                                }
                                ctx.set_fill_style("rgba(230,240,250,0.5)");
                                for _ in 0..15 {
                                    ctx.fill_rect(rnd() * w, rnd() * h, 2, 2);
                                }
                            }
                            StarCluster => {
                                ctx.set_fill_style("#e8d8f0");
                                let (cx, cy) = (w / 2.0, h / 2.0);
                                for _ in 0..80 {
                                    let rx = (rnd() + rnd() + rnd()) / 3.0 - 0.5;
                                    let ry = (rnd() + rnd() + rnd()) / 3.0 - 0.5;
                                    let sx = cx + rx * w * 0.9;
                                    let sy = cy + ry * h * 0.9;
                                    let s_size = if rnd() < 0.20 { 2.0 } else { 1.0 };
                                    ctx.set_global_alpha(0.4 + rnd() * 0.55);
                                    ctx.fill_rect(sx, sy, s_size, s_size);
                                }
                                ctx.set_global_alpha(1);
                                ctx.set_fill_style("#fff0f8");
                                ctx.fill_rect(cx - 2.0, cy - 2.0, 3, 3);
                                ctx.fill_rect(cx + 4.0, cy - 5.0, 2, 2);
                                ctx.fill_rect(cx - 5.0, cy + 3.0, 2, 2);
                            }
                            Eyes => {
                                let eye_y = 46.0;
                                ctx.set_fill_style("#1a1008");
                                for eye_x in [34.0, 62.0] {
                                    ctx.begin_path();
                                    ctx.ellipse(eye_x, eye_y, 12, 8, 0, 0, PI * 2.0);
                                    ctx.fill();
                                }
                                ctx.set_fill_style("#050302");
                                for eye_x in [34.0, 62.0] {
                                    ctx.begin_path();
                                    ctx.arc(eye_x, eye_y, 4, 0, PI * 2.0);
                                    ctx.fill();
                                }
                                ctx.set_fill_style("rgba(240,230,210,0.85)");
                                for eye_x in [34.0, 62.0] {
                                    ctx.fill_rect(eye_x + 1.0, eye_y - 2.0, 2, 2);
                                }
                            }
                            Shapes(shapes) => {
                                ctx.set_fill_style("#3a2a20");
                                for s in shapes.iter() {
                                    ctx.fill_rect(s[0], s[1], s[2], s[3]);
                                }
                            }
                        }
                        // Corner fade.
                        let grd = ctx.create_radial_gradient(w / 2.0, h / 2.0, w * 0.3, w / 2.0, h / 2.0, w * 0.7);
                        grd.add_color_stop(0, "rgba(0,0,0,0)");
                        grd.add_color_stop(1, "rgba(0,0,0,0.4)");
                        ctx.set_fill_style(&grd);
                        ctx.fill_rect(0, 0, w, h);
                    }));
                }
                textures
            })
        })
        .clone()
    })
}
