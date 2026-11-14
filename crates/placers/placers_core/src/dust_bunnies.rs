//! `place_dust_bunnies`
use crate::shared::mark_tex;
use crate::shared::vary_by;
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::mark_mat;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;
use placers_kit::wet_floor::floor_kind;

// Dust bunnies: 10% chance per level. Where no air moves, what a floor
// sheds (textile lint, hair, skin, grit) felts itself into clumps held by
// static and their own fibres (`docs/research/props-remodelling/floor_marks_and_dust.md`):
// five to ten of them, from a small wisp to a sprawl of fluff 6 cm long
// (family `floor_marks_and_dust`), grey with a fleck of a jumper's colour here and there, on a
// thin film of the dust they gather from and the loose fibres and hairs
// round them.
pub(crate) fn place_dust_bunnies(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let hard = floor_kind(g, x, z) != 0;
    let grp = group();
    let fluff = dust_fluff_tex();
    let fringe = dust_fringe_tex();
    let n_bunnies = 5 + (rnd() * 6.0).floor() as i32;
    let mut spots = vec![];
    for _i in 0..n_bunnies {
        // the old ball's draws: its colour (which bunny), its radius (how big,
        // within 13%), its three scales (its turn, its grey, and one kept) and
        // where it lay
        let kind = ((rnd() * 3.0).floor() as usize).min(2);
        let k = 0.87 + (rnd() * 0.006) / 0.006 * 0.26;
        let yaw = rnd() * PI * 2.0;
        let grey = rnd();
        let _ = rnd(); // keeps the old draw (its depth's scale)
        let (bx, bz) = ((rnd() - 0.5) * 0.40, (rnd() - 0.5) * 0.40);
        let v = 0.84 + 0.16 * grey;
        let byte = |f: f64| ((f * 255.0).round() as u32).min(255);
        let tint = (byte(v) << 16) | (byte(v) << 8) | byte(v * 0.99);
        let bunny = group();
        let tag = ["A", "B", "C"][kind];
        if let Some(geo) = prop_part_as_modelled(&format!("Real__dustBunny{tag}")) {
            bunny.add(&mesh(geo, &lambert!({ map: &fluff, color: tint })));
        }
        // the loose fuzz round it: its fibres cut out of a shell
        if let Some(geo) = prop_part_as_modelled(&format!("Real__dustBunnyFuzz{tag}")) {
            bunny.add(&mesh(geo, &lambert!({ map: &fringe, color: tint, alphaTest: 0.5, side: DoubleSide })));
        }
        bunny.scale().set_scalar(k);
        bunny.rotation().y = yaw;
        bunny.position().set(bx, 0.0, bz);
        grp.add(&bunny);
        spots.push((bx, bz, 0.03 * k));
    }
    // the film of dust they lie on, and its loose fibres
    let film = mesh(
        {
            let geo = plane_geo!(0.46, 0.46);
            geo.rotate_x(-PI / 2.0);
            geo
        },
        mark_mat(&dust_film_tex(spots, hard, vary_by(x, z, 0x510))),
    );
    film.position().y = 0.0015;
    grp.add(&film);
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

// The loose fibres round a dust bunny: lint every way on nothing, cut out
// by the alpha test (laid on its fuzz shell by its box faces); made once,
// shared by every bunny
game::lazy_static_tex!(
    dust_fringe_tex,
    canvas_tex(160, 160, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        ctx.set_line_cap("round");
        for k in 0..520 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI * 2.0;
            let l = 8.0 + rnd() * 30.0;
            let v = 120.0 + rnd() * 90.0;
            ctx.set_stroke_style(if k % 41 == 0 { "rgb(104,112,136)".to_string() } else { format!("rgb({v:.0},{v:.0},{:.0})", v - 3.0) });
            ctx.set_line_width(0.9 + rnd() * 1.1);
            ctx.begin_path();
            ctx.move_to(x, y);
            let bend = (rnd() - 0.5) * 1.6;
            ctx.quadratic_curve_to(x + (a + bend).dcos() * l * 0.5, y + (a + bend).dsin() * l * 0.5, x + a.dcos() * l, y + a.dsin() * l);
            ctx.stroke();
        }
    })
);

