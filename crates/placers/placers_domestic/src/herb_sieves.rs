//! `place_herb_sieves`
use crate::shared::NAIL_R;
use crate::shared::NAIL_RISE;
use crate::shared::WALL_Z;
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::paint::fill;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::salt;
use placers_kit::place::wall_pos;
use placers_kit::place::wall_tiles;

// Hanging herb sieve: 4% chance per level. A 7 in fine-mesh strainer
// (family `kitchen_hall_and_floor_things`: a tinned band round a woven mesh bowl, a looped wire handle
// and a pan hook) hung by its handle's loop on a nail, its rim to the wall.
pub(crate) fn place_herb_sieves(g: &mut Game, _dist: &[i32]) {
    g.pp.herb_sieve_positions.clear(); // reset per level
    if rnd() < 0.96 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    // the old metal draw: stainless, a brass-wire sieve, tinned steel gone dull
    let metal_colors: [u32; 3] = [0xb0b0b8, 0xa08040, 0x606060];
    let m = pick(&metal_colors);
    let kind = metal_colors.iter().position(|&c| c == m).unwrap_or(0);
    let salt = salt(g.wx(x), g.wz(z), 0x5e5 ^ kind as u32);
    let sieve = group();
    let nail_y = 1.84;
    let steel = phong!({ color: 0x4e4c48, specular: 0x3a3a38, shininess: 30 });
    if let Some(n) = put(&sieve, "Real__colanderNail", &steel) {
        n.position().set(0, nail_y, WALL_Z);
    }
    // hung by its loop: the loop's top (0.4 mm under its frame's origin,
    // 11.8 mm out) on the nail's top there
    let hang = group();
    hang.position().set(0, nail_y + 0.0118 * NAIL_RISE + NAIL_R + 0.0004, WALL_Z);
    let (metal, spec) = [(0x76787a, 0x5a5a5a), (0x6e5a34, 0x4a3a20), (0x5a5854, 0x3a3a38)][kind];
    put(&hang, "Real__sieveFrame", &phong!({ color: metal, specular: spec, shininess: 40 }));
    put(&hang, "Real__sieveMesh", &lambert!({ map: &sieve_mesh_tex(kind, salt), transparent: true, depthWrite: false, side: DoubleSide }));
    sieve.add(&hang);
    sieve.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    sieve.position().set(px, 0, pz);
    g.world.add(&sieve);
    let p = sieve.pos();
    g.pp.herb_sieve_positions.push(v3(p.x, 0, p.z));
}

/// The sieve's woven mesh, seen from in front (u along x, v up): its wires
/// over and under about 1.4 mm apart, the light through it; flour and
/// dried leaf caught in it, darker where it was scrubbed least
fn sieve_mesh_tex(kind: usize, salt: u32) -> Tex {
    canvas_tex_opt(512, 512, false, move |ctx, w, h| {
        let seed = salt ^ 0x51e;
        let wire = [[0.55, 0.56, 0.57], [0.50, 0.40, 0.22], [0.42, 0.41, 0.39]][kind];
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u * w / 4.0, v * h / 4.0);
            let (fx, fy) = (x - x.floor(), y - y.floor());
            // the warp and the weft, each passing over and under the other
            let over = ((x.floor() + y.floor()) as i64 % 2) == 0;
            let wx = (1.0 - ((fx - 0.5).abs() / 0.17)).max(0.0);
            let wy = (1.0 - ((fy - 0.5).abs() / 0.17)).max(0.0);
            let shade = if over { 0.85 + 0.15 * wx } else { 0.85 + 0.15 * wy };
            let a_wire = wx.max(wy).min(1.0);
            let clog = sstep(0.66, 0.82, fbm(seed, u * 7.0, v * 7.0, 4));
            let dirt = fbm(seed ^ 3, u * 30.0, v * 30.0, 2);
            let mut c = [wire[0] * shade, wire[1] * shade, wire[2] * shade];
            c = mix(c, [0.36, 0.33, 0.27], fbm(seed ^ 5, u * 4.0, v * 4.0, 3) * 0.4);
            let fill_c = mix([0.46, 0.43, 0.37], [0.26, 0.25, 0.18], dirt);
            let a = (a_wire.dpowf(0.6) * 0.95 + 0.04).min(1.0);
            let c = mix(c, fill_c, clog * (1.0 - a_wire * 0.5));
            [c[0], c[1], c[2], a.max(clog * 0.9)]
        });
    })
}
