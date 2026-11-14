//! `place_teething_rings`: Teething ring: 8% chance per level. A water-filled teething ring: two sheets of soft PVC welded round their edges and filled, 83 mm across, ribbed in two sectors and nubbed in two; lying flat. Its pastel faded, glossy still, the water gone cloudy with specks of black mould in it, grimed and dusty.
use game::prelude::*;
use placers_kit::geometry::put_world;
use placers_kit::noise::cell_edge;
use placers_kit::noise::fbmp;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vnp;
use placers_kit::paint::mul;
use placers_kit::paint::paint_tile;
use placers_kit::paint::rgb;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_at;

/// Teething ring: 8% chance per level. A water-filled teething ring:
/// two sheets of soft PVC welded round their edges and filled, 83 mm
/// across, ribbed in two sectors and nubbed in two; lying flat. Its pastel
/// faded, glossy still, the water gone cloudy with specks of black mould
/// in it, grimed and dusty.
pub(crate) fn place_teething_rings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old pastels, faded
    let ring_colors: [u32; 5] = [0xf0c0c0, 0xc0c0f0, 0xa0e0c0, 0xf0e0a0, 0xd0a0e0];
    let c = pick(&ring_colors);
    let k = ring_colors.iter().position(|&r| r == c).unwrap_or(0);
    let pvc = rgb([0xc6a29e, 0xa0a6c6, 0x92bca6, 0xc8b98e, 0xac96bc][k]);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    // the old wobble's phase: it lies still (the draw kept)
    let _ = rnd();
    let salt = salt_at(px, pz);
    let tex = paint_tile(128, move |u, v| {
        let w = Wear { base: pvc, alt: mix(pvc, [0.9, 0.9, 0.86], 0.45), rub: 0.0, grime: 0.4, scuff: 0.3, mottle: 0.08, scales: 0 };
        let mut c = wear_at(salt ^ 81, u, v, &w);
        // the water inside gone cloudy, and black mould in clusters of specks
        let cloud = fbmp(salt ^ 83, u, v, 2, 3);
        c = mix(c, [0.7, 0.7, 0.66], sstep(0.5, 0.8, cloud) * 0.25);
        let cluster = sstep(0.6, 0.75, fbmp(salt ^ 85, u, v, 3, 3));
        let s = hash(salt ^ 87, (u * 128.0) as i32, (v * 128.0) as i32);
        if s > 0.93 - 0.2 * cluster && cluster > 0.0 {
            c = mix(c, [0.08, 0.08, 0.07], 0.85 * cluster);
        }
        c
    });
    let ring = group();
    put_world(&ring, "Real__teethingRing", &phong!({ map: &tex, specular: 0x3a3a3a, shininess: 46 }), 0.05);
    ring.position().set(px, 0, pz);
    ring.rotation().y = turn;
    g.world.add(&ring);
}

/// How a moulded or painted thing has worn on a floor for years
#[derive(Clone, Copy)]
struct Wear {
    /// its colour, and what shows where it is rubbed through (the plastic
    /// under paint, or plastic whitened by stress)
    base: [f64; 3],
    alt: [f64; 3],
    /// how much of it is rubbed through, grimed, scuffed, mottled (faded
    /// unevenly); its sculpted scales (cells across a tile; 0 none)
    rub: f64,
    grime: f64,
    scuff: f64,
    mottle: f64,
    scales: i32,
}

/// The worn surface at (u, v) of a repeating tile
fn wear_at(seed: u32, u: f64, v: f64, w: &Wear) -> [f64; 3] {
    let m = fbmp(seed, u, v, 3, 4);
    let mut c = mul(w.base, 1.0 + w.mottle * (m - 0.5) * 2.4);
    if w.scales > 0 {
        let e = cell_edge(seed ^ 0x51ab, u, v, w.scales);
        c = mul(c, 1.0 - 0.2 * (1.0 - sstep(0.0, 0.14, e)));
    }
    if w.rub > 0.0 {
        // rubbed through in small patches: `rub` is about the share of the
        // surface (this fractal noise sits near 0.49, its spread about 0.15)
        let r = fbmp(seed ^ 0x2b3c, u, v, 12, 3);
        let t = 0.8 - 0.9 * w.rub;
        c = mix(c, w.alt, sstep(t, t + 0.025, r));
    }
    // grime settled in soft patches
    let gm = fbmp(seed ^ 0x77e1, u, v, 3, 4);
    c = mix(c, [0.15, 0.13, 0.10], w.grime * 0.7 * sstep(0.5, 0.8, gm));
    if w.scuff > 0.0 {
        // fine scratches in patches: thin lines along a stretched noise
        let patch = sstep(0.5, 0.7, fbmp(seed ^ 0x9137, u, v, 3, 2));
        let l1 = (vnp(seed ^ 0x4421, u * 40.0, v * 5.0, 40, 5) - 0.5).abs();
        let l2 = (vnp(seed ^ 0x1d77, u * 6.0, v * 34.0, 6, 34) - 0.5).abs();
        let line = (1.0 - sstep(0.0, 0.025, l1)).max(1.0 - sstep(0.0, 0.025, l2));
        c = mix(c, mix(w.base, [0.86, 0.84, 0.8], 0.45), w.scuff * patch * line * 0.8);
    }
    // dust in the grain, and grit
    let d = vnp(seed ^ 0xd057, u * 90.0, v * 90.0, 90, 90);
    if d > 0.8 {
        c = mix(c, [0.55, 0.52, 0.47], (d - 0.8) * 2.2);
    }
    c
}
