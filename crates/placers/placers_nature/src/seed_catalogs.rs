//! `place_seed_catalogs`
use crate::shared::blob;
use crate::shared::css;
use crate::shared::over;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use game::visuals::pictures::Window;
use placers_kit::garden::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::paint::rgb;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Framed vintage seed catalogue: 3% chance per level.
pub(crate) fn place_seed_catalogs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    // A seed catalogue's cover of 1937 (9 x 6 in) framed in a stained pine
    // moulding with a cream mat and glass (Blender family `garden_things`), its paper
    // gone yellow and foxed, the glass dusty
    let catalog = group();
    let mount_y = 1.65;
    let body = group();
    // the frame's back on the wall (0.013 behind the old box's middle)
    body.position().set(0, mount_y, -0.013);
    catalog.add(&body);
    // askew on its nail (the old draw)
    catalog.rotation().z = (rnd() - 0.5) * 0.05;
    catalog.rotation().y = face_wall(dx, dz);
    catalog.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    let p = catalog.pos();
    let vary = pictures::vary_of(p.x, p.z, 0.47);
    put(
        &body,
        "Real__seedCatFrame",
        &lambert!({ map: &pictures::moulding_tex(SC_SIZE, &SC_FACE, Outline::Rect, Finish::Wood([0x6a, 0x48, 0x2c]), 1100.0, vary) }),
    );
    put(
        &body,
        "Real__seedCatMat",
        &lambert!({ map: &pictures::mat_tex(SC_MAT, Window::Rect(0.136, 0.212, 0.0), [0xb8, 0xaa, 0x88], vary, None) }),
    );
    put(&body, "Real__seedCatCover", &lambert!({ map: &seed_cover_tex(vary) }));
    put(&body, "Real__seedCatBack", &lambert!({ color: 0x4a3c2e }));
    // the glass: a little dust, the room faint in it
    let glass = pictures::glass_mat(vary, 0.05);
    glass.r().patch_uniforms.borrow_mut().f[0] = 0.08;
    put(&body, "Real__seedCatGlass", &glass);
    g.world.add(&catalog);
}

// ========================================================== seed catalogs
/// The seed catalogue's frame: its face from the outer edge to the sight
/// (d, z), as `seed_cat_frame` in `nature/garden_things.py`
static SC_FACE: [(f64, f64); 8] = [
    (0.0, 0.0145),
    (0.0012, 0.0168),
    (0.0040, 0.0182),
    (0.0080, 0.0187),
    (0.0120, 0.0180),
    (0.0150, 0.0160),
    (0.0178, 0.0122),
    (0.020, 0.0112),
];

const SC_SIZE: (f64, f64) = (0.220, 0.300);

const SC_MAT: (f64, f64) = (0.195, 0.275);

