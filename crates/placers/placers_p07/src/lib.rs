//! Prop placers: country-kitchen props (mason jars, brooms, herb bundles, rolling
//! pins, boards, bowls, whisks, aprons, teapots, tins, teacups, ladles, recipes,
//! shakers, bread, wine, cheese, pie, cake stands, muffins, kitchen clocks),
//! wall art (landscapes, still lifes), sewing baskets, eyeliners and teething
//! rings.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod t01;
use t01::{place_abstract_landscapes, place_still_lifes};
mod t06;
use t06::place_herb_bundles;
mod t15;
use t15::place_teething_rings;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_mason_jars", place_mason_jars));
    r.push(("place_wall_brooms", place_wall_brooms));
    r.push(("place_herb_bundles", place_herb_bundles));
    r.push(("place_rolling_pins", place_rolling_pins));
    r.push(("place_cutting_boards", place_cutting_boards));
    r.push(("place_mixing_bowls", place_mixing_bowls));
    r.push(("place_whisks", place_whisks));
    r.push(("place_wall_aprons", place_wall_aprons));
    r.push(("place_teapots", place_teapots));
    r.push(("place_cookie_tins", place_cookie_tins));
    r.push(("place_teacups", place_teacups));
    r.push(("place_cast_iron_ladles", place_cast_iron_ladles));
    r.push(("place_spoon_rests", place_spoon_rests));
    r.push(("place_framed_recipes", place_framed_recipes));
    r.push(("place_shaker_pair", place_shaker_pair));
    r.push(("place_abstract_landscapes", place_abstract_landscapes));
    r.push(("place_bread_loaves", place_bread_loaves));
    r.push(("place_still_lifes", place_still_lifes));
    r.push(("place_wine_bottles", place_wine_bottles));
    r.push(("place_cheese_wedges", place_cheese_wedges));
    r.push(("place_pencil_eyeliners", place_pencil_eyeliners));
    r.push(("place_teething_rings", place_teething_rings));
    r.push(("place_kitchen_clocks", place_kitchen_clocks));
    r.push(("place_rolling_pin_stands", place_rolling_pin_stands));
    r.push(("place_pie_slices", place_pie_slices));
    r.push(("place_cake_stands", place_cake_stands));
    r.push(("place_cornbread_muffins", place_cornbread_muffins));
    r.push(("place_sewing_baskets", place_sewing_baskets));
}

// ------------------------------------------------------------------ helpers

/// The `open_tiles` scan every floor placer repeats: open interior tiles, row-major.
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                v.push((x, z));
            }
        }
    }
    v
}

/// The `wall_tiles` scan: open interior tiles with the first adjacent wall
/// (`tiles[..] == 1`) in `[(1,0), (-1,0), (0,1), (0,-1)]` order.
fn wall_tiles(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        v.push((x, z, dx, dz));
                        break;
                    }
                }
            }
        }
    }
    v
}

/// A uniformly random element (one `rnd()` draw).
fn pick<T: Copy>(a: &[T]) -> T {
    a[(rnd() * a.len() as f64).floor() as usize]
}

/// `atan2(-dx, -dz)` (keeps -0.0 for a zero component, on purpose)
fn face_wall(dx: i32, dz: i32) -> f64 {
    (-(dx as f64)).atan2(-(dz as f64))
}

// Mason jar: 7% chance per level. Glass body + metal
// screw lid + optional interior contents.
fn place_mason_jars(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let glass_mat = lambert!({ color: 0xd8e0d8, transparent: true, opacity: 0.55 });
    let lid_mat = lambert!({ color: 0xc0c0c0 });
    let content_colors: [u32; 5] = [0xc02020, 0xf0e020, 0x60a040, 0xa06030, 0xe0a020];
    let jar = group();
    // Glass body (short cylinder w/ slight shoulder taper).
    let body = mesh(cyl_geo!(0.036, 0.036, 0.10, 12), &glass_mat);
    body.position().y = 0.050;
    jar.add(&body);
    // Shoulder taper (small cone top).
    let shoulder = mesh(cyl_geo!(0.028, 0.036, 0.010, 12), &glass_mat);
    shoulder.position().y = 0.105;
    jar.add(&shoulder);
    // Neck.
    let neck = mesh(cyl_geo!(0.028, 0.028, 0.012, 10), &glass_mat);
    neck.position().y = 0.116;
    jar.add(&neck);
    // Metal lid (band + top disc).
    let band = mesh(pg("MasonJars__band", cyl_geo!(0.030, 0.030, 0.010, 12)), &lid_mat);
    band.position().y = 0.127;
    jar.add(&band);
    let lid_top = mesh(pg("MasonJars__lid", cyl_geo!(0.028, 0.028, 0.003, 12)), &lid_mat);
    lid_top.position().y = 0.134;
    jar.add(&lid_top);
    // 70% chance interior contents (colored fill).
    if rnd() < 0.7 {
        let c_mat = lambert!({ color: pick(&content_colors) });
        let fill_h = 0.05 + rnd() * 0.04;
        let content = mesh(pg("MasonJars__fill", cyl_geo!(0.032, 0.032, fill_h, 10)), &c_mat);
        content.position().y = fill_h / 2.0 + 0.002;
        jar.add(&content);
    }
    jar.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    jar.rotation().y = rnd() * PI * 2.0;
    g.world.add(&jar);
}

// Wall broom (hanging): 4% chance per level (Blender batch `r13b`): a
// hand-made round hearth broom, 0.71 m, hung on a nail by a jute loop through
// the hole in its handle's top, its head's back against the wall: broomcorn
// bound round a hardwood stick with twine, its ends splayed, worn short on
// one side and bent over from standing on it, straws sticking out; the
// handle dark where hands held it. (No wider along the wall than the old
// one and spanning the same heights, so it meets the wall's face where the
// old one did.)
fn place_wall_brooms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    // which broom this is comes from where it hangs (no draws)
    let vary = kitchen_hash(x, z, 0x6b70);
    let wood = [0x8a6a48, 0x7a5c3c, 0x9a7a54, 0x6e5238][(vary % 4) as usize];
    let twine = [0x8a4436, 0x4e5e3e, 0x9a8a68][((vary >> 4) % 3) as usize];
    let handle_mat = lambert!({ map: &broom_handle_tex(wood, vary) });
    let corn_mat = lambert!({ map: &broom_corn_tex(vary) });
    let steel = phong!({ color: 0x4a4641, specular: 0x262626, shininess: 26 });
    let twine_mat = lambert!({ color: twine });
    let cord_mat = lambert!({ color: 0x7a6a4c });
    let broom = group();
    // the nail, so the broom spans the old one's 1.14-1.85 m; the parts as
    // modelled about where it enters the wall, 10 mm behind the group's origin
    let nail_y = 1.8415;
    for (name, mat) in [
        ("Real__broomHandle", &handle_mat),
        ("Real__broomCorn", &corn_mat),
        ("Real__broomNail", &steel),
        ("Real__broomTwine", &twine_mat),
        ("Real__broomCord", &cord_mat),
    ] {
        if let Some(geo) = game::assets::props::prop_part_as_modelled(name) {
            let m = mesh(geo, mat);
            m.position().set(0, nail_y, -0.010);
            broom.add(&m);
        }
    }
    broom.rotation().y = face_wall(dx, dz);
    broom.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.010), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.010));
    g.world.add(&broom);
}

