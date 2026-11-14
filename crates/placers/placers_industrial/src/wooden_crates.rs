//! `place_wooden_crates`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Wooden crate: 15% chance per level. A nailed slatted crate of rough
// sawn pine gone grey (Blender family `crate_sash_umbrella_and_things_put_away`): its boards with gaps between
// them nailed to corner posts, a slatted lid, wood wool packed inside;
// stencilled on its long sides. Now and then a board stove in, splinters and
// wood wool on the floor by it; a smaller crate stacked on it.
pub(crate) fn place_wooden_crates(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // the pine as it weathered: grey, grey-brown, a dull honey, darkened by damp
    let tones = [0x8c8476, 0x7e7262, 0x9a8566, 0x6e665a];
    let tone = tones[ri(tones.len())];
    let cs = 0.35 + rnd() * 0.10; // 35-45 cm: the 0.40 m crate, a little smaller or larger
    let s = cs / CRATE_W;
    // (what is painted on it differs from crate to crate by its size: no draw of its own)
    let look = CrateLook::new(tone, (cs * 7919.0).fract());
    let (crate_, sides) = crate_group("", &look);
    crate_.scale().set(s, s, s);
    crate_.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    crate_.rotation().y = rnd() * PI * 2.0;
    g.world.add(&crate_);
    // 20% chance: one or two boards stove in (a face, a board), wood
    // wool pulled out through the hole
    let has_broken_holes = rnd() < 0.20;
    if has_broken_holes {
        let hole_count = 1 + (rnd() * 2.0).floor() as i32;
        // (face -> its boards broken: +z, -z, +x, -x as the face draw counts them)
        let mut broken: [Vec<usize>; 4] = Default::default();
        for _hh in 0..hole_count {
            let face = (rnd() * 4.0).floor() as usize;
            let hy = cs * 0.3 + rnd() * cs * 0.4;
            let (_, n, _, gap, _, _, bw) = crate_dims("");
            let k = ((hy / s / (bw + gap)).floor() as usize).clamp(1, n - 2);
            if !broken[face].contains(&k) {
                broken[face].push(k);
            }
            // the wood wool pulled out through it, hanging from the gap
            let (hx_off, hz_off) = [(0.0, 1.0), (0.0, -1.0), (1.0, 0.0), (-1.0, 0.0)][face];
            let cp = crate_.pos();
            let cry = crate_.rot().y;
            let yk = (k as f64 * (bw + gap) + bw * 0.35) * s;
            let tuft = group();
            if let Some(gm) = game::assets::props::prop_part_as_modelled("Real__crateWoolTuft") {
                let m = mesh(gm, &look.wool);
                m.rotation().x = -1.2;
                m.position().set(0.0, -0.012, -0.012);
                tuft.add(&m);
            }
            let r = cs / 2.0 + 0.006;
            let (lx, lz) = (hx_off * r, hz_off * r);
            tuft.position().set(cp.x + lx * cry.dcos() + lz * cry.dsin(), yk, cp.z - lx * cry.dsin() + lz * cry.dcos());
            tuft.rotation().y = cry + [0.0, PI, PI / 2.0, -PI / 2.0][face];
            tuft.scale().set(s, s, s);
            g.world.add(&tuft);
        }
        // each stove-in face: its side made again with those boards snapped
        // (slot: +z 0, +x 1, -z 2, -x 3)
        for (face, ks) in broken.iter().enumerate() {
            if ks.is_empty() {
                continue;
            }
            let slot = [0, 2, 1, 3][face];
            let tag = if ks.len() > 1 {
                "12"
            } else if ks[0] == 1 {
                "1"
            } else {
                "2"
            };
            let ab = if slot % 2 == 0 { "A" } else { "B" };
            if let Some(gm) = game::assets::props::prop_part_as_modelled(&format!("Real__crateSide{ab}Broken{tag}")) {
                sides[slot].set_geo(gm);
            }
        }
    }
    // If broken holes: 50% chance 4-6 splinters of the board and tufts
    // of wood wool on the floor by it
    if has_broken_holes && rnd() < 0.50 {
        let pellet_count = 4 + (rnd() * 3.0).floor() as i32;
        for p in 0..pellet_count {
            let size = 0.008 + rnd() * 0.004;
            let p_ang = rnd() * PI * 2.0;
            let p_rad = cs * 0.5 + rnd() * 0.30;
            let cp = crate_.pos();
            // clear of the crate's own footprint (it is square, and turned)
            let cry = crate_.rot().y;
            let (dx, dz) = (p_ang.dcos(), p_ang.dsin());
            let (lx, lz) = (dx * cry.dcos() - dz * cry.dsin(), dx * cry.dsin() + dz * cry.dcos());
            let foot = cs / 2.0 / lx.abs().max(lz.abs()).max(1e-3) + 0.03;
            let rad = p_rad.max(foot);
            let name = if p % 2 == 0 { "Real__crateSplinter" } else { "Real__crateWoolTuft" };
            let bit = group();
            if let Some(gm) = game::assets::props::prop_part_as_modelled(name) {
                let m = mesh(gm, if p % 2 == 0 { &look.split } else { &look.wool });
                bit.add(&m);
            }
            let k = 0.9 + (size - 0.008) / 0.004 * 0.2;
            bit.scale().set(k, k, k);
            bit.rotation().y = p_ang * 2.7 + p as f64;
            bit.position().set(cp.x + dx * rad, 0.0, cp.z + dz * rad);
            g.world.add(&bit);
        }
    }
    // 25% chance: a smaller crate stacked on its lid (freight pile)
    if rnd() < 0.25 {
        let cs2 = cs * (0.55 + rnd() * 0.20); // 55-75% of base
        // the smaller crates are their own (0.30 or 0.225 m, their boards
        // thinner), scaled within 15% of that
        let key = if cs2 < 0.26 { "S" } else { "M" };
        let (w2, ..) = crate_dims(key);
        let (sib_crate, _) = crate_group(key, &look);
        let s2 = cs2 / w2;
        sib_crate.scale().set(s2, s2, s2);
        let cp = crate_.pos();
        sib_crate.position().set(cp.x, cs, cp.z); // on the lid
        sib_crate.position().x += (rnd() - 0.5) * cs * 0.15;
        sib_crate.position().z += (rnd() - 0.5) * cs * 0.15;
        sib_crate.rotation().y = rnd() * PI * 2.0;
        g.world.add(&sib_crate);
    }
}

