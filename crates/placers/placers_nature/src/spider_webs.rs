//! `place_spider_webs`
use crate::shared::blob;
use crate::shared::css;
use game::prelude::*;
use placers_kit::geometry::part;
use placers_kit::noise::fbm;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::paint_px;
use placers_kit::place::pick_wall;
use placers_kit::place::salt_of;
use placers_kit::place::wall_mount;

// Spider web: 5% chance per level. A cellar spider's web in the angle of
// the wall and the ceiling (Blender family `bits_of_the_outside_indoors`): a loose tangle of fine
// threads grey with dust, guy lines to the wall and the ceiling, strands
// heavy with dust; the spider dead in it, hanging, prey wrapped and hung.
// It keeps the old web's place and its width along the wall.
pub(crate) fn place_spider_webs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let web = group();
    // the web's frame: the wall at z = 0, the ceiling at y = 0; the group
    // stands 2 cm off the wall, as the old one did
    let o = group();
    o.position().set(0, WALL_H, -0.020);
    web.add(&o);
    let threads = phong!({ color: 0xa8a49a, specular: 0x34332f, shininess: 30, transparent: true, opacity: 0.7, depthWrite: false });
    if let Some(geo) = part("Real__spiderWebThreads") {
        o.add(&mesh(geo, &threads));
    }
    let salt = salt_of(g.wx(x) + dx as f64, g.wz(z) + dz as f64);
    // the spider and its prey, where it hung them (not in every web: by where it is)
    if salt % 4 != 0 {
        if let Some(geo) = part("Real__spiderBody") {
            o.add(&mesh(geo, lambert!({ color: 0x8a7e6c })));
        }
    }
    if salt % 3 != 0 {
        if let Some(geo) = part("Real__spiderWrap") {
            o.add(&mesh(geo, lambert!({ color: 0x9e9a90 })));
        }
    }
    // the finest threads: a sheet painted across the angle (the threads'
    // hammock, `nature/bits_of_the_outside_indoors.py`: from the wall 20 cm under the ceiling to the
    // ceiling 12.5 cm out, sagging into the room), and a thinner one a
    // little under it where it sags (the old second sheet down the wall's
    // face read as a hairball on the wall, M8)
    let (down, out) = (0.20, 0.125);
    let (len, tilt) = (f64::hypot(down, out), out.atan2(down));
    // out of the angle, into the room: (y, z) = (-out, down) / len
    let sag = |d: f64| (-out / len * d, down / len * d);
    let veil = lambert!({ map: &web_veil_tex(salt), transparent: true, depthWrite: false, side: DoubleSide });
    let across = mesh(plane_geo!(0.32, len), &veil);
    let (sy, sz) = sag(0.010);
    across.position().set(0.0, -down / 2.0 + sy, out / 2.0 + sz);
    across.rotation().x = tilt;
    o.add(&across);
    let under = mesh(
        plane_geo!(0.24, len * 0.7),
        lambert!({ map: &web_veil_tex(salt ^ 0x51), transparent: true, depthWrite: false, side: DoubleSide }),
    );
    let (sy, sz) = sag(0.024);
    under.position().set(0.01, -down / 2.0 + sy, out / 2.0 + sz);
    under.rotation().x = tilt;
    o.add(&under);
    wall_mount(g, &web, x, z, dx, dz, 0.020);
    // it hangs from the ceiling and the wall by its threads: the placement
    // pass leaves it be (it never saw the old see-through web, and would let
    // the spider, the one solid thing in it, down to the floor)
    web.user().floats = true;
    g.world.add(&web);
}