/// A hash of a tile and a salt: what a thing placed there is, with no draw
fn kitchen_hash(x: i32, z: i32, salt: u32) -> u32 {
    let mut h = salt ^ (x as u32).wrapping_mul(0x27d4_eb2d) ^ (z as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

/// The broom's handle round its length (u round it, v up it): a hardwood
/// stick oiled long ago, its grain and a knot, gone dark at the top where it
/// was held, grimed toward the broomcorn
fn broom_handle_tex(wood: u32, vary: u32) -> Tex {
    canvas_tex(32, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            ctx.set_fill_style(&format!("#{wood:06x}"));
            ctx.fill_rect(0, 0, w, h);
            // the grain, up the stick
            for _ in 0..24 {
                let x = rnd() * w;
                ctx.set_stroke_style(&format!("rgba(40,26,14,{})", 0.10 + rnd() * 0.2));
                ctx.set_line_width(0.5 + rnd());
                ctx.begin_path();
                ctx.move_to(x, 0);
                ctx.line_to(x + (rnd() - 0.5) * 3.0, h);
                ctx.stroke();
            }
            // a knot or two in it
            for _ in 0..2 {
                let (x, y) = (rnd() * w, rnd() * h);
                ctx.set_fill_style("rgba(50,32,18,0.45)");
                ctx.begin_path();
                ctx.ellipse(x, y, 1.5, 3.0, 0, 0, 7);
                ctx.fill();
            }
            // the top, where hands held it: dark and greasy
            let gr = ctx.create_linear_gradient(0, 0, 0, h * 0.45);
            gr.add_color_stop(0, "rgba(30,20,12,0.40)");
            gr.add_color_stop(1, "rgba(30,20,12,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, 0, w, h * 0.45);
            // grime toward its foot
            let gr = ctx.create_linear_gradient(0, h * 0.7, 0, h);
            gr.add_color_stop(0, "rgba(30,24,18,0)");
            gr.add_color_stop(1, "rgba(30,24,18,0.35)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, h * 0.7, w, h * 0.3);
        });
    })
}

/// The broomcorn seen from the front (u across, v up the head): straws side
/// by side in their own shades of straw, green-gold to brown, the sweep's
/// ends gone dark with what it swept, dust in it
fn broom_corn_tex(vary: u32) -> Tex {
    canvas_tex(64, 192, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            ctx.set_fill_style("#a48a5a");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..300 {
                let x = rnd() * w;
                let (y0, y1) = (rnd() * h * 0.5, h * (0.55 + 0.45 * rnd()));
                let k = rnd();
                let (r, gg, b) = if k < 0.55 {
                    (196.0, 170.0, 112.0)
                } else if k < 0.75 {
                    (168.0, 158.0, 98.0)
                } else if k < 0.92 {
                    (150.0, 112.0, 70.0)
                } else {
                    (112.0, 84.0, 56.0)
                };
                let s = 0.8 + 0.3 * rnd();
                ctx.set_stroke_style(&format!("rgba({},{},{},{})", (r * s) as i32, (gg * s) as i32, (b * s) as i32, 0.5 + 0.5 * rnd()));
                ctx.set_line_width(0.6 + rnd() * 1.1);
                ctx.begin_path();
                ctx.move_to(x, y0);
                ctx.line_to(x + (rnd() - 0.5) * 3.0, y1);
                ctx.stroke();
            }
            // the gaps between straws
            for _ in 0..90 {
                ctx.set_stroke_style(&format!("rgba(40,30,18,{})", 0.15 + rnd() * 0.25));
                ctx.set_line_width(0.5);
                let x = rnd() * w;
                ctx.begin_path();
                ctx.move_to(x, rnd() * h);
                ctx.line_to(x + (rnd() - 0.5) * 2.0, rnd() * h);
                ctx.stroke();
            }
            // the sweep's ends: dirt worked into them
            let gr = ctx.create_linear_gradient(0, h * 0.62, 0, h);
            gr.add_color_stop(0, "rgba(58,48,36,0)");
            gr.add_color_stop(0.7, "rgba(58,48,36,0.45)");
            gr.add_color_stop(1, "rgba(48,40,30,0.7)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, h * 0.62, w, h * 0.38);
            // dust caught in it
            for _ in 0..60 {
                ctx.set_fill_style(&format!("rgba(170,160,140,{})", 0.10 + rnd() * 0.15));
                ctx.fill_rect(rnd() * w, rnd() * h, 2.0 + rnd() * 6.0, 1.0 + rnd() * 3.0);
            }
        });
    })
}

// Wooden rolling pin: 6% chance per level. Central roll
// + 2 handle knobs.
fn place_rolling_pins(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_colors: [u32; 4] = [0xc0a070, 0xa07050, 0xd0b080, 0x8b5a2b];
    let wood_mat = lambert!({ color: pick(&wood_colors) });
    let pin = group();
    // Main roller cylinder (28 cm).
    let roller = mesh(pg("RollingPins__roller", cyl_geo!(0.024, 0.024, 0.28, 12)), &wood_mat);
    roller.rotation().z = PI / 2.0;
    roller.position().y = 0.024;
    pin.add(&roller);
    // 2 handle stems (thinner cylinders extending each end).
    for side in [-1.0, 1.0] {
        let stem = mesh(pg("RollingPins__stem", cyl_geo!(0.010, 0.010, 0.050, 8)), &wood_mat);
        stem.rotation().z = PI / 2.0;
        stem.position().set(side * 0.165, 0.024, 0);
        pin.add(&stem);
        // Handle knob at end (small sphere).
        let knob = mesh(pg("RollingPins__knob", sphere_geo!(0.014, 6, 5)), &wood_mat);
        knob.position().set(side * 0.194, 0.024, 0);
        pin.add(&knob);
    }
    pin.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pin.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pin);
}

// Cutting board: 6% chance per level. Wooden board
// w/ hole handle.
fn place_cutting_boards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_colors: [u32; 4] = [0xc0a070, 0x8b5a2b, 0xa08050, 0xd0b080];
    let board_mat = lambert!({ color: pick(&wood_colors) });
    let board = group();
    let plank = mesh(pg("CuttingBoards__plank", box_geo!(0.30, 0.018, 0.20)), &board_mat);
    plank.position().y = 0.009;
    board.add(&plank);
    let hole = mesh(cyl_geo!(0.012, 0.012, 0.022, 8), lambert!({ color: 0x201810 }));
    hole.scale().y = 0.8; // 0..0.0176: just under the board top (0.018), not proud of it
    hole.position().set(-0.130, 0.0088, 0);
    board.add(&hole);
    for i in 0..3 {
        let score = mesh(box_geo!(0.10 + rnd() * 0.06, 0.001, 0.001), lambert!({ color: 0x503020 }));
        score.position().set(0.02 + i as f64 * 0.02, 0.019, -0.06 + i as f64 * 0.03);
        score.rotation().y = (rnd() - 0.5) * 0.2;
        board.add(&score);
    }
    board.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    board.rotation().y = rnd() * PI * 2.0;
    g.world.add(&board);
}

// Ceramic mixing bowl: 6% chance per level.
fn place_mixing_bowls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bowl_colors: [u32; 6] = [0xf0e8d8, 0xd8b090, 0x8080a0, 0x408090, 0xa0d0e0, 0xe0e0e0];
    let bowl_mat = lambert!({ color: pick(&bowl_colors) });
    let bowl = group();
    let body = mesh(pg("MixingBowls__body", cyl_geo!(0.075, 0.048, 0.055, 14)), &bowl_mat);
    body.position().y = 0.028;
    bowl.add(&body);
    let rim = mesh(pg("MixingBowls__rim", torus_geo!(0.075, 0.005, 4, 16)), &bowl_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.056;
    bowl.add(&rim);
    if rnd() < 0.5 {
        let dough_mat = lambert!({ color: 0xf0e0b8 });
        let dough = mesh(pg("MixingBowls__dough", sphere_geo!(0.055, 10, 6)), &dough_mat);
        dough.scale().y = 0.4;
        dough.position().y = 0.046;
        bowl.add(&dough);
    }
    bowl.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    bowl.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bowl);
}

// Wire whisk: 7% chance per level. Handle + bulb of
// curved wire loops.
fn place_whisks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wire_mat = lambert!({ color: 0xc0c0c0 });
    let handle_colors: [u32; 4] = [0x606060, 0x8b5a2b, 0x202020, 0xc02020];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    let whisk = group();
    // Handle laid flat.
    let handle = mesh(pg("Whisks__handle", cyl_geo!(0.010, 0.010, 0.10, 8)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(-0.030, 0.010, 0);
    whisk.add(&handle);
    // Handle end cap.
    let cap = mesh(pg("Whisks__cap", sphere_geo!(0.011, 6, 5)), &handle_mat);
    cap.position().set(-0.084, 0.010, 0);
    whisk.add(&cap);
    // Metal ferrule between handle + wires.
    let ferrule = mesh(pg("Whisks__ferrule", cyl_geo!(0.011, 0.011, 0.010, 8)), &wire_mat);
    ferrule.rotation().z = PI / 2.0;
    ferrule.position().set(0.024, 0.010, 0);
    whisk.add(&ferrule);
    // 6 curved wire loops forming teardrop bulb.
    for i in 0..6 {
        let a = (i as f64 / 6.0) * PI * 2.0;
        let wire = mesh(pg("Whisks__wire", torus_geo!(0.020, 0.0012, 3, 10, PI)), &wire_mat);
        // Loops fanned round the handle axis (x): their ends meet at the poles,
        // the near pole (x 0.027) inside the ferrule.
        wire.rotation().x = a;
        wire.position().set(0.047, 0.010, 0);
        whisk.add(&wire);
    }
    whisk.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    whisk.rotation().y = rnd() * PI * 2.0;
    // The wire ball (r 0.021) is fatter than the handle: tip the whisk so the
    // ball and the end cap both rest on the floor.
    whisk.rotation().z = 0.078;
    whisk.position().y = 0.0076;
    g.world.add(&whisk);
}

// Hanging wall apron: 5% chance per level (Blender batch `r13b`): a
// cotton bib apron, 72 x 86 cm, hung by its neck strap on a cast iron hook
// (`r06`'s): the bib's top fallen into a cowl between the strap's ends, the
// waist's corners dropped, the skirt against the wall in folds, its patch
// pocket, its ties hanging; faded, stained where hands were wiped on it.
fn place_wall_aprons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let apron_colors: [u32; 6] = [0xc02020, 0x2060a0, 0x408040, 0xc0a040, 0x606060, 0xd050a0];
    // the old pick, as a real cloth: its dye faded (the blue a butcher's
    // stripe, painted in its map)
    let (cloth, stripe) = match pick(&apron_colors) {
        0xc02020 => (0x8c3d34, false),
        0x2060a0 => (0xffffff, true),
        0x408040 => (0x66705a, false),
        0xc0a040 => (0xa68e5e, false),
        0x606060 => (0x77746c, false),
        _ => (0xa67876, false),
    };
    let vary = kitchen_hash(x, z, 0xa970);
    let apron = group();
    // the hook at 1.85 m, its plate on the wall (8 mm behind the group's
    // origin); the apron as modelled hanging from it
    let hook_y = 1.85;
    let iron = phong!({ color: 0x1c1b1a, specular: 0x1a1a1a, shininess: 22 });
    let hook = mesh(game::assets::props::prop_part_as_modelled("Real__coatHook").unwrap_or_else(|| box_geo!(0.026, 0.12, 0.1)), &iron);
    hook.position().set(0, hook_y, -0.008);
    apron.add(&hook);
    if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__apronCloth") {
        let bb = geo.bounding_box();
        let mat = lambert!({ map: &apron_tex(stripe, vary, [bb.min.x, bb.max.x, bb.min.y, bb.max.y]), color: cloth, side: DoubleSide });
        let m = mesh(geo, &mat);
        m.position().set(0, hook_y, -0.008);
        apron.add(&m);
    }
    apron.rotation().y = face_wall(dx, dz);
    apron.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.008), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.008));
    g.world.add(&apron);
}