/// The big crate's side, m (`r09`'s `CRATES`)
const CRATE_W: f64 = 0.40;

/// A crate's sizes (`r09`'s `crate_dims`): W, boards a side, board
/// thickness, gap, post, the sides' height, a board's width
fn crate_dims(key: &str) -> (f64, usize, f64, f64, f64, f64, f64) {
    let (w, n, t, gap, post) = match key {
        "M" => (0.30, 3, 0.010, 0.018, 0.026),
        "S" => (0.225, 3, 0.009, 0.014, 0.022),
        _ => (CRATE_W, 4, 0.012, 0.020, 0.030),
    };
    let hs = w - t;
    (w, n, t, gap, post, hs, (hs - (n - 1) as f64 * gap) / n as f64)
}

/// What a crate is made of: its boards (grain, and the long sides'
/// stencils), the posts and bottom, the nails, the wood wool, the raw wood
/// of a splinter
struct CrateLook {
    side_a: Mat,
    side_b: Mat,
    frame: Mat,
    nails: Mat,
    wool: Mat,
    split: Mat,
}

impl CrateLook {
    fn new(tone: u32, v: f64) -> CrateLook {
        let wool_tex = crate_wool_tex();
        wool_tex.t().repeat.set(3, 3);
        CrateLook {
            side_a: lambert!({ color: tone, map: &crate_board_tex(true, v) }),
            side_b: lambert!({ color: tone, map: &crate_board_tex(false, v) }),
            frame: lambert!({ color: tone }),
            nails: lambert!({ color: 0x3a3029 }),
            wool: lambert!({ color: 0xb09a74, map: &wool_tex }),
            split: lambert!({ color: 0xa88f6c }),
        }
    }
}

/// A crate built of its parts as modelled (`r09`), its bottom at y = 0:
/// the group, and its four sides (+z, +x, -z, -x) so a stove-in one can be
/// swapped in
fn crate_group(key: &str, look: &CrateLook) -> (Obj, Vec<Obj>) {
    use game::assets::props::prop_part_as_modelled as part;
    let c = group();
    let (w, ..) = crate_dims(key);
    let mut sides = vec![];
    for slot in 0..4 {
        let ab = if slot % 2 == 0 { "A" } else { "B" };
        let geo = part(&format!("Real__crateSide{ab}{key}")).unwrap_or_else(|| {
            let g = box_geo!(w, w, 0.012);
            g.translate(0, w / 2.0, w / 2.0 - 0.006);
            g
        });
        let m = mesh(geo, if slot % 2 == 0 { &look.side_a } else { &look.side_b });
        m.rotation().y = slot as f64 * PI / 2.0;
        c.add(&m);
        sides.push(m);
    }
    for (name, mat) in [("Frame", &look.frame), ("Lid", &look.side_b), ("Nails", &look.nails), ("Wool", &look.wool)] {
        if let Some(geo) = part(&format!("Real__crate{name}{key}")) {
            c.add(&mesh(geo, mat));
        }
    }
    (c, sides)
}

