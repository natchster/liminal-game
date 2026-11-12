//! `backrooms --tour DIR` (`docs/VISUAL_PLAYTEST.md`): the visual
//! playtest's renders. On each level, every view of the tour
//! (`game::debug::tour`) is rendered five times: the look a player gets
//! (the HUD on, the lamp lit where it is dark), and bare (no HUD layer)
//! cleared to magenta, to green, and to magenta with the camera turned by
//! `sight::JITTER` one way and the other. The pixel detectors
//! (`game::debug::sight`) read the bare four: *gaps* and *flicker*. Writes `DIR/L<n>/views.tsv` (a row a view:
//! its camera for `--shots-at`, the background, the gaps and the flicker),
//! each view's look as `NAME.png`, and for a view flagged its masks painted
//! over the bare render (`NAME_gaps.png`, `NAME_flicker.png`).
//!
//! `TOUR_DEBUG=NAME` prints, for the view of that name, every 400th gap
//! pixel's ray and what of the level it met (the tile, its ground, the
//! solid): how a detector's claim is checked.
//!
//! `--tour-plant hole|fight` puts a known fault at the tour's first stand
//! before its views are rendered (the detectors' known answers,
//! `tests/visual_tour.rs`): `hole` hides the thing the stand faces (a hole
//! where its wall was), `fight` stands two squares on one plane in front of
//! its eye, each cut differently (they fight for the same depth).
use crate::{draw_with, make_game, Options};
use engine::math::{Box3, Vec3};
use engine::render::Renderer;
use game::debug::sight::{self, Eye};
use game::debug::tour::{TourOpts, View};
use game::Game;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLCommandBuffer, MTLCreateSystemDefaultDevice, MTLTexture};

/// Background pixels are followed by a ray every this many pixels, both ways
/// (a crack a pixel wide is met every other pixel along it)
const GAP_STEP: usize = 2;
const MAGENTA: u32 = 0xff00ff;
const GREEN: u32 = 0x00ff00;

/// What a view's renders said
#[derive(Clone, Debug, Default)]
pub struct Seen {
    pub background: usize,
    pub gaps: usize,
    pub gap_blob: usize,
    pub flicker: usize,
    pub flicker_blob: usize,
}

pub fn tour(o: &Options, dir: &str) {
    let seed = o.seed.unwrap_or(4242);
    let mut g = make_game(o);
    g.launch_flags = if o.flags.is_empty() { format!("seed={seed}") } else { format!("seed={seed}&{}", o.flags) };
    let (w, h) = ((o.width * o.scale) as usize, (o.height * o.scale) as usize);
    g.view_width = o.width;
    g.view_height = o.height;
    g.scale_factor = o.scale;
    g.quality.max_pr = o.scale.min(2.0);
    g.quality.pr = g.quality.max_pr;
    let device = MTLCreateSystemDefaultDevice().expect("Metal device");
    let mut r = Renderer::new(device);
    g.boot();
    g.start_new_game();
    // the film grain settled as if the game had run a second (as `--dump-level`)
    for _ in 0..60 {
        g.update_grain(1.0 / 60.0);
    }
    let t = r.offscreen_target(w, h);
    let levels = if o.levels.is_empty() { vec![o.level.unwrap_or(1)] } else { o.levels.clone() };
    for &l in &levels {
        g.load_level(l);
        let views = g.tour_views(&TourOpts { budget: o.tour_budget, area: o.tour_area });
        if let (Some(p), Some(first)) = (&o.tour_plant, views.first()) {
            plant(&mut g, p, first);
        }
        let ldir = format!("{dir}/L{l}");
        std::fs::create_dir_all(&ldir).expect("create tour dir");
        let mut tsv = String::from("name\tlook\tedge\tcorner\tshots_at\tbackground\tgaps\tgap_blob\tflicker\tflicker_blob\n");
        let (mut flag_gaps, mut flag_flicker) = (0, 0);
        for v in &views {
            let (seen, look, a, gaps, flicker) = render_view(&mut g, &mut r, &t, (w, h), v);
            let (tw, th) = (w, h);
            engine::png::write_rgba(&format!("{ldir}/{}.png", v.name), tw as u32, th as u32, &look).expect("write png");
            if seen.gaps >= sight::FLAG_PX {
                flag_gaps += 1;
                let p = sight::paint(&a, &gaps, [255, 30, 30]);
                engine::png::write_rgba(&format!("{ldir}/{}_gaps.png", v.name), tw as u32, th as u32, &p).expect("write png");
            }
            if seen.flicker >= sight::FLAG_PX {
                flag_flicker += 1;
                let p = sight::paint(&a, &flicker, [30, 230, 255]);
                engine::png::write_rgba(&format!("{ldir}/{}_flicker.png", v.name), tw as u32, th as u32, &p).expect("write png");
            }
            tsv.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}:{:.4},{:.4},{:.4},{:.5},{:.5}\t{}\t{}\t{}\t{}\t{}\n",
                v.name,
                v.look,
                v.edge,
                v.corner,
                v.name,
                v.x,
                v.y,
                v.z,
                v.yaw,
                v.pitch,
                seen.background,
                seen.gaps,
                seen.gap_blob,
                seen.flicker,
                seen.flicker_blob
            ));
        }
        std::fs::write(format!("{ldir}/views.tsv"), tsv).expect("write views.tsv");
        println!("level {l}: {} views, gaps in {flag_gaps}, flicker in {flag_flicker}: {ldir}/views.tsv", views.len());
    }
}