/// The apron's cloth seen from the front over its bounds `b` (x0, x1, y0,
/// y1 in its frame, the hook at the origin): cotton twill, white to be laid
/// over its colour (or a butcher's navy stripe), faded, grease spotted down
/// the bib, hands wiped at the hips and the pocket's mouth, the hem grimed
fn apron_tex(stripe: bool, vary: u32, b: [f64; 4]) -> Tex {
    canvas_tex(256, 512, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let (x0, x1, y0, y1) = (b[0], b[1], b[2], b[3]);
            // where a point of the apron (its frame) is on the map
            let at = |x: f64, y: f64| ((x - x0) / (x1 - x0) * w, (1.0 - (y - y0) / (y1 - y0)) * h);
            let px_m = w / (x1 - x0);
            ctx.set_fill_style(if stripe { "#d6d1c4" } else { "#f2f0ea" });
            ctx.fill_rect(0, 0, w, h);
            if stripe {
                // the butcher's stripe: 3 mm navy every 9 mm, down it
                ctx.set_fill_style("#2e3a52");
                let mut x = 0.0;
                while x < w {
                    ctx.fill_rect(x, 0, (0.003 * px_m).max(1.0), h);
                    x += 0.009 * px_m;
                }
            }
            // the twill's diagonal and the weave's unevenness
            ctx.set_line_width(1);
            let mut d = -h;
            while d < w {
                ctx.set_stroke_style(&format!("rgba(0,0,0,{})", 0.03 + rnd() * 0.04));
                ctx.begin_path();
                ctx.move_to(d, 0);
                ctx.line_to(d + h * 0.5, h);
                ctx.stroke();
                d += 2.0;
            }
            // grease and old splashes down the bib and the skirt: each a
            // cluster of soft blots, uneven, some run down a little
            for _ in 0..6 {
                let (cx, cy) = ((rnd() - 0.5) * 0.30, -0.42 - rnd() * 0.50);
                let size = 0.006 + rnd() * 0.02;
                let a = 0.10 + rnd() * 0.12;
                for _ in 0..(3 + (rnd() * 4.0) as i32) {
                    let (x, y) = at(cx + (rnd() - 0.5) * size * 2.5, cy + (rnd() - 0.5) * size * 2.0);
                    let r = size * (0.4 + rnd()) * px_m;
                    ctx.save();
                    ctx.translate(x, y);
                    ctx.rotate(rnd() * 3.0);
                    ctx.scale(1, 0.5 + 0.6 * rnd());
                    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
                    gr.add_color_stop(0, &format!("rgba(74,56,34,{a})"));
                    gr.add_color_stop(0.7, &format!("rgba(74,56,34,{})", a * 0.7));
                    gr.add_color_stop(1, "rgba(74,56,34,0)");
                    ctx.set_fill_style(gr);
                    ctx.begin_path();
                    ctx.arc(0, 0, r, 0, 7);
                    ctx.fill();
                    ctx.restore();
                }
                if rnd() < 0.5 {
                    // a run down from it
                    let (x, y) = at(cx, cy);
                    let l = (0.02 + rnd() * 0.05) * px_m;
                    let gr = ctx.create_linear_gradient(0, y, 0, y + l);
                    gr.add_color_stop(0, &format!("rgba(74,56,34,{})", a * 0.8));
                    gr.add_color_stop(1, "rgba(74,56,34,0)");
                    ctx.set_fill_style(gr);
                    ctx.fill_rect(x - 1.0, y, 2.0 + rnd() * 1.5, l);
                }
            }
            // hands wiped at the hips and at the pocket's mouth
            for (hx, hy, rx, ry, a) in [(-0.24, -0.70, 0.07, 0.12, 0.30), (0.24, -0.70, 0.07, 0.12, 0.30), (0.0, -0.80, 0.18, 0.03, 0.22)] {
                let (x, y) = at(hx + (rnd() - 0.5) * 0.04, hy + (rnd() - 0.5) * 0.04);
                ctx.save();
                ctx.translate(x, y);
                ctx.scale(1, ry / rx);
                let r = rx * px_m;
                let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
                gr.add_color_stop(0, &format!("rgba(56,44,32,{a})"));
                gr.add_color_stop(1, "rgba(56,44,32,0)");
                ctx.set_fill_style(gr);
                ctx.begin_path();
                ctx.arc(0, 0, r, 0, 7);
                ctx.fill();
                ctx.restore();
            }
            // the hem and the ties' ends grimed, the top faded paler
            let (_, yh) = at(0.0, -1.10);
            let gr = ctx.create_linear_gradient(0, yh, 0, h);
            gr.add_color_stop(0, "rgba(50,42,34,0)");
            gr.add_color_stop(1, "rgba(50,42,34,0.35)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, yh, w, h - yh);
            let (_, yt) = at(0.0, -0.55);
            let gr = ctx.create_linear_gradient(0, 0, 0, yt);
            gr.add_color_stop(0, "rgba(236,232,222,0.22)");
            gr.add_color_stop(1, "rgba(236,232,222,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, 0, w, yt);
            for _ in 0..400 {
                let v = if rnd() < 0.5 { "255,255,255" } else { "0,0,0" };
                ctx.set_fill_style(&format!("rgba({v},{})", 0.03 + rnd() * 0.06));
                ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
            }
        });
    })
}