/// A side's boards (u along them, v up the side): the grain of rough sawn
/// pine gone grey, its knots, each board a shade of its own, darker where
/// it stood in the wet; on the long sides a shipper's stencil, worn
fn crate_board_tex(stencil: bool, v: f64) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        // (the painter's own stream, moved on by this crate's own value)
        for _ in 0..(v * 61.0) as usize {
            rnd();
        }
        ctx.set_fill_style("#e4ded2");
        ctx.fill_rect(0, 0, w, h);
        // each board its own shade (4 boards and 3 gaps up the side)
        let (bw, gap) = (0.082 / 0.388 * h, 0.020 / 0.388 * h);
        for k in 0..4 {
            let y0 = h - (k as f64 * (bw + gap) + bw);
            let v = rnd();
            ctx.set_fill_style(format!("rgba({},{},{},{:.3})", 70 + (v * 40.0) as i32, 62 + (v * 30.0) as i32, 50, 0.06 + rnd() * 0.12));
            ctx.fill_rect(0, y0, w, bw);
        }
        // the grain: long wavy lines along the boards, and the fibres torn up by the saw
        for _ in 0..170 {
            let y = rnd() * h;
            let (a, f, ph) = (0.6 + rnd() * 2.0, 0.01 + rnd() * 0.03, rnd() * 6.3);
            ctx.set_stroke_style(format!("rgba(70,58,44,{:.3})", 0.05 + rnd() * 0.16));
            ctx.set_line_width(0.6 + rnd() * 1.4);
            ctx.begin_path();
            let x0 = rnd() * w * 0.5 - 20.0;
            let len = 60.0 + rnd() * 240.0;
            let mut x = x0;
            ctx.move_to(x, y);
            while x < x0 + len {
                x += 8.0;
                ctx.line_to(x, y + (x * f + ph).dsin() * a);
            }
            ctx.stroke();
        }
        for _ in 0..900 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_fill_style(format!("rgba(255,250,240,{:.3})", 0.05 + rnd() * 0.10));
            ctx.fill_rect(x, y, 2.0 + rnd() * 6.0, 1);
            ctx.set_fill_style(format!("rgba(50,42,34,{:.3})", 0.05 + rnd() * 0.10));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 4.0, 1);
        }
        // knots: a dark heart, its rings, the grain swept round it
        for _ in 0..(2 + (rnd() * 3.0) as i32) {
            let (x, y, r) = (rnd() * w, rnd() * h, 2.5 + rnd() * 4.5);
            for ring in (0..4).rev() {
                ctx.set_fill_style(format!("rgba(60,44,30,{:.3})", 0.10 + 0.18 * (4 - ring) as f64 / 4.0));
                ctx.begin_path();
                ctx.ellipse(x, y, r * (1.0 + ring as f64 * 0.9) * 1.7, r * (1.0 + ring as f64 * 0.6), 0, 0, 7);
                ctx.fill();
            }
        }
        // the weather: grey bleaching in streaks, and the damp it stood in at the foot
        for _ in 0..30 {
            ctx.set_fill_style(format!("rgba(200,200,196,{:.3})", 0.04 + rnd() * 0.08));
            ctx.fill_rect(rnd() * w, rnd() * h, 30.0 + rnd() * 120.0, 2.0 + rnd() * 6.0);
        }
        let damp = ctx.create_linear_gradient(0, h, 0, h * 0.6);
        damp.add_color_stop(0, "rgba(40,32,24,0.45)");
        damp.add_color_stop(1, "rgba(40,32,24,0)");
        ctx.set_fill_style(&damp);
        ctx.fill_rect(0, h * 0.6, w, h * 0.4);
        if stencil {
            // a shipper's stencil, sprayed black through a card: the spray's
            // soft edge and its overspray, the bridges of the letters left
            // open; then rubbed and weathered thin
            let marks: [(&str, &str); 6] = [
                ("FRAGILE", "No 47"),
                ("HANDLE WITH CARE", "LOT 3"),
                ("THIS SIDE UP", "No 112"),
                ("KEEP DRY", "No 8"),
                ("GLASS", "No 21"),
                ("DO NOT DROP", "LOT 9"),
            ];
            let (big, small) = marks[((v * 6.0) as usize).min(5)];
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            // the third board from the floor, and the second
            let row = |k: f64| h - (k * (bw + gap) + bw / 2.0);
            let (dx, dy) = ((rnd() - 0.5) * 16.0, (rnd() - 0.5) * 4.0);
            for (a, o) in [(0.2, 1.2), (0.2, -1.0), (0.5, 0.0)] {
                ctx.set_fill_style(format!("rgba(26,24,22,{a})"));
                ctx.set_font("bold 44px Avenir Next Condensed");
                ctx.fill_text_max(big, w / 2.0 + dx + o, row(2.0) + 2.0 + dy + o * 0.5, w * 0.84);
                ctx.set_font("bold 30px Avenir Next Condensed");
                ctx.fill_text_max(small, w * 0.5 + dx * 0.5 + o, row(1.0) + 2.0 - o * 0.4, w * 0.5);
            }
            for _ in 0..500 {
                let (x, y) = (w / 2.0 + dx + (rnd() - 0.5) * w * 0.9, row(2.0) + (rnd() - 0.5) * bw * 1.1);
                ctx.set_fill_style(format!("rgba(26,24,22,{:.3})", 0.1 + rnd() * 0.25));
                ctx.fill_rect(x, y, 1, 1);
            }
            ctx.set_global_composite_operation("destination-out");
            ctx.set_fill_style("rgba(0,0,0,1)");
            // the bridges: a thin bar of card across each letter's middle
            for k in 0..44 {
                let x = 8.0 + k as f64 * (w - 16.0) / 44.0 + rnd() * 2.0;
                ctx.fill_rect(x, row(2.0) - 1.0 + dy, 1.4, 3.0 + rnd() * 2.0);
            }
            // weathered: whole patches gone thin, flakes gone
            for _ in 0..60 {
                let (x, y, r) = (rnd() * w, row(1.0 + rnd() * 1.2), 3.0 + rnd() * 14.0);
                ctx.save();
                ctx.translate(x, y);
                ctx.scale(1, 0.45);
                let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
                let a = 0.2 + rnd() * 0.5;
                gr.add_color_stop(0, &format!("rgba(0,0,0,{a:.3})"));
                gr.add_color_stop(1, "rgba(0,0,0,0)");
                ctx.set_fill_style(&gr);
                ctx.begin_path();
                ctx.arc(0, 0, r, 0, 7);
                ctx.fill();
                ctx.restore();
            }
            ctx.set_global_composite_operation("source-over");
        }
        // worn: rubbed through to the wood here and there, and scuffed
        ctx.set_global_composite_operation("source-atop");
        for _ in 0..140 {
            ctx.set_fill_style(format!("rgba(214,206,192,{:.3})", 0.2 + rnd() * 0.5));
            ctx.begin_path();
            ctx.ellipse(rnd() * w, rnd() * h, 1.0 + rnd() * 6.0, 0.6 + rnd() * 2.0, 0, 0, 7);
            ctx.fill();
        }
        ctx.set_global_composite_operation("source-over");
    })
}

/// Wood wool: fine curled shavings, pale straw with darker ones among them
fn crate_wool_tex() -> Tex {
    canvas_tex(128, 128, |ctx, w, h| {
        ctx.set_fill_style("#8a7656");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..420 {
            let v = rnd();
            ctx.set_stroke_style(format!(
                "rgba({},{},{},{:.3})",
                190 + (v * 50.0) as i32,
                168 + (v * 50.0) as i32,
                120 + (v * 40.0) as i32,
                0.35 + rnd() * 0.5
            ));
            ctx.set_line_width(0.6 + rnd() * 1.0);
            let (x, y) = (rnd() * w, rnd() * h);
            let (a, l) = (rnd() * 6.3, 6.0 + rnd() * 26.0);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.quadratic_curve_to(x + (a + 1.2).dcos() * l, y + (a + 1.2).dsin() * l, x + a.dcos() * l * 1.4, y + a.dsin() * l * 1.4);
            ctx.stroke();
        }
    })
}
