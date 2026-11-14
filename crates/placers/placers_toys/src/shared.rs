//! What several toys placers share.
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::cell_edge;
use placers_kit::noise::fbm;
use placers_kit::noise::fbmp;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::noise::vnp;
use placers_kit::paint::fill;
use placers_kit::paint::mul;
use placers_kit::paint::paint_tile;

/// Part `name` of family `toys_left_on_the_floor` as modelled, added to `grp` in `mat`
pub(crate) fn put(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

/// How a moulded or painted thing has worn on a floor for years
#[derive(Clone, Copy)]
pub(crate) struct Wear {
    /// its colour, and what shows where it is rubbed through (the plastic
    /// under paint, or plastic whitened by stress)
    pub(crate) base: [f64; 3],
    pub(crate) alt: [f64; 3],
    /// how much of it is rubbed through, grimed, scuffed, mottled (faded
    /// unevenly); its sculpted scales (cells across a tile; 0 none)
    pub(crate) rub: f64,
    pub(crate) grime: f64,
    pub(crate) scuff: f64,
    pub(crate) mottle: f64,
    pub(crate) scales: i32,
}

/// The worn surface at (u, v) of a repeating tile
pub(crate) fn wear_at(seed: u32, u: f64, v: f64, w: &Wear) -> [f64; 3] {
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

/// A worn surface's tile
pub(crate) fn wear_tex(seed: u32, px: u32, w: Wear) -> Tex {
    paint_tile(px, move |u, v| wear_at(seed, u, v, &w))
}

/// Soft vinyl gone yellow (`base`): faintly uneven, grime gathered in
/// soft patches and specks, scuffed paler where it rubbed on the floor, a
/// child's ballpoint here and there
pub(crate) fn vinyl_tex(salt: u32, base: u32) -> Tex {
    let b = [f64::from((base >> 16) & 255) / 255.0, f64::from((base >> 8) & 255) / 255.0, f64::from(base & 255) / 255.0];
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x51d;
        fill(ctx, w, h, |u, v| {
            let mut c = b;
            let n = fbm(seed, u * 5.0, v * 5.0, 4);
            c = mix(c, [b[0] * 0.84, b[1] * 0.78, b[2] * 0.70], sstep(0.45, 0.8, n) * 0.45);
            let pale = fbm(seed ^ 3, u * 7.0, v * 7.0, 3);
            c = mix(c, [0.88, 0.78, 0.70], sstep(0.6, 0.85, pale) * 0.2);
            // grime: blotches and the dirt of years in its pores
            let grime = fbm(seed ^ 5, u * 3.0, v * 3.0, 3);
            c = mix(c, [0.40, 0.33, 0.27], sstep(0.55, 0.8, grime) * 0.3);
            let speck = vn(seed ^ 7, u * 70.0, v * 70.0);
            if speck > 0.88 {
                c = mix(c, [0.36, 0.30, 0.24], (speck - 0.88) * 4.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
        // a ballpoint's strokes, faded
        if hash(salt, 1, 2) < 0.5 {
            ctx.set_stroke_style("rgba(60,72,120,0.4)");
            ctx.set_line_width(1.0);
            let (x0, y0) = (w * (0.2 + 0.6 * hash(salt, 3, 4)), h * (0.2 + 0.6 * hash(salt, 5, 6)));
            ctx.begin_path();
            ctx.move_to(x0, y0);
            for i in 1..6 {
                let fi = f64::from(i);
                ctx.line_to(x0 + fi * 3.0 + rnd() * 2.0, y0 + if i % 2 == 0 { 0.0 } else { 3.0 + rnd() * 2.0 });
            }
            ctx.stroke();
        }
    })
}

/// A custom geometry from a flat position list (f64 → f32) and an index.
pub(crate) fn tri_geo(verts: &[f64], idx: Vec<u32>) -> Geo {
    let geo = Geo::new();
    geo.set_position(verts.iter().map(|&v| v as f32).collect());
    geo.set_index(idx);
    geo.compute_vertex_normals();
    geo
}