// Teapot on trivet: 4% chance per level.
fn place_teapots(g: &mut Game, _dist: &[i32]) {
    g.pp.teapot_positions.clear(); // reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let pot_colors: [u32; 6] = [0xf0f0f0, 0x408080, 0xc02020, 0x203050, 0xd0a040, 0x606060];
    let pot_mat = lambert!({ color: pick(&pot_colors) });
    let trivet_mat = lambert!({ color: 0x505050 });
    let teapot = group();
    let trivet = mesh(pg("Teapots__trivet", cyl_geo!(0.075, 0.075, 0.006, 10)), &trivet_mat);
    trivet.position().y = 0.003;
    teapot.add(&trivet);
    for i in 0..3 {
        let a = (i as f64 / 3.0) * PI * 2.0;
        let foot = mesh(pg("Teapots__foot", cyl_geo!(0.006, 0.006, 0.008, 4)), &trivet_mat);
        // Under the trivet (top 0.5 mm into its underside); the group is raised onto the feet.
        foot.position().set(a.cos() * 0.060, -0.0035, a.sin() * 0.060);
        teapot.add(&foot);
    }
    let body = mesh(pg("Teapots__body", sphere_geo!(0.060, 12, 10)), &pot_mat);
    body.scale().y = 0.75;
    body.position().y = 0.052;
    teapot.add(&body);
    let spout1 = mesh(pg("Teapots__spout", cyl_geo!(0.012, 0.014, 0.030, 6)), &pot_mat);
    // Wide end in the body, narrow end rising outward (-x) at 30 degrees.
    spout1.rotation().z = PI / 3.0;
    spout1.position().set(-0.058, 0.058, 0);
    teapot.add(&spout1);
    let spout2 = mesh(pg("Teapots__spoutTip", cyl_geo!(0.009, 0.012, 0.024, 6)), &pot_mat);
    // Tip continues from spout1's narrow end (-0.071, 0.0655), rising at 45 degrees.
    spout2.rotation().z = PI / 4.0;
    spout2.position().set(-0.0795, 0.074, 0);
    teapot.add(&spout2);
    let handle = mesh(pg("Teapots__handle", torus_geo!(0.030, 0.006, 4, 10, PI)), &pot_mat);
    // C looping out from the +x side, both ends on the belly.
    handle.rotation().z = -PI / 2.0;
    handle.position().set(0.044, 0.052, 0);
    teapot.add(&handle);
    let lid = mesh(pg("Teapots__lid", cyl_geo!(0.022, 0.026, 0.010, 10)), &pot_mat);
    lid.position().y = 0.101; // seated on the body top (0.097)
    teapot.add(&lid);
    let knob = mesh(pg("Teapots__knob", sphere_geo!(0.008, 6, 5)), &pot_mat);
    knob.position().y = 0.111;
    teapot.add(&knob);
    teapot.position().set(
        g.wx(x) + (rnd() - 0.5) * 2.0,
        0.0075, // standing on the trivet feet
        g.wz(z) + (rnd() - 0.5) * 2.0,
    );
    teapot.rotation().y = rnd() * PI * 2.0;
    g.world.add(&teapot);
    // Subtle lid lift-and-settle: rare, brief Y bob when
    // sin(t*0.12 + phase) > 0.94 (~6% of time). Simulates steam
    // pressure lifting lid slightly.
    let phase = rnd() * PI * 2.0;
    let (lid2, knob2) = (lid.clone(), knob.clone());
    g.animate(move |_g, _dt, t| {
        let trig = (t * 0.12 + phase).sin();
        if trig > 0.94 {
            let local = (trig - 0.94) / 0.06; // 0..1
            let lift = (local * PI).sin() * 0.003;
            lid2.position().y = 0.101 + lift;
            knob2.position().y = 0.111 + lift;
        } else {
            lid2.position().y = 0.101;
            knob2.position().y = 0.111;
        }
    });
    let p = teapot.pos();
    g.pp.teapot_positions.push(v3(p.x, 0, p.z)); // track for whistle
}

// Cookie tin box: 6% chance per level.
game::lazy_static_tex!(
    cookie_tin_texture,
    canvas_tex(160, 160, |ctx, w, h| {
        ctx.set_fill_style("#e8d8a8");
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        ctx.set_stroke_style("#804020");
        ctx.set_line_width(3);
        ctx.begin_path();
        ctx.arc(cx, cy, w * 0.42, 0, PI * 2.0);
        ctx.stroke();
        ctx.set_fill_style("#c04040");
        for i in 0..8 {
            let a = (i as f64 / 8.0) * PI * 2.0;
            ctx.begin_path();
            ctx.arc(cx + a.cos() * w * 0.36, cy + a.sin() * w * 0.36, 6, 0, PI * 2.0);
            ctx.fill();
        }
        ctx.set_fill_style("#804020");
        ctx.begin_path();
        ctx.arc(cx, cy, 20, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style("#f0e0a0");
        ctx.begin_path();
        for i in 0..10 {
            let a = (i as f64 / 10.0) * PI * 2.0 - PI / 2.0;
            let r = if i % 2 == 0 { 14.0 } else { 6.0 };
            let px = cx + a.cos() * r;
            let py = cy + a.sin() * r;
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.close_path();
        ctx.fill();
    })
);

fn place_cookie_tins(g: &mut Game, _dist: &[i32]) {
    g.pp.cookie_tin_positions.clear(); // reset per level
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let tin_colors: [u32; 4] = [0xa04020, 0x408060, 0x2060a0, 0x604020];
    let tin_mat = lambert!({ color: pick(&tin_colors) });
    let lid_mat = lambert!({ map: cookie_tin_texture() });
    let tin = group();
    let body = mesh(pg("CookieTins__body", cyl_geo!(0.075, 0.075, 0.055, 16)), &tin_mat);
    body.position().y = 0.028;
    tin.add(&body);
    let lid = mesh(circle_geo!(0.075, 16), &lid_mat);
    lid.rotation().x = -PI / 2.0;
    lid.position().y = 0.056;
    tin.add(&lid);
    let rim = mesh(pg("CookieTins__rim", torus_geo!(0.075, 0.003, 4, 16)), &tin_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.056;
    tin.add(&rim);
    // Subtle lid Y-jitter — 35% chance. Pairs w/ the tin's rattle audio:
    // when the tin "rattles", visually the lid is also settling.
    if rnd() < 0.35 {
        let lid_base_y = 0.056;
        let jitter_phase = rnd() * PI * 2.0;
        let jitter_phase2 = rnd() * PI * 2.0;
        let lid2 = lid.clone();
        g.animate(move |_g, _dt, t| {
            // Multi-freq combined sine, very small amplitude (~0.6 mm).
            lid2.position().y = lid_base_y + (t * 2.7 + jitter_phase).sin() * 0.0006 + (t * 4.3 + jitter_phase2).sin() * 0.0004;
        });
    }
    tin.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    tin.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tin);
    let p = tin.pos();
    g.pp.cookie_tin_positions.push(v3(p.x, 0, p.z));
}

// Teacup + saucer: 7% chance per level.
fn place_teacups(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let cup_colors: [u32; 6] = [0xffffff, 0xf0e0d0, 0x8080a0, 0xd050a0, 0xc0e0a0, 0xa0d0e0];
    let cup_mat = lambert!({ color: pick(&cup_colors) });
    let cup = group();
    let saucer = mesh(pg("Teacups__saucer", cyl_geo!(0.055, 0.050, 0.006, 14)), &cup_mat);
    saucer.position().y = 0.003;
    cup.add(&saucer);
    let indent = mesh(cyl_geo!(0.028, 0.028, 0.002, 12), lambert!({ color: 0xf0e0d0 }));
    indent.position().y = 0.007;
    cup.add(&indent);
    let body = mesh(pg("Teacups__cup", cyl_geo!(0.028, 0.022, 0.032, 12)), &cup_mat);
    body.position().y = 0.022;
    cup.add(&body);
    let handle = mesh(pg("Teacups__handle", torus_geo!(0.012, 0.003, 4, 8, PI)), &cup_mat);
    // C looping out from the cup wall (+x), both ends on the wall.
    handle.rotation().z = -PI / 2.0;
    handle.position().set(0.0245, 0.022, 0);
    cup.add(&handle);
    if rnd() < 0.6 {
        let tea_mat = lambert!({ color: 0x40200a });
        let tea = mesh(pg("Teacups__tea", cyl_geo!(0.024, 0.024, 0.003, 10)), &tea_mat);
        tea.position().y = 0.036;
        cup.add(&tea);
    }
    cup.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cup.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cup);
    let p = cup.pos();
    g.pp.teacup_positions.push(v3(p.x, 0, p.z));
}

// Hanging cast iron ladle: 4% chance per level.
fn place_cast_iron_ladles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let iron_mat = lambert!({ color: 0x181818 });
    let ladle = group();
    let mount_y = 1.80;
    let handle = mesh(pg("CastIronLadles__handle", cyl_geo!(0.008, 0.008, 0.34, 6)), &iron_mat);
    handle.position().set(0, mount_y, 0.010);
    ladle.add(&handle);
    let ring = mesh(pg("CastIronLadles__ring", torus_geo!(0.010, 0.002, 3, 8)), &iron_mat);
    // Upright hanging ring (facing the room), its lower edge through the handle top (0.17).
    ring.position().set(0, mount_y + 0.176, 0.010);
    ladle.add(&ring);
    let bowl = mesh(pg("CastIronLadles__bowl", sphere_geo!(0.035, 10, 6, 0, PI * 2.0, PI / 2.0, PI / 2.0)), &iron_mat);
    // Rim (opening at the bowl centre height) meets the handle's lower end at its back.
    bowl.position().set(0, mount_y - 0.172, 0.037);
    ladle.add(&bowl);
    ladle.rotation().y = face_wall(dx, dz);
    ladle.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&ladle);
}