/// The cover of a seed house's catalogue of 1937 (the house invented),
/// printed in colour like the chromolithographed covers of the time:
/// tomatoes on the vine, a cabbage, carrots and pea pods under the title;
/// the paper gone yellow, the inks faded, spotted with foxing
fn seed_cover_tex(vary: u32) -> Tex {
    canvas_tex_opt(456, 687, false, move |ctx, w, h| {
        let seed = vary ^ 0x5eed;
        let paper = rgb(0xab9a72);
        ctx.set_fill_style(css(paper, 1.0));
        ctx.fill_rect(0, 0, w, h);
        let ink = rgb(0x1e281a);
        let red = rgb(0x96281e);
        // the border: a double rule and corner squares
        ctx.set_stroke_style(css(ink, 1.0));
        ctx.set_line_width(3);
        ctx.stroke_rect(30, 30, w - 60.0, h - 60.0);
        ctx.set_line_width(1);
        ctx.stroke_rect(37, 37, w - 74.0, h - 74.0);
        // the house and the title
        ctx.set_fill_style(css(ink, 1.0));
        ctx.set_text_align("center");
        ctx.set_text_baseline("alphabetic");
        ctx.set_font("bold 22px Copperplate");
        ctx.fill_text_max("WARBECK'S", w / 2.0, 92, w - 120.0);
        ctx.set_fill_style(css(red, 1.0));
        ctx.set_font("bold 54px Times New Roman");
        ctx.fill_text_max("Garden Book", w / 2.0, 146, w - 100.0);
        ctx.set_fill_style(css(ink, 1.0));
        ctx.set_font("italic 20px Times New Roman");
        ctx.fill_text_max("Vegetable \u{b7} Flower \u{b7} Field Seeds", w / 2.0, 176, w - 110.0);
        // the year in a banner
        ctx.set_fill_style(css(rgb(0x3f6a3a), 1.0));
        ctx.fill_rect(w / 2.0 - 60.0, 188, 120, 34);
        ctx.set_fill_style(css(paper, 1.0));
        ctx.set_font("bold 28px Times New Roman");
        ctx.fill_text_max("1937", w / 2.0, 215, 110);
        // the picture: a still life of the garden's yield
        let mut r = SplitMix64::new(11, 0xc0f);
        let outline = |ctx: &mut Canvas| {
            ctx.set_stroke_style("rgba(52,32,20,0.7)");
            ctx.set_line_width(1.2);
            ctx.stroke();
        };
        // the cabbage, back right
        let (cx, cy) = (w * 0.66, 360.0);
        for k in 0..14 {
            let a = k as f64 / 14.0 * PI * 2.0;
            let rr = 70.0 - (k % 3) as f64 * 8.0;
            let (lx, ly) = (cx + a.dcos() * rr * 0.5, cy + a.dsin() * rr * 0.4);
            let gr = ctx.create_radial_gradient(lx - 10.0, ly - 12.0, 4, lx, ly, 52);
            gr.add_color_stop(0, &css(rgb(0x94b06a), 1.0));
            gr.add_color_stop(1, &css(rgb(0x34582a), 1.0));
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.ellipse(lx, ly, 46, 34, a, 0, 7);
            ctx.fill();
            outline(ctx);
        }
        let gr = ctx.create_radial_gradient(cx - 12.0, cy - 16.0, 4, cx, cy, 48);
        gr.add_color_stop(0, &css(rgb(0xc2d090), 1.0));
        gr.add_color_stop(1, &css(rgb(0x6a8a4a), 1.0));
        ctx.set_fill_style(&gr);
        ctx.begin_path();
        ctx.arc(cx, cy, 44, 0, 7);
        ctx.fill();
        outline(ctx);
        ctx.set_stroke_style("rgba(240,245,220,0.55)");
        ctx.set_line_width(1.5);
        for k in 0..9 {
            let a = k as f64 / 9.0 * PI * 2.0 + 0.3;
            ctx.begin_path();
            ctx.move_to(cx + a.dcos() * 8.0, cy + a.dsin() * 8.0);
            ctx.quadratic_curve_to(
                cx + (a + 0.3).dcos() * 40.0,
                cy + (a + 0.3).dsin() * 34.0,
                cx + (a + 0.2).dcos() * 66.0,
                cy + (a + 0.2).dsin() * 52.0,
            );
            ctx.stroke();
        }
        // carrots across the front, their tops feathery
        for k in 0..4 {
            let (x0, y0) = (90.0 + k as f64 * 26.0, 470.0 + k as f64 * 10.0);
            let (x1, y1) = (x0 + 190.0, y0 + 40.0 - k as f64 * 6.0);
            let (nx, ny) = (-(y1 - y0), x1 - x0);
            let l = nx.dhypot(ny);
            let (nx, ny) = (nx / l * 13.0, ny / l * 13.0);
            let gr = ctx.create_linear_gradient(x0 + nx, y0 + ny, x0 - nx, y0 - ny);
            gr.add_color_stop(0, &css(rgb(0xd8842e), 1.0));
            gr.add_color_stop(1, &css(rgb(0x94461a), 1.0));
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.move_to(x0 + nx, y0 + ny);
            ctx.quadratic_curve_to((x0 + x1) / 2.0 + nx * 0.7, (y0 + y1) / 2.0 + ny * 0.7, x1, y1);
            ctx.quadratic_curve_to((x0 + x1) / 2.0 - nx * 0.7, (y0 + y1) / 2.0 - ny * 0.7, x0 - nx, y0 - ny);
            ctx.close_path();
            ctx.fill();
            outline(ctx);
            ctx.set_stroke_style("rgba(120,60,20,0.5)");
            ctx.set_line_width(1);
            for j in 1..8 {
                let t = j as f64 / 9.0;
                let (mx, my) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
                let s = 1.0 - t;
                ctx.begin_path();
                ctx.move_to(mx + nx * 0.6 * s, my + ny * 0.6 * s);
                ctx.line_to(mx + nx * 0.2 * s, my + ny * 0.2 * s);
                ctx.stroke();
            }
            ctx.set_stroke_style(css(rgb(0x4a7a34), 1.0));
            ctx.set_line_width(1.6);
            for j in 0..7 {
                let a = PI + 0.9 + j as f64 * 0.12 + r.range(-0.05, 0.05);
                ctx.begin_path();
                ctx.move_to(x0, y0);
                ctx.quadratic_curve_to(x0 + a.dcos() * 30.0, y0 + a.dsin() * 30.0 - 10.0, x0 + a.dcos() * 60.0, y0 + a.dsin() * 56.0);
                ctx.stroke();
            }
        }
        // tomatoes on the vine, left
        for (k, (tx, ty, tr)) in
            [(130.0, 330.0, 40.0), (190.0, 352.0, 34.0), (112.0, 400.0, 36.0), (174.0, 418.0, 30.0)].into_iter().enumerate()
        {
            let gr = ctx.create_radial_gradient(tx - tr * 0.35, ty - tr * 0.4, tr * 0.1, tx, ty, tr);
            gr.add_color_stop(0, &css(rgb(0xd8806a), 1.0));
            gr.add_color_stop(0.35, &css(rgb(0xb8341f), 1.0));
            gr.add_color_stop(1, &css(rgb(0x641410), 1.0));
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.ellipse(tx, ty, tr, tr * 0.86, 0.1 * k as f64, 0, 7);
            ctx.fill();
            outline(ctx);
            // the calyx
            for j in 0..5 {
                let a = j as f64 / 5.0 * PI * 2.0;
                blob(ctx, tx + a.dcos() * 7.0, ty - tr * 0.8 + a.dsin() * 3.0, 7.0, 0.35, a, rgb(0x3f6a2e), 1.0);
            }
            blob(ctx, tx - tr * 0.4, ty - tr * 0.45, tr * 0.25, 0.6, -0.5, [1.0, 0.95, 0.9], 0.6);
        }
        // pea pods
        for k in 0..3 {
            let (x0, y0) = (250.0 + k as f64 * 20.0, 520.0 - k as f64 * 14.0);
            ctx.set_fill_style(css(mix(rgb(0x6a9a46), rgb(0x9cc06a), k as f64 / 3.0), 1.0));
            ctx.begin_path();
            ctx.move_to(x0, y0);
            ctx.quadratic_curve_to(x0 + 50.0, y0 - 26.0, x0 + 110.0, y0 - 8.0);
            ctx.quadratic_curve_to(x0 + 54.0, y0 + 6.0, x0, y0);
            ctx.fill();
            outline(ctx);
            for j in 0..5 {
                let t = 0.2 + j as f64 * 0.14;
                blob(ctx, x0 + 110.0 * t, y0 - 12.0 * (1.0 - (2.0 * t - 1.0).dpowi(2)) - 2.0, 6.0, 1.0, 0.0, rgb(0xc2dc8c), 0.5);
            }
        }
        // the house's name at the foot
        ctx.set_fill_style(css(ink, 1.0));
        ctx.set_font("bold 17px Copperplate");
        ctx.fill_text_max("WARBECK SEED CO.", w / 2.0, 598, w - 120.0);
        ctx.set_font("italic 15px Times New Roman");
        ctx.fill_text_max("Columbus, Ohio \u{b7} Established 1884", w / 2.0, 622, w - 120.0);
        // aged: the paper yellowed and browned toward its edges, the inks
        // faded toward it, foxing, a tide line where it once got wet
        let mut s = SplitMix64::new(vary, 0xf0c5);
        let fox: Vec<(f64, f64, f64)> = (0..(s.range(18.0, 40.0) as usize)).map(|_| (s.next(), s.next(), s.range(0.002, 0.009))).collect();
        let tide = (s.range(0.0, 1.0), s.range(0.0, 0.3), s.range(0.18, 0.35));
        over(ctx, w, h, move |u, v, c| {
            let n = fbm(seed, u * 7.0, v * 10.0, 4);
            let mut o = mix(c, paper, 0.08 + 0.08 * n);
            o = mix(o, [0.64, 0.5, 0.3], 0.12 + 0.1 * n);
            let edge = sstep(0.09, 0.0, u.min(1.0 - u).min(v.min(1.0 - v)));
            o = mix(o, [0.5, 0.37, 0.22], edge * 0.6);
            for &(fx, fy, fr) in &fox {
                let d = (u - fx).dhypot((v - fy) * 1.5) / fr;
                o = mix(o, [0.58, 0.42, 0.26], sstep(1.0, 0.3, d) * 0.6);
            }
            let d = (u - tide.0).dhypot(v - tide.1) / tide.2 + 0.2 * (fbm(seed ^ 6, u * 9.0, v * 9.0, 2) - 0.5);
            o = mix(o, o.map(|k| k * 0.86), sstep(1.0, 0.96, d) * 0.4 + sstep(0.93, 1.0, d) * sstep(1.04, 1.0, d) * 0.5);
            o
        });
    })
}
