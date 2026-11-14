//! Garden things (bird baths, gnomes, soil bags, seed catalogues): a part put with its own texture coordinates.
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;

// ---------------------------------------------------------------- the kit
/// Adds the Blender part `name` as modelled to `grp` (nothing when there is
/// no such part)
pub fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = prop_part_as_modelled(name)?;
    let m = mesh(geo, mat);
    grp.add(&m);
    Some(m)
}

/// Adds part `name` with its texture coordinates made by `uv` (a copy)
pub fn put_uv(grp: &Obj, name: &str, mat: &Mat, uv: impl Fn(f64, f64, f64, [f64; 3]) -> (f64, f64)) -> Option<Obj> {
    let geo = prop_part_as_modelled(name)?.duplicate();
    {
        let mut gg = geo.g();
        let n = gg.position.len() / 3;
        let mut out = Vec::with_capacity(n * 2);
        for i in 0..n {
            let p = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 1]), f64::from(gg.position[i * 3 + 2]));
            let nn = [f64::from(gg.normal[i * 3]), f64::from(gg.normal[i * 3 + 1]), f64::from(gg.normal[i * 3 + 2])];
            let (u, v) = uv(p.0, p.1, p.2, nn);
            out.push(u as f32);
            out.push(v as f32);
        }
        gg.uv = out;
        gg.touch();
    }
    let m = mesh(geo, mat);
    grp.add(&m);
    Some(m)
}

/// From above: u along x over [x0, x1], v = 1 at z0 down to 0 at z1
pub fn top(x0: f64, x1: f64, z0: f64, z1: f64) -> impl Fn(f64, f64, f64, [f64; 3]) -> (f64, f64) {
    move |x, _y, z, _n| ((x - x0) / (x1 - x0), 1.0 - (z - z0) / (z1 - z0))
}