/// One view's five renders and what the detectors made of them: (what was
/// seen, the look, the bare render, the gaps mask, the flicker mask)
#[allow(clippy::type_complexity)]
fn render_view(
    g: &mut Game,
    r: &mut Renderer,
    t: &ProtocolObject<dyn MTLTexture>,
    (w, h): (usize, usize),
    v: &View,
) -> (Seen, Vec<u8>, Vec<u8>, Vec<bool>, Vec<bool>) {
    // the body where the view stands, one step of play, then the eye exactly
    g.player.pos = Vec3::new(v.x, g.floor_y_at(v.x, v.z) + game::prelude::EYE, v.z);
    g.player.yaw = v.yaw;
    g.player.pitch = v.pitch;
    // seen as a player sees it in the dark: by the lamp (as the census)
    g.player.flashlight_on = g.region_light_at(g.tx(v.x), g.tz(v.z)) < 0.3;
    g.player.battery = game::prelude::BAT_MAX;
    g.prop_cull.t = 0.0;
    g.fixed_step_dt = 1.0 / 60.0;
    g.frame(1.0 / 60.0);
    g.fixed_step_dt = 0.0;
    let aim = |g: &mut Game, dy: f64, dp: f64| {
        g.camera.position().set(v.x, v.y, v.z);
        g.camera.rotation().x = v.pitch + dp;
        g.camera.rotation().y = v.yaw + dy;
        g.camera.rotation().z = 0.0;
    };
    aim(g, 0.0, 0.0);
    // (no toast: the tour walks over pickups)
    g.msg_t = 0.0;
    g.ui.msg.set_opacity(0.0);
    g.ui.msg.set_text("");
    g.sim_frozen = true;
    let mut shot = |g: &mut Game, hud: bool| {
        let cb = draw_with(g, r, t, w, h, hud);
        cb.commit();
        cb.waitUntilCompleted();
        r.read_texture(t)
    };
    let (_, _, look) = shot(g, true);
    let keep = g.background;
    g.background = engine::math::Color::hex(MAGENTA);
    let (_, _, a) = shot(g, false);
    g.background = engine::math::Color::hex(GREEN);
    let (_, _, b) = shot(g, false);
    g.background = engine::math::Color::hex(MAGENTA);
    aim(g, sight::JITTER, sight::JITTER);
    let (_, _, plus) = shot(g, false);
    aim(g, -sight::JITTER, -sight::JITTER);
    let (_, _, minus) = shot(g, false);
    aim(g, 0.0, 0.0);
    g.background = keep;
    g.sim_frozen = false;
    g.player.flashlight_on = false;
    let bg = sight::background_mask(&a, &b);
    let eye = Eye { x: v.x, y: v.y, z: v.z, yaw: v.yaw, pitch: v.pitch, fov: g.camera.camera().fov.to_radians(), w, h };
    let gaps = sight::gap_mask(g, &eye, &bg, GAP_STEP);
    if std::env::var("TOUR_DEBUG").ok().as_deref() == Some(v.name.as_str()) {
        let mut k = 0;
        for (i, &on) in gaps.iter().enumerate() {
            if !on {
                continue;
            }
            k += 1;
            if k % 400 != 1 {
                continue;
            }
            let (px, py) = ((i % w) as f64, (i / w) as f64);
            let d = eye.ray(px, py);
            let t = g.ray_meets_level((v.x, v.y, v.z), d, sight::GAP_REACH).unwrap_or(-1.0);
            let (hx, hy, hz) = (v.x + d.0 * t, v.y + d.1 * t, v.z + d.2 * t);
            let (tx, tz) = (g.tx(hx), g.tz(hz));
            let gr = g.ground.get(g.t_idx(tx, tz)).copied();
            let sid = g.solids.as_ref().map(|s| game::level::archgen::point_in_solid(s, g.to_u(hx), g.to_v(hz), hy)).unwrap_or(-9);
            eprintln!(
                "px {px},{py} dir {:.3},{:.3},{:.3} t {t:.2} hit {hx:.2},{hy:.2},{hz:.2} tile {tx},{tz} ground {gr:?} solid {sid} wall {}",
                d.0,
                d.1,
                d.2,
                g.is_wall_tile(tx, tz)
            );
        }
    }
    let flicker = sight::flicker_mask(&a, &plus, &minus, &bg, w, h);
    let count = |m: &[bool]| m.iter().filter(|&&b| b).count();
    let largest = |m: &[bool]| sight::blobs(m, w, h).first().map(|b| b.n).unwrap_or(0);
    let seen = Seen {
        background: count(&bg),
        gaps: count(&gaps),
        gap_blob: largest(&gaps),
        flicker: count(&flicker),
        flicker_blob: largest(&flicker),
    };
    (seen, look, a, gaps, flicker)
}

