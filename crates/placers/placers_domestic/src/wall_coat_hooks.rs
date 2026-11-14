//! `place_wall_coat_hooks`
use game::prelude::*;
use placers_kit::geometry::clone_obj;
use placers_kit::place::mount_slid_ref;
use placers_kit::place::ri;
use placers_kit::place::shuffle;
use placers_kit::place::wall_candidates;

// Wall coat hook: 15% chance per level, 1-2 hooks at 1.7-1.9 m
// (Blender batch `r13b`, the hook of `r06`): a cast iron double coat-and-hat
// hook screwed to an oak block on the wall; now and then 2-3 more along the
// same wall (a cloakroom's row), now and then a knitted scarf left thrown
// over the first's arm. (A coat is wider than the cloth that hung there,
// and would change where the wall's face is found: see docs/research/props-remodelling/coat_hook_broom_apron_and_rugs.md.)
pub(crate) fn place_wall_coat_hooks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let iron = phong!({ color: 0x1c1b1a, specular: 0x1a1a1a, shininess: 22 });
    let oak = lambert!({ map: &hook_block_tex() });
    let steel = phong!({ color: 0x5e5c57, specular: 0x2a2a2a, shininess: 30 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        let hook = hook_on_block(&iron, &oak, &steel);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.004);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.004);
        let hook_height = 1.7 + rnd() * 0.20;
        mount_slid_ref(&hook, &c, wall_x, wall_z, hook_height, 1.4);
        g.world.add(&hook);
        // 20% chance: 2-3 additional sibling hooks in a row along
        // the same wall (cloakroom cluster).
        if rnd() < 0.20 {
            let sib_count = 2 + (rnd() * 2.0).floor() as i32;
            let sib_step = 0.14 + rnd() * 0.06; // spacing per sibling
            let sib_dir = if rnd() < 0.5 { -1.0 } else { 1.0 };
            for sib in 1..=sib_count {
                let s_hook = clone_obj(&hook);
                let hp = hook.pos();
                if c.dx.abs() == 1 {
                    s_hook.position().z = hp.z + sib_dir * sib as f64 * sib_step;
                } else {
                    s_hook.position().x = hp.x + sib_dir * sib as f64 * sib_step;
                }
                g.world.add(&s_hook);
            }
        }
        // 25% chance: a wool scarf left on it, thrown over its arm (the
        // old cloth's palette pick, its length and its sway: the scarf's
        // colour, size and hang)
        if rnd() < 0.25 {
            let cloth_colors = [0x282018, 0x1a2028, 0x2a1010, 0x3a2820];
            let old = cloth_colors[ri(cloth_colors.len())];
            let length = 0.22 + rnd() * 0.10;
            let sway = (rnd() - 0.5) * 0.15;
            let wool = match old {
                0x282018 => 0x564838, // brown
                0x1a2028 => 0x3c4656, // navy
                0x2a1010 => 0x5a302c, // oxblood
                _ => 0x76604a,        // camel
            };
            // where the old cloth hung: 4 cm out from the hook, 15 cm below it
            let hp = hook.pos();
            let scarf = group();
            scarf.rotation().y = hook.rot().y;
            scarf.position().set(hp.x - c.dx as f64 * 0.04, hook_height - 0.15, hp.z - c.dz as f64 * 0.04);
            if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__scarfHung") {
                let bb = geo.bounding_box();
                let k = (length / 0.27).clamp(0.88, 1.12);
                // turned and sized about where it hangs on the arm (the
                // hook's cradle, on its block, 4 mm proud of the wall)
                let hang = group();
                hang.position().set(0, 0.15 - 0.047, -0.04 - 0.004 + HOOK_BLOCK_D + 0.058);
                hang.rotation().z = sway * 0.5;
                hang.scale().set(k, k, k);
                // (its fringe's tassels are single strips: both sides drawn)
                let m = mesh(geo, lambert!({ map: &scarf_tex(old, [bb.min.y, bb.max.y]), color: wool, side: DoubleSide }));
                m.position().set(0, 0.047, -0.058);
                hang.add(&m);
                scarf.add(&hang);
            }
            g.world.add(&scarf);
        }
        i += 1;
    }
}

/// The oak block's thickness (batch `r13b`, `Real__hookBlock`)
const HOOK_BLOCK_D: f64 = 0.019;