// Spoon rest w/ spoon: 7% chance per level.
fn place_spoon_rests(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let rest_colors: [u32; 5] = [0xffffff, 0xf0e0d0, 0x8080a0, 0x408080, 0xd050a0];
    let rest_mat = lambert!({ color: pick(&rest_colors) });
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let rest = group();
    let dish = mesh(pg("SpoonRests__dish", sphere_geo!(0.045, 12, 8, 0, PI * 2.0, 0, PI / 2.0)), &rest_mat);
    // Flipped into a shallow dish standing on the plate (0.004..0.0139), not half under the floor.
    dish.scale().set(1.6, 0.22, 1);
    dish.rotation().x = PI;
    dish.position().y = 0.0139;
    rest.add(&dish);
    let plate = mesh(pg("SpoonRests__plate", cyl_geo!(0.070, 0.070, 0.004, 14)), &rest_mat);
    plate.scale().set(1, 1, 0.65);
    plate.position().y = 0.002;
    rest.add(&plate);
    if rnd() < 0.7 {
        let handle = mesh(pg("SpoonRests__handle", cyl_geo!(0.006, 0.006, 0.13, 6)), &wood_mat);
        handle.rotation().z = PI / 2.0;
        handle.position().set(0.016, 0.0195, 0); // resting on the dish rim (top 0.0139)
        rest.add(&handle);
        let bowl = mesh(pg("SpoonRests__bowl", sphere_geo!(0.014, 8, 6)), &wood_mat);
        bowl.scale().set(1, 0.4, 0.75);
        bowl.position().set(-0.056, 0.0165, 0); // lying in the dish, inside its rim
        rest.add(&bowl);
    }
    rest.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    rest.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rest);
}

// Framed hand-drawn recipe: 4% chance per level.
game::lazy_static_tex!(
    recipe_texture,
    canvas_tex(112, 152, |ctx, w, h| {
        ctx.set_fill_style("#f8f0d8");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_stroke_style("#804020");
        ctx.set_line_width(1);
        ctx.stroke_rect(6, 6, w - 12.0, h - 12.0);
        ctx.set_fill_style("#40200a");
        ctx.fill_rect(14, 14, w - 28.0, 6);
        ctx.set_fill_style("#c02020");
        ctx.begin_path();
        ctx.arc(w * 0.5, 32, 4, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style("#40301a");
        for i in 0..8 {
            let y = 46.0 + i as f64 * 11.0;
            let start_x = 14.0 + if i % 2 == 0 { 0.0 } else { 6.0 };
            let w2 = 40.0 + rnd() * (w - 60.0 - start_x);
            ctx.fill_rect(start_x, y, w2, 1.6);
        }
        ctx.set_fill_style("#40200a");
        ctx.fill_rect(w * 0.5, h - 22.0, w * 0.35, 1.8);
    })
);

fn place_framed_recipes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let frame_mat = lambert!({ color: 0x604020 });
    let paper_mat = lambert!({ map: recipe_texture() });
    let rec = group();
    let mount_y = 1.65;
    let frame = mesh(pg("FramedRecipes__frame", box_geo!(0.20, 0.26, 0.014)), &frame_mat);
    frame.position().y = mount_y;
    rec.add(&frame);
    let paper = mesh(plane_geo!(0.16, 0.22), &paper_mat);
    paper.position().set(0, mount_y, 0.002); // in the frame's rebate (floor z 0.001), behind the sight edge
    rec.add(&paper);
    rec.rotation().z = (rnd() - 0.5) * 0.06;
    rec.rotation().y = face_wall(dx, dz);
    rec.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    g.world.add(&rec);
}

// Salt shaker + pepper mill pair: 6% chance per level.
fn place_shaker_pair(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let glass_mat = lambert!({ color: 0xffffff, transparent: true, opacity: 0.75 });
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let cap_mat = lambert!({ color: 0xc0c0c0 });
    let salt_mat = lambert!({ color: 0xf8f4e8 });
    let pepper_mat = lambert!({ color: 0x202018 });
    let pair = group();
    let salt_body = mesh(cyl_geo!(0.022, 0.022, 0.058, 10), &glass_mat);
    salt_body.position().set(-0.028, 0.029, 0);
    pair.add(&salt_body);
    let salt_fill = mesh(pg("ShakerPair__salt", cyl_geo!(0.020, 0.020, 0.030, 10)), &salt_mat);
    salt_fill.position().set(-0.028, 0.017, 0);
    pair.add(&salt_fill);
    let salt_cap = mesh(pg("ShakerPair__cap", cyl_geo!(0.024, 0.022, 0.014, 10)), &cap_mat);
    salt_cap.position().set(-0.028, 0.065, 0);
    pair.add(&salt_cap);
    for i in 0..5 {
        let a = (i as f64 / 5.0) * PI * 2.0;
        let hole = mesh(circle_geo!(0.002, 4), lambert!({ color: 0x101010 }));
        hole.rotation().x = -PI / 2.0;
        hole.position().set(-0.028 + a.cos() * 0.012, 0.073, a.sin() * 0.012);
        pair.add(&hole);
    }
    let mill_body = mesh(pg("ShakerPair__mill", cyl_geo!(0.020, 0.024, 0.100, 10)), &wood_mat);
    mill_body.position().set(0.028, 0.050, 0);
    pair.add(&mill_body);
    let pepper_fill = mesh(cyl_geo!(0.017, 0.017, 0.030, 8), &pepper_mat);
    pepper_fill.position().set(0.028, 0.020, 0);
    pair.add(&pepper_fill);
    let mill_knob = mesh(pg("ShakerPair__knob", cyl_geo!(0.008, 0.008, 0.014, 8)), &cap_mat);
    mill_knob.position().set(0.028, 0.107, 0);
    pair.add(&mill_knob);
    pair.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    pair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pair);
}

// A loaf left on its board (Blender batch `r16`): a small white bloomer
// on a beech bread board, its end cut off long ago; stale, shrunk and
// cracked, its crust gone dull, mould grown over it in blue-green and white
// colonies, thickest on the cut face; the board scored by knives, greyed,
// a hanging hole at its end, crumbs by the loaf. 5% chance per level.
fn place_bread_loaves(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let loaf = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    let yaw = rnd() * PI * 2.0;
    // how far the mould has got differs floor to floor: from where it lies
    let salt = ((px * 977.0).floor() as i64 ^ ((pz * 1181.0).floor() as i64).rotate_left(13)) as u32;
    let parts = [
        ("Real__breadBoard", lambert!({ map: &loaf_board_tex(salt) })),
        ("Real__breadCrust", lambert!({ map: &loaf_crust_tex(salt) })),
        ("Real__breadCrumb", lambert!({ map: &loaf_crumb_tex(salt) })),
        ("Real__breadCrumbs", lambert!({ color: 0x9a8058 })),
    ];
    for (name, mat) in &parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            loaf.add(&mesh(geo, mat));
        }
    }
    loaf.position().set(px, 0, pz);
    loaf.rotation().y = yaw;
    g.world.add(&loaf);
}

/// The loaf as modelled (r16.py): the crust part's bounds in x and z, the
/// slashes (their middles along x from the loaf's own middle, the angle of
/// the line across them), the board's size
const LOAF_CRUST_X: (f64, f64) = (-0.1415, 0.0330);
const LOAF_CRUST_Z: (f64, f64) = (-0.0550, 0.0550);
const LOAF_X0: f64 = -0.035;
const LOAF_SLASHES: [(f64, f64); 5] = [(-0.074, 0.55), (-0.040, 0.58), (-0.006, 0.54), (0.028, 0.57), (0.062, 0.55)];
const LOAF_BOARD: (f64, f64) = (0.320, 0.220);

fn loaf_hash(seed: u32, x: i32, y: i32) -> f64 {
    let mut h = seed ^ (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h = (h ^ (h >> 13)).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    h as f64 / 4_294_967_296.0
}
fn loaf_noise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (fx, fy) = (x - xf, y - yf);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xf as i32, yf as i32);
    let (a, b) = (loaf_hash(seed, xi, yi), loaf_hash(seed, xi + 1, yi));
    let (c, d) = (loaf_hash(seed, xi, yi + 1), loaf_hash(seed, xi + 1, yi + 1));
    a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}
fn loaf_fbm(seed: u32, x: f64, y: f64) -> f64 {
    0.55 * loaf_noise(seed, x, y) + 0.3 * loaf_noise(seed ^ 0x51, x * 2.3, y * 2.3) + 0.15 * loaf_noise(seed ^ 0xa2, x * 5.1, y * 5.1)
}
fn loaf_mix(c: &mut [f64; 3], to: [f64; 3], k: f64) {
    let k = k.clamp(0.0, 1.0);
    for i in 0..3 {
        c[i] += (to[i] - c[i]) * k;
    }
}

/// Mould at a point, from the colonies [(x, y, radius)] (in px): ragged
/// patches, a grey-green spore mass in the middle, white cottony fuzz
/// round it and over it in wisps, a yellowed stain in the bread round it;
/// returns (spores, fuzz, stain)
fn loaf_mould(cols: &[(f64, f64, f64)], x: f64, y: f64, seed: u32) -> (f64, f64, f64) {
    let (mut s, mut wr, mut st) = (0.0f64, 0.0f64, 0.0f64);
    let warp = loaf_fbm(seed, x / 7.0, y / 7.0) - 0.5;
    let fuzz = loaf_noise(seed ^ 0x33, x / 1.6, y / 1.6);
    for &(cx, cy, cr) in cols {
        let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() / cr + 0.9 * warp;
        s = s.max(((0.55 - d) * 3.0).clamp(0.0, 1.0) * (0.6 + 0.4 * fuzz));
        wr = wr.max(((1.05 - d) * 2.2).clamp(0.0, 1.0) * (0.35 + 0.65 * fuzz));
        st = st.max(((1.5 - d) * 1.6).clamp(0.0, 1.0));
    }
    (s, wr, st)
}