/// A known fault at the tour's first stand (`--tour-plant`)
fn plant(g: &mut Game, what: &str, v: &View) {
    let (fx, fz) = (-v.yaw.sin(), -v.yaw.cos());
    match what {
        // the smallest thing whose bounds hold a point just behind the face
        // the stand looks at, hidden: a hole where it was
        "hole" => {
            let p = Vec3::new(v.x + fx * 0.35, v.y, v.z + fz * 0.35);
            let mut best: Option<(f64, engine::scene::Obj)> = None;
            g.scene.traverse_visible(&mut |o| {
                if !o.is_mesh() {
                    return;
                }
                let mut b = Box3::empty();
                b.set_from_object(o);
                if b.contains_point(&p) {
                    let s = b.get_size();
                    let vol = s.x * s.y * s.z;
                    if best.as_ref().is_none_or(|(bv, _)| vol < *bv) {
                        best = Some((vol, o.clone()));
                    }
                }
            });
            match best {
                Some((_, o)) => {
                    println!("planted a hole: hid {:?}", o.name());
                    o.set_visible(false);
                }
                None => println!("planted no hole: nothing behind {:?}", v.name),
            }
        }
        // two squares on one plane 0.15 m before the eye, facing it, one cut
        // turned 45° in the plane: their depths differ in the last bits
        "fight" => {
            for (k, color) in [(0, 0xb03020u32), (1, 0x2040c0u32)] {
                let geo = engine::geometry::plane_geometry(0.12, 0.12, 3.0, 3.0);
                if k == 1 {
                    geo.rotate_z(std::f64::consts::FRAC_PI_4);
                }
                let m = engine::material::Mat::basic();
                m.color(color);
                let q = engine::scene::mesh(geo, m);
                q.set_name(&format!("tour_plant_fight_{k}"));
                q.position().set(v.x + fx * 0.15, v.y, v.z + fz * 0.15);
                q.rotation().y = v.yaw;
                g.scene.add(&q);
            }
            println!("planted a fight before {}", v.name);
        }
        other => panic!("--tour-plant {other}: hole or fight"),
    }
}