/// A cast iron double hook (`r06`) screwed to its oak block, the block's
/// back on the wall 4 mm behind the group's origin (where the old plate's
/// group stood off the wall)
fn hook_on_block(iron: &Mat, oak: &Mat, steel: &Mat) -> Obj {
    let hook = group();
    let z0 = -0.004;
    let part = |name: &str, w: f64, h: f64, d: f64| game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(w, h, d));
    let block = mesh(part("Real__hookBlock", 0.07, 0.13, HOOK_BLOCK_D), oak);
    block.position().z = z0;
    hook.add(&block);
    for sy in [-0.05, 0.05] {
        let sc = mesh(part("Real__hookScrew", 0.01, 0.01, 0.002), steel);
        sc.position().set(0, sy, z0 + HOOK_BLOCK_D - 0.0004);
        hook.add(&sc);
    }
    let iron_hook = mesh(part("Real__coatHook", 0.026, 0.12, 0.1), iron);
    iron_hook.position().z = z0 + HOOK_BLOCK_D;
    hook.add(&iron_hook);
    hook
}

/// Old oak, oiled and gone dark: its grain up the block, its rays
fn hook_block_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex(64, 128, |ctx, w, h| {
                ctx.set_fill_style("#6b4f34");
                ctx.fill_rect(0, 0, w, h);
                for _ in 0..26 {
                    let x = rnd() * w;
                    ctx.set_stroke_style(format!("rgba(42,28,16,{})", 0.10 + rnd() * 0.2));
                    ctx.set_line_width(0.5 + rnd() * 1.2);
                    ctx.begin_path();
                    ctx.move_to(x, 0);
                    let mut xx = x;
                    let mut y = 0.0;
                    while y < h {
                        y += 8.0;
                        xx += (rnd() - 0.5) * 0.8;
                        ctx.line_to(xx, y);
                    }
                    ctx.stroke();
                }
                for _ in 0..30 {
                    ctx.set_fill_style(format!("rgba(168,128,88,{})", 0.08 + rnd() * 0.12));
                    ctx.fill_rect(rnd() * w, rnd() * h, 1.0, 2.0 + rnd() * 5.0);
                }
                // hands' grime round the middle, where coats were taken off
                let gr = ctx.create_radial_gradient(w / 2.0, h * 0.45, 0, w / 2.0, h * 0.45, w * 0.8);
                gr.add_color_stop(0, "rgba(22,16,10,0.25)");
                gr.add_color_stop(1, "rgba(22,16,10,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, 0, w, h);
            })
        })
        .clone()
    })
}

/// A knitted scarf seen from the front over its height `y` (its frame's
/// bottom and top, m): its ribs down it, two stripes near its end, the
/// wool pilled and fuzzed, a moth hole or two, dust on what faces up; grey
/// to be laid over its colour. The old cloth's colour picks its stripes
fn scarf_tex(old: u32, y: [f64; 2]) -> Tex {
    canvas_tex(128, 256, move |ctx, w, h| {
        ctx.set_fill_style("#c8c8c8");
        ctx.fill_rect(0, 0, w, h);
        // the ribs: knit and purl, 4 mm, down its length
        let px_m = w / 0.153;
        let mut x = 0.0;
        while x < w {
            ctx.set_fill_style("rgba(0,0,0,0.16)");
            ctx.fill_rect(x, 0, (0.0016 * px_m).max(1.0), h);
            x += 0.004 * px_m;
        }
        // the stripes near its end (the front end hangs to the bottom)
        let at = |m: f64| (1.0 - (m - y[0]) / (y[1] - y[0])) * h;
        let stripe = if old == 0x2a1010 || old == 0x3a2820 { "rgba(235,228,210,0.55)" } else { "rgba(20,18,16,0.45)" };
        ctx.set_fill_style(stripe);
        for (a, b) in [(0.030, 0.052), (0.066, 0.078)] {
            let (y0, y1) = (at(y[0] + b), at(y[0] + a));
            ctx.fill_rect(0, y0, w, y1 - y0);
        }
        // pills and fuzz
        for _ in 0..700 {
            let v = if rnd() < 0.5 { "255,255,255" } else { "0,0,0" };
            ctx.set_fill_style(format!("rgba({v},{})", 0.05 + rnd() * 0.12));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 1.5, 1.0 + rnd() * 1.5);
        }
        // a moth hole or two
        for _ in 0..2 {
            ctx.set_fill_style("rgba(10,8,6,0.8)");
            ctx.begin_path();
            ctx.ellipse(rnd() * w, h * (0.3 + 0.6 * rnd()), 1.5 + rnd() * 1.5, 1.0 + rnd(), rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        // dust along its top, where it bends over the hook
        let gr = ctx.create_linear_gradient(0, 0, 0, h * 0.3);
        gr.add_color_stop(0, "rgba(235,228,214,0.30)");
        gr.add_color_stop(1, "rgba(235,228,214,0)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, 0, w, h * 0.3);
    })
}