/// The crust (the box's texture coordinates over the crust's bounds: on
/// top, u along the loaf and v across it): a bake's browns, darker on the
/// ridges, paler down the opened slashes, gone dull and grey with age;
/// cracks; mould colonies, most toward the cut end and low on the sides
fn loaf_crust_tex(salt: u32) -> Tex {
    canvas_tex_opt(384, 192, false, move |ctx, w, h| {
        let seed = salt ^ 0xb0a1;
        let mut r = SplitMix64::new(salt, 0x10af);
        // mould colonies (px): a few big, many small, more at the cut end (u = 1)
        let cols: Vec<(f64, f64, f64)> = (0..10)
            .map(|k| {
                let u = 1.0 - r.next().powf(2.2) * 0.8;
                let rad = if k < 2 { r.range(14.0, 26.0) } else { r.range(3.0, 9.0) };
                (u * w, r.next() * h, rad)
            })
            .collect();
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let fv = (j as f64 + 0.5) / h;
            let zz = LOAF_CRUST_Z.1 - fv * (LOAF_CRUST_Z.1 - LOAF_CRUST_Z.0);
            for i in 0..n {
                let fu = (i as f64 + 0.5) / w;
                let xx = LOAF_CRUST_X.0 + fu * (LOAF_CRUST_X.1 - LOAF_CRUST_X.0);
                let f = loaf_fbm(seed, xx * 60.0, zz * 60.0);
                // the bake: golden brown, darker across the middle of the top
                let mid = 1.0 - (zz.abs() / 0.055).min(1.0);
                let mut c =
                    [150.0 - 30.0 * mid + 18.0 * (f - 0.5), 108.0 - 26.0 * mid + 14.0 * (f - 0.5), 64.0 - 18.0 * mid + 10.0 * (f - 0.5)];
                // the slashes opened pale down their middles (the grooves
                // themselves are modelled; their paint only near the crown,
                // where the top's mapping is the crust's own)
                let crown = (1.0 - (zz.abs() / 0.03).min(1.0)).powi(2);
                for &(xs, ang) in &LOAF_SLASHES {
                    let (dx, dz) = (xx - (xs + LOAF_X0), zz);
                    let along = dx * ang.cos() + dz * ang.sin();
                    let pale = (-(along / 0.004).powi(2)).exp() * crown;
                    loaf_mix(&mut c, [176.0, 150.0, 108.0], pale * 0.45);
                }
                // stale: dull, greyed
                loaf_mix(&mut c, [128.0, 118.0, 100.0], 0.25);
                // cracks in the dried crust
                let cr = (1.0 - ((loaf_noise(seed ^ 4, xx * 140.0, zz * 140.0) - 0.5).abs() * 30.0).min(1.0)) * (f > 0.5) as i32 as f64;
                loaf_mix(&mut c, [70.0, 52.0, 34.0], cr * 0.7);
                // the mould
                let (s, wr, st) = loaf_mould(&cols, i as f64, j as f64, seed ^ 7);
                loaf_mix(&mut c, [140.0, 128.0, 92.0], st * 0.3);
                loaf_mix(&mut c, [204.0, 204.0, 194.0], wr * 0.7);
                let sp = loaf_noise(seed ^ 9, i as f64 / 1.5, j as f64 / 1.5);
                loaf_mix(&mut c, [92.0 + 24.0 * sp, 106.0 + 18.0 * sp, 90.0 + 12.0 * sp], s * 0.9);
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // black pin mould here and there
        for _ in 0..30 {
            let (x, y) = (w * (1.0 - r.next().powf(1.4)), r.next() * h);
            ctx.set_fill_style("rgba(30,30,28,0.8)");
            ctx.begin_path();
            ctx.arc(x, y, r.range(0.5, 1.4), 0, PI * 2.0);
            ctx.fill();
        }
    })
}

/// The cut face (u across it, v up it): dry crumb, its holes, the crust's
/// edge round it, and the mould grown over it from its rim: blue-green
/// colonies with white rims, a yellow stain round them, black pin mould
fn loaf_crumb_tex(salt: u32) -> Tex {
    canvas_tex_opt(192, 160, false, move |ctx, w, h| {
        let seed = salt ^ 0xc4b3;
        let mut r = SplitMix64::new(salt, 0x2c8d);
        let cols: Vec<(f64, f64, f64)> = (0..9)
            .map(|k| {
                let a = r.range(0.0, PI * 2.0);
                let d = r.range(0.25, 0.48);
                (w * (0.5 + a.cos() * d), h * (0.55 + a.sin() * d * 0.9), if k < 3 { r.range(18.0, 34.0) } else { r.range(5.0, 12.0) })
            })
            .collect();
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = (i as f64, j as f64);
                let holes = loaf_noise(seed, x / 2.2, y / 2.6);
                let big = loaf_noise(seed ^ 1, x / 6.0, y / 7.0);
                let mut c = [186.0, 172.0, 140.0];
                let k = 0.92 + 0.12 * holes - 0.35 * (0.3 - holes).max(0.0) - 0.4 * (0.22 - big).max(0.0);
                for q in 0..3 {
                    c[q] *= k;
                }
                // dried and greyed toward the crust
                let (dx, dy) = ((x / w - 0.5) * 2.0, (y / h - 0.55) * 2.1);
                let rim = (dx * dx + dy * dy).sqrt();
                loaf_mix(&mut c, [150.0, 136.0, 108.0], (rim - 0.6).max(0.0) * 1.5);
                let (s, wr, st) = loaf_mould(&cols, x, y, seed ^ 7);
                loaf_mix(&mut c, [160.0, 144.0, 96.0], st * 0.4);
                loaf_mix(&mut c, [210.0, 210.0, 200.0], wr * 0.75);
                let sp = loaf_noise(seed ^ 9, x / 1.4, y / 1.4);
                loaf_mix(&mut c, [88.0 + 26.0 * sp, 104.0 + 20.0 * sp, 88.0 + 14.0 * sp], s * 0.9);
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        for _ in 0..24 {
            let a = r.range(0.0, PI * 2.0);
            let d = r.range(0.1, 0.5);
            let (x, y) = (w * (0.5 + a.cos() * d), h * (0.55 + a.sin() * d));
            ctx.set_fill_style("rgba(28,28,26,0.85)");
            ctx.begin_path();
            ctx.arc(x, y, r.range(0.5, 1.3), 0, PI * 2.0);
            ctx.fill();
        }
    })
}

/// The board's top (u along it, v across): beech, its grain along it, oiled
/// once and long dry, greyed; scored by the bread knife across and along,
/// darker where the loaf lay and round its foot; a water ring
fn loaf_board_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 352, false, move |ctx, w, h| {
        let seed = salt ^ 0xb0a2;
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = j as f64;
            for i in 0..n {
                let x = i as f64;
                let grain = loaf_noise(seed, x / 60.0, y / 1.6) * 0.6 + loaf_noise(seed ^ 1, x / 14.0, y / 0.8) * 0.4;
                let fleck = (loaf_noise(seed ^ 2, x / 1.5, y / 3.0) - 0.8).max(0.0) * 3.0;
                let mut c = [152.0 + 22.0 * (grain - 0.5), 124.0 + 18.0 * (grain - 0.5), 92.0 + 14.0 * (grain - 0.5)];
                loaf_mix(&mut c, [120.0, 92.0, 64.0], fleck);
                // greyed with years and dust
                let dust = loaf_fbm(seed ^ 3, x / 50.0, y / 50.0);
                loaf_mix(&mut c, [126.0, 118.0, 106.0], 0.35 + 0.3 * dust);
                // darker where the loaf has lain on it (its middle at x_0)
                let (lu, lv) =
                    (((x / w) * LOAF_BOARD.0 - LOAF_BOARD.0 / 2.0 - LOAF_X0 + 0.035) / 0.12, (y / h - 0.5) * LOAF_BOARD.1 / 0.06);
                let under = (1.0 - (lu * lu + lv * lv).sqrt()).max(0.0);
                loaf_mix(&mut c, [104.0, 84.0, 60.0], under * 0.5);
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        let mut r = SplitMix64::new(salt, 0x60a4);
        // knife scores: fine dark lines, most across the board
        for _ in 0..90 {
            let (x, y) = (r.next() * w, r.next() * h);
            let a = PI / 2.0 + r.range(-0.6, 0.6) + if r.next() < 0.25 { PI / 2.0 } else { 0.0 };
            let l = r.range(10.0, 60.0);
            ctx.set_stroke_style(&format!("rgba(70,54,40,{:.2})", r.range(0.15, 0.45)));
            ctx.set_line_width(r.range(0.5, 1.1));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        // a water ring from a mug, long ago
        let (cx, cy) = (w * r.range(0.7, 0.85), h * r.range(0.25, 0.75));
        ctx.set_stroke_style("rgba(84,66,48,0.18)");
        ctx.set_line_width(1.6);
        ctx.begin_path();
        ctx.arc(cx, cy, 26.0, 0.3, 5.6);
        ctx.stroke();
    })
}

// Wine bottle: 6% chance per level.
fn place_wine_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let glass_colors: [u32; 3] = [0x304018, 0x603010, 0x201820];
    let glass_mat = lambert!({ color: pick(&glass_colors) });
    let foil_colors: [u32; 3] = [0xa02020, 0xd0a040, 0x606060];
    let foil_mat = lambert!({ color: pick(&foil_colors) });
    let label_mat = lambert!({ color: 0xf0e8c8 });
    let bottle = group();
    let body = mesh(pg("WineBottles__body", cyl_geo!(0.036, 0.036, 0.18, 12)), &glass_mat);
    body.position().y = 0.09;
    bottle.add(&body);
    let shoulder = mesh(pg("WineBottles__shoulder", cyl_geo!(0.014, 0.036, 0.030, 12)), &glass_mat);
    shoulder.position().y = 0.195;
    bottle.add(&shoulder);
    let neck = mesh(pg("WineBottles__neck", cyl_geo!(0.014, 0.014, 0.06, 10)), &glass_mat);
    neck.position().y = 0.24;
    bottle.add(&neck);
    let foil = mesh(pg("WineBottles__foil", cyl_geo!(0.016, 0.016, 0.020, 10)), &foil_mat);
    foil.position().y = 0.28;
    bottle.add(&foil);
    let label = mesh(plane_geo!(0.11, 0.070), &label_mat);
    label.scale().x = 0.5; // flat label kept inside the bottle's 0.072 m silhouette (was 0.11 wide)
    label.position().set(0, 0.10, 0.0365);
    bottle.add(&label);
    if rnd() < 0.20 {
        bottle.rotation().z = PI / 2.0;
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let py = if bottle.rot().z != 0.0 { 0.036 } else { 0.0 }; // lying: axis one radius up, not on the floor
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    bottle.position().set(px, py, pz);
    bottle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bottle);
}