// The fibres a dust bunny is felted of, laid on its parts by their box faces:
// grey and white lint, dark hairs, a fleck of coloured wool, grit; made
// once, shared by every bunny
game::lazy_static_tex!(
    dust_fluff_tex,
    canvas_tex(192, 192, |ctx, w, h| {
        ctx.set_fill_style("rgb(128,128,127)");
        ctx.fill_rect(0, 0, w, h);
        // the felt: fibres every way, light and dark greys
        ctx.set_line_cap("round");
        for k in 0..900 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI * 2.0;
            let l = 6.0 + rnd() * 26.0;
            let v = 110.0 + rnd() * 110.0;
            let tint = if k % 37 == 0 {
                // a fibre off a jumper or a carpet: faded blue, red, ochre
                ["rgba(96,108,140,0.6)", "rgba(140,90,82,0.6)", "rgba(150,128,86,0.6)"][(rnd() * 3.0) as usize % 3].to_string()
            } else {
                format!("rgba({v:.0},{v:.0},{:.0},{:.2})", v - 3.0, 0.25 + rnd() * 0.4)
            };
            ctx.set_stroke_style(&tint);
            ctx.set_line_width(0.6 + rnd() * 1.2);
            ctx.begin_path();
            ctx.move_to(x, y);
            let bend = (rnd() - 0.5) * 1.2;
            ctx.quadratic_curve_to(x + (a + bend).dcos() * l * 0.5, y + (a + bend).dsin() * l * 0.5, x + a.dcos() * l, y + a.dsin() * l);
            ctx.stroke();
        }
        // hairs, dark and long
        for _ in 0..24 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI * 2.0;
            let l = 40.0 + rnd() * 80.0;
            ctx.set_stroke_style(format!("rgba(40,34,30,{:.2})", 0.35 + rnd() * 0.3));
            ctx.set_line_width(0.6);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.quadratic_curve_to(x + (a + 0.6).dcos() * l * 0.5, y + (a + 0.6).dsin() * l * 0.5, x + a.dcos() * l, y + a.dsin() * l);
            ctx.stroke();
        }
        // grit and the dark of the felt's hollows
        for _ in 0..260 {
            let (x, y) = (rnd() * w, rnd() * h);
            let v = 50.0 + rnd() * 50.0;
            ctx.set_fill_style(format!("rgba({v:.0},{:.0},{:.0},{:.2})", v - 4.0, v - 10.0, 0.2 + rnd() * 0.4));
            ctx.fill_rect(x, y, 1.0 + rnd() * 1.5, 1.0 + rnd() * 1.5);
        }
    })
);

/// The dust under a clutch of dust bunnies on a plane 0.46 m square: a thin
/// grey film thickest round them (`spots`, m, from its middle: x, z and how
/// big), and the fibres and hairs come loose from them; on carpet the film
/// hardly shows, the fibres do
fn dust_film_tex(spots: Vec<(f64, f64, f64)>, hard: bool, vary: u32) -> Tex {
    let side = 0.46;
    mark_tex(256.0, 256.0, [0.5, 0.495, 0.485], vary, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32;
        let s = side / w;
        let film_k = if hard { 0.3 } else { 0.1 };
        let mut img = ctx.create_image_data(w, h);
        let n = w as usize;
        let spots2 = spots.clone();
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = (j as f64 + 0.5) * s - side / 2.0;
            for i in 0..n {
                let x = (i as f64 + 0.5) * s - side / 2.0;
                let mut near = 0.0f64;
                for &(bx, bz, r) in &spots2 {
                    near += (-((x - bx).dhypot(y - bz) / (r * 2.5 + 0.03)).dpowi(2)).dexp();
                }
                let cloud = fbm(seed, x * 18.0, y * 18.0, 4);
                let edge = sstep(0.0, 0.08, side / 2.0 - x.abs().max(y.abs()));
                let a = (near.min(1.0) * 0.8 + 0.25 * cloud) * sstep(0.25, 0.75, cloud + 0.3 * near.min(1.0)) * film_k * edge;
                let v = 0.48 + 0.06 * vn(seed ^ 0x9, x * 300.0, y * 300.0);
                let c = [v, v * 0.99, v * 0.97, a];
                for (k, q) in c.iter().enumerate() {
                    out[i * 4 + k] = (q.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // the loose fibres and hairs, near the bunnies mostly
        ctx.set_line_cap("round");
        let to_px = |m: f64| (m + side / 2.0) / s;
        for k in 0..70 {
            let (bx, bz, r) = spots[k % spots.len()];
            let a = rnd() * PI * 2.0;
            let d = r * (0.6 + 2.5 * rnd() * rnd());
            let (x, y) = (to_px(bx + a.dcos() * d), to_px(bz + a.dsin() * d));
            let hair = k % 9 == 0;
            let l = if hair { 30.0 + rnd() * 40.0 } else { 5.0 + rnd() * 14.0 };
            let b = rnd() * PI * 2.0;
            let v = 110.0 + rnd() * 70.0;
            ctx.set_stroke_style(if hair {
                format!("rgba(42,38,36,{:.2})", 0.35 + rnd() * 0.25)
            } else {
                format!("rgba({v:.0},{v:.0},{:.0},{:.2})", v - 3.0, 0.2 + rnd() * 0.3)
            });
            ctx.set_line_width(if hair { 0.5 } else { 0.5 + rnd() * 0.5 });
            ctx.begin_path();
            ctx.move_to(x, y);
            let bend = (rnd() - 0.5) * 1.6;
            ctx.quadratic_curve_to(x + (b + bend).dcos() * l * 0.5, y + (b + bend).dsin() * l * 0.5, x + b.dcos() * l, y + b.dsin() * l);
            ctx.stroke();
        }
    })
}
