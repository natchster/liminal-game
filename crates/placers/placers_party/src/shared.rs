//! What several party placers share.
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;

/// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
pub(crate) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = prop_part_as_modelled(name)?;
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}

/// The same, its texture coordinates folded so its sides and its tops
/// each get their own place on one map: a triangle that mostly looks up
/// or down takes the disc the game lays on a cylinder's caps (from
/// above), folded into the upper left quarter; any other the round and
/// up of its sides, folded into the lower half. A corner the two kinds
/// share is doubled, so no triangle runs across the map from one to
/// the other.
pub(crate) fn put_split(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = prop_part_as_modelled(name)?.duplicate();
    {
        let mut gg = geo.g();
        let idx = gg.index.clone()?;
        let n = gg.position.len() / 3;
        let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
        for i in 0..n {
            for k in 0..3 {
                let p = f64::from(gg.position[i * 3 + k]);
                lo[k] = lo[k].min(p);
                hi[k] = hi[k].max(p);
            }
        }
        let ex = [0, 1, 2].map(|k| if hi[k] - lo[k] > 1e-9 { hi[k] - lo[k] } else { 1.0 });
        let ny = |i: usize| gg.normal[i * 3 + 1];
        let cap = |i: usize| ny(i).abs() > 0.75;
        let (mut pos, mut nrm) = (gg.position.clone(), gg.normal.clone());
        let mut uv: Vec<f32> = Vec::with_capacity(gg.uv.len());
        for i in 0..n {
            let (u, v) = (gg.uv[i * 2], gg.uv[i * 2 + 1]);
            if cap(i) {
                uv.extend_from_slice(&[u * 0.5, 0.5 + v * 0.5]);
            } else {
                uv.extend_from_slice(&[u, v * 0.5]);
            }
        }
        let mut out = Vec::with_capacity(idx.len());
        let mut dup: std::collections::HashMap<(u32, u8), u32> = std::collections::HashMap::new();
        for t in idx.chunks(3) {
            let tri_cap = t.iter().map(|&i| ny(i as usize).abs()).sum::<f32>() / 3.0 > 0.6;
            let up = t.iter().map(|&i| ny(i as usize)).sum::<f32>() >= 0.0;
            let side_u = t.iter().map(|&i| i as usize).find(|&i| !cap(i)).map(|i| uv[i * 2]);
            for &i in t {
                let iu = i as usize;
                if cap(iu) == tri_cap {
                    out.push(i);
                    continue;
                }
                let f = [0, 1, 2].map(|k| (f64::from(gg.position[iu * 3 + k]) - lo[k]) / ex[k]);
                let (u, v, key) = if tri_cap {
                    (f[2] * 0.5, 0.5 + if up { f[0] } else { 1.0 - f[0] } * 0.5, u8::from(up))
                } else {
                    let mut u = ((f[0] * 2.0 - 1.0).datan2(f[2] * 2.0 - 1.0) / (2.0 * PI) + 1.0) % 1.0;
                    let mut key = 2;
                    if let Some(m) = side_u.map(f64::from) {
                        if u - m > 0.5 {
                            u -= 1.0;
                            key = 3;
                        } else if m - u > 0.5 {
                            u += 1.0;
                            key = 4;
                        }
                    }
                    (u, f[1] * 0.5, key)
                };
                let d = *dup.entry((i, key)).or_insert_with(|| {
                    pos.extend_from_slice(&[gg.position[iu * 3], gg.position[iu * 3 + 1], gg.position[iu * 3 + 2]]);
                    nrm.extend_from_slice(&[gg.normal[iu * 3], gg.normal[iu * 3 + 1], gg.normal[iu * 3 + 2]]);
                    uv.extend_from_slice(&[u as f32, v as f32]);
                    (pos.len() / 3 - 1) as u32
                });
                out.push(d);
            }
        }
        gg.position = pos;
        gg.normal = nrm;
        gg.uv = uv;
        gg.index = Some(out);
        gg.touch();
    }
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}

/// Tissue paper after years: each old colour faded
pub(crate) const TISSUE: [u32; 7] = [0xb57884, 0x7383a6, 0xcdbd84, 0x86a07e, 0x96809c, 0xc9a2b3, 0xcfc8b8];

/// The same with its geometry (a copy) changed by `edit` first (its
/// texture coordinates)
pub(crate) fn put_with(grp: &Obj, name: &str, mat: &Mat, edit: impl FnOnce(&mut Geometry)) -> Option<Obj> {
    let geo = game::assets::props::prop_part_as_modelled(name)?.duplicate();
    {
        let mut gg = geo.g();
        edit(&mut gg);
        gg.touch();
    }
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}