// Cheese wheel wedge: 7% chance per level.
fn place_cheese_wedges(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let cheese_colors: [u32; 4] = [0xf0d040, 0xd0a020, 0xe8c060, 0xf0e090];
    let cheese_mat = lambert!({ color: pick(&cheese_colors) });
    let rind_mat = lambert!({ color: 0xa08020 });
    let wedge = group();
    let body = mesh(pg("CheeseWedges__body", cyl_geo!(0.055, 0.055, 0.040, 3, 1, false, 0, PI * 0.5)), &cheese_mat);
    // Lying on its triangular side (quarter disc flat, x/z 0..0.055), rind lips
    // resting on the floor; was stood on its rind arc, half under the floor.
    body.position().y = 0.0221;
    wedge.add(&body);
    let rind = mesh(pg("CheeseWedges__rind", cyl_geo!(0.056, 0.056, 0.042, 8, 1, false, 0, PI * 0.5)), &rind_mat);
    rind.position().y = 0.0221;
    rind.scale().y = 1.05;
    wedge.add(&rind);
    for i in 0..3 {
        let hole = mesh(circle_geo!(0.005 + rnd() * 0.004, 6), lambert!({ color: 0xa08020 }));
        hole.rotation().x = -PI / 2.0;
        // on the wedge's top face (0.0421), inside the quarter disc
        hole.position().set(0.010 + i as f64 * 0.013, 0.0426, 0.010 + rnd() * 0.012);
        wedge.add(&hole);
    }
    wedge.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    wedge.rotation().y = rnd() * PI * 2.0;
    g.world.add(&wedge);
}

// Pencil eyeliner: 9% chance per level. Slim tapered pencil.
fn place_pencil_eyeliners(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors: [u32; 4] = [0x202020, 0x604060, 0x804020, 0x2060a0];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let tip_mat = lambert!({ color: 0x101010 });
    let pen = group();
    let body = mesh(pg("PencilEyeliners__body", cyl_geo!(0.004, 0.004, 0.075, 6)), &body_mat);
    body.rotation().z = PI / 2.0;
    body.position().y = 0.004;
    pen.add(&body);
    let tip = mesh(pg("PencilEyeliners__tip", cone_geo!(0.004, 0.014, 5)), &tip_mat);
    tip.rotation().z = -PI / 2.0;
    tip.position().set(0.044, 0.004, 0);
    pen.add(&tip);
    let cap = mesh(pg("PencilEyeliners__cap", cyl_geo!(0.005, 0.005, 0.020, 6)), &body_mat);
    cap.rotation().z = PI / 2.0;
    cap.position().set(-0.046, 0.005, 0); // r 5 mm: rests on the floor
    pen.add(&cap);
    pen.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pen.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pen);
}

// Kitchen wall clock: 4% chance per level. Round face
// w/ 4 tick marks + hands.
fn place_kitchen_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.kitchen_clock_positions.clear(); // reset per level
    if rnd() < 0.96 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let rim_colors: [u32; 5] = [0xa02020, 0xd0a040, 0x2060a0, 0x606060, 0x408060];
    let rim_mat = lambert!({ color: pick(&rim_colors) });
    let face_mat = lambert!({ color: 0xf8f4e8 });
    let hand_mat = lambert!({ color: 0x101010 });
    let clock = group();
    let mount_y = 1.95;
    let rim = mesh(pg("KitchenClocks__case", cyl_geo!(0.11, 0.11, 0.024, 20)), &rim_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = mount_y;
    clock.add(&rim);
    let face = mesh(circle_geo!(0.105, 20), &face_mat);
    face.position().set(0, mount_y, 0.013);
    clock.add(&face);
    // 4 tick marks at cardinal positions.
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0 - PI / 2.0;
        let tick = mesh(pg("KitchenClocks__tick", box_geo!(0.006, 0.020, 0.003)), &hand_mat);
        tick.position().set(a.cos() * 0.090, mount_y + a.sin() * 0.090, 0.014);
        tick.rotation().z = a + PI / 2.0; // radial baton (was tangential)
        clock.add(&tick);
    }
    // Frozen hands, each turned about the hub (pivot group), not its own centre.
    let hour = mesh(pg("KitchenClocks__hour", box_geo!(0.006, 0.050, 0.003)), &hand_mat);
    let hour_pivot = group();
    hour_pivot.position().set(0, mount_y, 0.016);
    hour.position().set(0, 0.025, 0);
    hour_pivot.rotation().z = rnd() * PI * 2.0;
    hour_pivot.add(&hour);
    clock.add(&hour_pivot);
    let minute = mesh(pg("KitchenClocks__minute", box_geo!(0.004, 0.075, 0.003)), &hand_mat);
    let minute_pivot = group();
    minute_pivot.position().set(0, mount_y, 0.017);
    minute.position().set(0, 0.037, 0);
    minute_pivot.rotation().z = rnd() * PI * 2.0;
    minute_pivot.add(&minute);
    clock.add(&minute_pivot);
    let center = mesh(circle_geo!(0.006, 8), &hand_mat);
    center.position().set(0, mount_y, 0.018);
    clock.add(&center);
    clock.rotation().y = face_wall(dx, dz);
    clock.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&clock);
    let p = clock.pos();
    g.pp.kitchen_clock_positions.push(v3(p.x, 0, p.z));
}

// Rolling pin stand w/ 2 pins: 4% chance per level.
fn place_rolling_pin_stands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let pin_color1 = lambert!({ color: 0xd0b080 });
    let pin_color2 = lambert!({ color: 0xa07050 });
    let stand = group();
    // Base plate.
    let base = mesh(pg("RollingPinStands__base", box_geo!(0.30, 0.010, 0.10)), &wood_mat);
    base.position().y = 0.005;
    stand.add(&base);
    // 4 short vertical uprights.
    for sx in [-1.0, 1.0] {
        for sz in [-1.0, 1.0] {
            let post = mesh(pg("RollingPinStands__post", cyl_geo!(0.006, 0.006, 0.045, 5)), &wood_mat);
            post.position().set(sx * 0.130, 0.028, sz * 0.035);
            stand.add(&post);
        }
    }
    // 2 rolling pins resting horizontally on posts.
    for i in 0..2 {
        let pin_mat = if i == 0 { &pin_color1 } else { &pin_color2 };
        // Side by side, each pin's handles resting on the tops (0.0505) of its
        // pair of posts (z +-0.035); they were stacked 8 mm into each other.
        let pin_y = 0.057;
        let pin_z = if i == 0 { -0.030 } else { 0.030 };
        let roller = mesh(pg("RollingPinStands__roller", cyl_geo!(0.020, 0.020, 0.24, 10)), pin_mat);
        roller.rotation().z = PI / 2.0;
        roller.position().set(0, pin_y, pin_z);
        stand.add(&roller);
        for side in [-1.0, 1.0] {
            let stem = mesh(pg("RollingPinStands__handle", cyl_geo!(0.008, 0.008, 0.040, 6)), pin_mat);
            stem.rotation().z = PI / 2.0;
            stem.position().set(side * 0.140, pin_y, pin_z);
            stand.add(&stem);
            let knob = mesh(pg("RollingPinStands__knob", sphere_geo!(0.012, 6, 5)), pin_mat);
            knob.position().set(side * 0.164, pin_y, pin_z);
            stand.add(&knob);
        }
    }
    stand.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    stand.rotation().y = rnd() * PI * 2.0;
    g.world.add(&stand);
}