// ==================================================================== web
/// The finest threads of the tangle, too many and too fine to model (seen
/// together they are a grey veil where the dust has caught in them): the
/// veil thickest in the tangle's heart, wisps along the ways it was spun,
/// strands crossing it, dust caught in clumps, torn through in places
/// (`salt`: where it hangs; the canvas's top is the ceiling's side)
fn web_veil_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0x3eb);
        let (bx, by) = (w * (0.42 + 0.16 * r.next()), h * (0.35 + 0.2 * r.next()));
        let (brx, bry) = (w * (0.3 + 0.1 * r.next()), h * (0.28 + 0.1 * r.next()));
        let (wa, wb) = (r.next() * PI, r.next() * PI);
        let dust = [0.66, 0.65, 0.62];
        paint_px(ctx, w, h, |x, y| {
            let (u, v) = (x / w, y / h);
            let q = ((x - bx) / brx).hypot((y - by) / bry);
            let dens = (-q * q * 0.9).exp();
            let rag = fbm(salt ^ 0x99, u * 7.0, v * 3.0, 3) - 0.5;
            let ends = sstep(0.0, 0.18, u.min(1.0 - u) + rag * 0.12) * sstep(0.0, 0.2, v.min(1.0 - v) + rag * 0.2);
            let (u1, u2) = (u * wa.cos() + v * wa.sin(), -u * wa.sin() + v * wa.cos());
            let (v1, v2) = (u * wb.cos() + v * wb.sin(), -u * wb.sin() + v * wb.cos());
            let wisp = 0.5 * vn(salt ^ 0x5e, u1 * 3.0, u2 * 45.0) + 0.5 * vn(salt ^ 0x6f, v1 * 3.0, v2 * 45.0);
            let torn = sstep(0.3, 0.45, fbm(salt ^ 0x7a, u * 4.0, v * 3.0, 4) + 0.2 * dens);
            [dust[0], dust[1], dust[2], ((0.03 + 0.5 * dens) * ends * (0.15 + 1.6 * wisp * wisp * wisp) * torn).min(0.5)]
        });
        ctx.set_line_cap("round");
        let in_blob = |r: &mut SplitMix64, k: f64| {
            let (a, d) = (r.next() * PI * 2.0, r.next().powf(0.6) * k);
            (bx + a.cos() * d * brx, by + a.sin() * d * bry)
        };
        // strands along the two ways it was spun, many over each other, each
        // sagging a little between where it is held
        for k in 0..230 {
            let c = in_blob(&mut r, 1.2);
            let a = if k % 2 == 0 { wa } else { wb } + (r.next() - 0.5) * 1.2;
            let l = w * (0.1 + 0.4 * r.next());
            let (dx, dy) = (a.cos() * l / 2.0, a.sin() * l / 2.0);
            let tone = if k % 7 == 0 { [0.56, 0.55, 0.52] } else { [0.88, 0.88, 0.86] };
            ctx.set_stroke_style(css(tone, 0.2 + 0.3 * r.next()));
            ctx.set_line_width(0.6 + 0.6 * r.next());
            let sag = l * (0.04 + 0.1 * r.next());
            ctx.begin_path();
            ctx.move_to(c.0 - dx, c.1 - dy);
            ctx.quadratic_curve_to(c.0, c.1 + sag, c.0 + dx, c.1 + dy);
            ctx.stroke();
        }
        // lines out to where it is fixed: the ceiling (top) and the wall (bottom)
        for _ in 0..14 {
            let a = in_blob(&mut r, 0.5);
            let f = (w * (0.05 + 0.9 * r.next()), if r.next() < 0.5 { 0.0 } else { h });
            let heavy = r.next() < 0.3;
            ctx.set_stroke_style(css(if heavy { [0.56, 0.54, 0.50] } else { [0.80, 0.79, 0.76] }, if heavy { 0.45 } else { 0.3 }));
            ctx.set_line_width(if heavy { 1.8 } else { 0.9 });
            ctx.begin_path();
            ctx.move_to(a.0, a.1);
            ctx.quadratic_curve_to((a.0 + f.0) / 2.0, (a.1 + f.1) / 2.0 + 6.0, f.0, f.1);
            ctx.stroke();
        }
        // dust: specks all through it, clumps where it is thickest
        for _ in 0..90 {
            let p = in_blob(&mut r, 1.0);
            ctx.set_fill_style(css([0.58, 0.56, 0.52], 0.2 + 0.35 * r.next()));
            ctx.begin_path();
            ctx.arc(p.0, p.1, 0.5 + 1.0 * r.next(), 0, 7);
            ctx.fill();
        }
        for _ in 0..8 {
            let p = in_blob(&mut r, 0.7);
            blob(ctx, p.0, p.1, 4.0 + 8.0 * r.next(), 0.5 + 0.5 * r.next(), r.next() * 3.0, [0.62, 0.60, 0.56], 0.45);
        }
    })
}
