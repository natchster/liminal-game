//! `place_mason_jars`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Mason jar: 7% chance per level. Glass body + metal
// screw lid + optional interior contents.
pub(crate) fn place_mason_jars(g: &mut Game, _dist: &[i32]) {
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