// Pie slice on plate: 7% chance per level. Triangular
// wedge of pie (crust + filling) resting on ceramic plate.
fn place_pie_slices(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Filling color pool = variety of pie types.
    let filling_colors: [u32; 6] = [
        0xa02020, // cherry
        0x6a3a10, // pecan
        0xd0a040, // lemon meringue
        0x502010, // chocolate
        0x8040a0, // blueberry
        0xf0a060, // pumpkin
    ];
    let plate_colors: [u32; 3] = [0xffffff, 0xf0e8d8, 0xe8e0c8];
    let plate_mat = lambert!({ color: pick(&plate_colors) });
    let crust_mat = lambert!({ color: 0xc89060 });
    let filling_mat = lambert!({ color: pick(&filling_colors) });
    let pie = group();
    // Plate (shallow disc w/ raised rim).
    let plate = mesh(pg("PieSlices__plate", cyl_geo!(0.090, 0.082, 0.006, 16)), &plate_mat);
    plate.position().y = 0.003;
    pie.add(&plate);
    let plate_rim = mesh(pg("PieSlices__rim", torus_geo!(0.086, 0.004, 4, 16)), &plate_mat);
    plate_rim.rotation().x = PI / 2.0;
    plate_rim.position().y = 0.008;
    pie.add(&plate_rim);
    // Pie slice = partial cylinder with a 60° theta length.
    let slice_h = 0.028;
    let slice_r = 0.075;
    let slice_geom = pg("PieSlices__filling", cyl_geo!(slice_r, slice_r, slice_h, 10, 1, false, 0, PI / 3.0));
    let filling = mesh(slice_geom, &filling_mat);
    filling.position().y = 0.006 + slice_h / 2.0;
    filling.rotation().y = rnd() * PI * 2.0;
    pie.add(&filling);
    // Top crust cap (thin) on the filling — slightly lighter crust color.
    let top_crust = mesh(pg("PieSlices__crust", cyl_geo!(slice_r, slice_r, 0.004, 10, 1, false, 0, PI / 3.0)), &crust_mat);
    top_crust.position().y = 0.006 + slice_h - 0.002;
    top_crust.rotation().y = filling.rot().y;
    pie.add(&top_crust);
    // Optional dollop of cream: 40% chance.
    if rnd() < 0.4 {
        let cream = mesh(pg("PieSlices__cream", sphere_geo!(0.012, 6, 5)), &plate_mat);
        // Position roughly near the pointed apex of the slice.
        // The wedge's centre line is at (sin, cos) of its angle (the cylinder's theta
        // convention), 3 cm out where the slice is wide enough for the dollop.
        let ang = filling.rot().y + PI / 6.0;
        cream.position().set(ang.sin() * 0.030, 0.006 + slice_h + 0.004, ang.cos() * 0.030);
        cream.scale().y = 0.7;
        pie.add(&cream);
    }
    pie.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    pie.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pie);
}

// Cake stand, cornbread muffins: batch `t13`'s (`t13.rs`)
mod t13;
use t13::{place_cake_stands, place_cornbread_muffins};

// Sewing basket: 4% chance per level. Wicker basket
// w/ folded fabric + spool of thread + optional needles.
fn place_sewing_baskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Wicker color pool — earth tones.
    let wicker_colors: [u32; 4] = [0xa07040, 0x8b6030, 0xc09060, 0x704a20];
    let wicker_mat = lambert!({ color: pick(&wicker_colors) });
    let basket = group();
    // Basket body (open cylinder, tapered).
    let body = mesh(pg("SewingBaskets__body", cyl_geo!(0.10, 0.075, 0.070, 12, 1, true)), &wicker_mat);
    body.position().y = 0.035;
    basket.add(&body);
    // Basket bottom disc.
    let bottom = mesh(circle_geo!(0.075, 12), &wicker_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.001;
    basket.add(&bottom);
    // Rim (torus at top).
    let rim = mesh(pg("SewingBaskets__rim", torus_geo!(0.10, 0.006, 5, 14)), &wicker_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.070;
    basket.add(&rim);
    // Handle (partial torus arching over top).
    let handle = mesh(pg("SewingBaskets__handle", torus_geo!(0.095, 0.005, 4, 10, PI)), &wicker_mat);
    // Upright half hoop (XY plane) arching over the basket, ends on the rim
    // (the old x/z turns laid it flat inside the rim).
    handle.position().y = 0.070;
    basket.add(&handle);
    // Folded fabric inside (flat box — random pastel).
    let fabric_colors: [u32; 6] = [0xf0c0c0, 0xc0d0f0, 0xd0f0c0, 0xf0e8a0, 0xe0c0f0, 0xa0d0e0];
    let fabric_mat = lambert!({ color: pick(&fabric_colors) });
    let fabric = mesh(pg("SewingBaskets__fabric", box_geo!(0.12, 0.024, 0.09)), &fabric_mat);
    fabric.position().set(0, 0.055, 0);
    fabric.rotation().y = (rnd() - 0.5) * 0.4;
    basket.add(&fabric);
    // Spool of thread (small cylinder w/ different color).
    let thread_colors: [u32; 5] = [0x202020, 0xa02020, 0x2060a0, 0xf0f0f0, 0x408060];
    let spool_mat = lambert!({ color: 0xd8b878 });
    let thread_mat = lambert!({ color: pick(&thread_colors) });
    let spool_base = mesh(pg("SewingBaskets__spoolend", cyl_geo!(0.014, 0.014, 0.004, 8)), &spool_mat);
    spool_base.position().set(0.045, 0.070, 0.020);
    basket.add(&spool_base);
    let spool_thread = mesh(pg("SewingBaskets__thread", cyl_geo!(0.012, 0.012, 0.020, 8)), &thread_mat);
    spool_thread.position().set(0.045, 0.082, 0.020);
    basket.add(&spool_thread);
    let spool_top = mesh(pg("SewingBaskets__spoolend", cyl_geo!(0.014, 0.014, 0.004, 8)), &spool_mat);
    spool_top.position().set(0.045, 0.094, 0.020);
    basket.add(&spool_top);
    // Optional pincushion — 60% chance: tiny red hemisphere w/ pin marks.
    if rnd() < 0.6 {
        let cushion_mat = lambert!({ color: 0xa02020 });
        let cushion = mesh(pg("SewingBaskets__cushion", sphere_geo!(0.020, 6, 5, 0, PI * 2.0, 0, PI / 2.0)), &cushion_mat);
        cushion.position().set(-0.040, 0.067, 0.025); // on the fabric top (0.067)
        basket.add(&cushion);
        // 3 tiny pin heads (small spheres, white).
        let pin_head_mat = lambert!({ color: 0xf8f8f8 });
        let mut pins: Vec<(Obj, f64)> = vec![];
        for i in 0..3 {
            let ang = (i as f64 / 3.0) * PI * 2.0;
            let pin = mesh(sphere_geo!(0.003, 4, 3), &pin_head_mat);
            pin.position().set(-0.040 + ang.cos() * 0.010, 0.087, 0.025 + ang.sin() * 0.010);
            basket.add(&pin);
            let by = pin.pos().y;
            pins.push((pin, by));
        }
        // Subtle pincushion bob animator — 40% chance.
        // Tiny Y-bob on cushion + pins together (as if soft fabric
        // is settling from being pressed).
        if rnd() < 0.4 {
            let bob_phase = rnd() * PI * 2.0;
            let cushion_base_y = cushion.pos().y;
            g.animate(move |_g, _dt, t| {
                let bob = (t * 1.4 + bob_phase).sin() * 0.0006;
                cushion.position().y = cushion_base_y + bob;
                for (m, base_y) in &pins {
                    m.position().y = *base_y + bob;
                }
            });
        }
    }
    basket.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    basket.rotation().y = rnd() * PI * 2.0;
    g.world.add(&basket);
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
