//! `place_small_ribbons`: Curled gift ribbon: 10% chance per level. Curling ribbon cut from round a present (Blender batch r14b: 3/16 in polypropylene, crimped and glossy): the knot with four tails rolled in ringlets, or a single cut length curled at its ends; faded, trodden flat in places.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::seed_of;

/// Curled gift ribbon: 10% chance per level. Curling ribbon cut from
/// round a present (Blender batch r14b: 3/16 in polypropylene, crimped and
/// glossy): the knot with four tails rolled in ringlets, or a single cut
/// length curled at its ends; faded, trodden flat in places.
pub(crate) fn place_small_ribbons(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour draw: the ribbon's colour, faded
    let ribbon_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xf0a0a0];
    let c = pick(&ribbon_colors);
    let k = ribbon_colors.iter().position(|&r| r == c).unwrap_or(0);
    let ribbon = group();
    ribbon.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    ribbon.rotation().y = rnd() * PI * 2.0;
    let p = ribbon.pos();
    let seed = seed_of(&[p.x, p.z]);
    let faded = [0xa85464, 0x56689a, 0xb8a052, 0x5a8458, 0x865684, 0xcaa29e][k];
    let mat = phong!({ color: faded, specular: 0x585858, shininess: 52, side: DoubleSide });
    mat.m().map = Some(faded_tex(seed));
    let name = if seed % 10 < 7 { "Real__ribbonKnot" } else { "Real__ribbonLength" };
    if let Some(geo) = prop_part_as_modelled(name) {
        ribbon.add(&mesh(geo, &mat));
    }
    g.world.add(&ribbon);
}

/// Fading over the ribbon's colour (multiplied): bleached paler in
/// patches where the light lay, grimy where it was trodden
fn faded_tex(seed: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        ctx.set_fill_style("#e8e8e8");
        ctx.fill_rect(0, 0, w, h);
        let mut r = SplitMix64::new(seed, 0x21bb);
        for _ in 0..10 {
            let (x, y, rr) = (r.range(0.0, w), r.range(0.0, h), r.range(6.0, 18.0));
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(255,255,255,0.7)");
            gr.add_color_stop(1, "rgba(255,255,255,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        for _ in 0..8 {
            let (x, y, rr) = (r.range(0.0, w), r.range(0.0, h), r.range(4.0, 12.0));
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(70,60,50,0.55)");
            gr.add_color_stop(1, "rgba(70,60,50,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        for _ in 0..40 {
            ctx.set_fill_style(format!("rgba(40,34,28,{:.2})", r.range(0.2, 0.6)));
            ctx.fill_rect(r.range(0.0, w), r.range(0.0, h), r.range(0.6, 1.6), r.range(0.6, 1.6));
        }
    })
}
