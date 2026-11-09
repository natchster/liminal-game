//! The contact shadow under a small thing on the floor (`docs/PROPS.md`,
//! M8).
//!
//! Where a thing meets the floor, the floor under and round it sees less of
//! the room: a doll's shoes, a ball, a rake lying there are darker at their
//! foot. The light map is half a metre a texel, far too coarse to hold that,
//! and a lamp's shadow would tie light to a thing (`STYLE.md`: light belongs
//! to places). So each small thing standing or lying on the floor gets a
//! soft dark shape laid on the floor round its footprint: the darkest under
//! it, gone a few centimetres out (more out from a taller thing; darker
//! under a taller thing, faint under a flat one). A thing's footprint is
//! the outline (the convex hull) of the low part of its parts that touch the
//! floor, those parts merged where they overlap: a ball's is round, a
//! shoe's its sole, a spill of limbs lying apart gets one each.
//!
//! The shapes are one mesh a 10 x 10 tile chunk (as the instancing
//! chunks), drawn unlit in black with their alpha in their vertex colours:
//! no texture, one draw a chunk in view. Only what never moves gets one
//! (a themed placer's scenery, `static_prop`, and the core props of kinds
//! that stay where they lie); nothing on a stream: no draw.
use crate::prelude::*;

/// A small thing on the floor: at most this tall (m) and this wide
const MAX_H: f64 = 0.6;
const MAX_W: f64 = 0.9;
/// ... at least this tall (flat marks, papers and puddles cast nothing)
const MIN_H: f64 = 0.012;
/// ... and its foot this near its floor
const MAX_LIFT: f64 = 0.03;
/// A part touches the floor when its foot is within this of it
const PART_LIFT: f64 = 0.04;
/// How far the shadow's edge lies over the floor (clear of it, no flicker)
const OVER: f64 = 0.003;
/// The core props that stay where they lie (the rest move, float or burn)
const CORE_KINDS: [&str; 10] = [
    "baby_doll",
    "rubber_duck",
    "traffic_cone",
    "trash_can",
    "wet_floor_sign",
    "book_stack",
    "suitcase",
    "pile",
    "potted_plant",
    "floor_plant",
];

/// One shadow: the outline of the thing's foot on the floor (convex, round
/// the way), the floor's height there, how far the shadow reaches out past
/// it and how dark it is under the thing
#[derive(Clone, Debug)]
pub struct Blob {
    hull: Vec<(f64, f64)>,
    y: f64,
    reach: f64,
    dark: f64,
}

/// The convex hull of points on the floor, anticlockwise seen from above
/// (x right, z toward the viewer: turning from +x to +z), no point repeated
fn hull_of(mut pts: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    pts.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    pts.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6);
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: (f64, f64), a: (f64, f64), b: (f64, f64)| (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0);
    let mut h: Vec<(f64, f64)> = vec![];
    for pass in 0..2 {
        let start = h.len();
        let it: Box<dyn Iterator<Item = &(f64, f64)>> = if pass == 0 { Box::new(pts.iter()) } else { Box::new(pts.iter().rev()) };
        for &p in it {
            while h.len() >= start + 2 && cross(h[h.len() - 2], h[h.len() - 1], p) <= 0.0 {
                h.pop();
            }
            h.push(p);
        }
        h.pop();
    }
    h
}

impl Game {
    /// The contact shadows of the props from the world's child `start` on,
    /// once they are settled (`build_world`, before the instancing takes
    /// the repeated ones out of the world); `add_contact_shadows` lays them.
    pub fn contact_blobs(&self, start: usize) -> Vec<Blob> {
        if self.has_flag("nocontact") {
            return vec![];
        }
        let keep: Vec<Obj> = self.almonds.iter().chain(self.battery_items.iter()).map(|p| p.g.clone()).collect();
        let mut out = vec![];
        for top in self.world.children().into_iter().skip(start) {
            {
                let u = top.user();
                let core_ok = u.s.get("kind").is_some_and(|k| CORE_KINDS.contains(&k.as_str()));
                if u.gameplay || u.floats || u.arch || !(u.static_prop || core_ok) {
                    continue;
                }
            }
            if top.is_light() || keep.iter().any(|k| k.ptr_eq(&top)) {
                continue;
            }
            let mut bb = Box3::empty();
            bb.set_from_object(&top);
            if bb.is_empty() {
                continue;
            }
            let size = bb.get_size();
            let floor = self.floor_y_at((bb.min.x + bb.max.x) / 2.0, (bb.min.z + bb.max.z) / 2.0);
            // (a long thing lying low, a rake or a pipe, as well)
            let wide = size.x.max(size.z);
            if size.y > MAX_H || size.y < MIN_H || wide > if size.y < 0.2 { 2.0 } else { MAX_W } || (bb.min.y - floor).abs() > MAX_LIFT {
                continue;
            }
            // nothing that burns or glows: a flame's light is its own
            let mut lit = false;
            top.traverse(&mut |c: &Obj| lit |= c.is_light());
            if lit {
                continue;
            }
            // its parts that touch the floor: the low half of each, seen from above
            let mut parts: Vec<(Vec<(f64, f64)>, [f64; 4], f64)> = vec![];
            top.traverse(&mut |c: &Obj| {
                if !c.is_mesh() || !c.visible() {
                    return;
                }
                let Some(geo) = c.try_geo() else { return };
                // see-through films and decals (glass, a stain under it) cast nothing
                if c.mats().iter().all(|m| {
                    let m = m.r();
                    !m.depth_write || m.opacity < 0.5
                }) {
                    return;
                }
                let mw = c.matrix_world();
                let gg = geo.g();
                let n = gg.position.len() / 3;
                if n == 0 {
                    return;
                }
                let mut world = Vec::with_capacity(n.min(160));
                let step = n.div_ceil(160);
                let (mut foot, mut head) = (f64::MAX, f64::MIN);
                for i in (0..n).step_by(step) {
                    let mut p = Vec3::new(gg.position[i * 3] as f64, gg.position[i * 3 + 1] as f64, gg.position[i * 3 + 2] as f64);
                    p.apply_matrix4(&mw);
                    foot = foot.min(p.y);
                    head = head.max(p.y);
                    world.push(p);
                }
                if foot - floor > PART_LIFT {
                    return;
                }
                let low = foot + (head - foot) * 0.45 + 0.004;
                let pts: Vec<(f64, f64)> = world.iter().filter(|p| p.y <= low).map(|p| (p.x, p.z)).collect();
                let mut ext = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
                for &(x, z) in &pts {
                    ext = [ext[0].min(x), ext[1].min(z), ext[2].max(x), ext[3].max(z)];
                }
                parts.push((pts, ext, head));
            });
            // parts whose footprints overlap make one shadow
            let n = parts.len();
            let mut root: Vec<usize> = (0..n).collect();
            fn find(r: &mut [usize], i: usize) -> usize {
                let mut i = i;
                while r[i] != i {
                    r[i] = r[r[i]];
                    i = r[i];
                }
                i
            }
            for i in 0..n {
                for j in i + 1..n {
                    let (a, b) = (parts[i].1, parts[j].1);
                    if a[0] <= b[2] + 0.01 && b[0] <= a[2] + 0.01 && a[1] <= b[3] + 0.01 && b[1] <= a[3] + 0.01 {
                        let (ri, rj) = (find(&mut root, i), find(&mut root, j));
                        root[ri.max(rj)] = ri.min(rj);
                    }
                }
            }
            for r in 0..n {
                if find(&mut root, r) != r {
                    continue;
                }
                let mut pts = vec![];
                let mut h = 0.0f64;
                for i in 0..n {
                    if find(&mut root, i) == r {
                        pts.extend_from_slice(&parts[i].0);
                        h = h.max(parts[i].2 - floor);
                    }
                }
                let hull = hull_of(pts);
                if hull.len() < 3 {
                    continue;
                }
                // out past the footprint: a few centimetres, more from a taller thing
                let reach = (0.30 * h).min(0.05) + 0.008;
                // the same floor under it and round it (not over a step or an edge)
                let k = hull.len() as f64;
                let (cx, cz) = (hull.iter().map(|p| p.0).sum::<f64>() / k, hull.iter().map(|p| p.1).sum::<f64>() / k);
                let y = self.floor_y_at(cx, cz);
                if hull
                    .iter()
                    .any(|&(x, z)| (self.floor_y_at(x + (x - cx).signum() * reach, z + (z - cz).signum() * reach) - y).abs() > 0.02)
                {
                    continue;
                }
                // darker under a taller thing, faint under a flat one
                let dark = 0.16 + 0.46 * (h / 0.25).clamp(0.0, 1.0);
                out.push(Blob { hull, y, reach, dark });
            }
        }
        out
    }

    /// Lay the contact shadows: one mesh a chunk, added to the world after
    /// everything placed in it (so no index of a placed thing moves)
    pub fn add_contact_shadows(&mut self, blobs: &[Blob]) {
        if blobs.is_empty() {
            return;
        }
        let (w, h) = (self.w as f64, self.h as f64);
        let chunk_of = |x: f64, z: f64| {
            ((x / CELL + w / 2.0).floor() / 10.0).floor() as i64 * 64 + ((z / CELL + h / 2.0).floor() / 10.0).floor() as i64
        };
        let mut chunks: std::collections::BTreeMap<i64, Vec<&Blob>> = Default::default();
        for b in blobs {
            chunks.entry(chunk_of(b.hull[0].0, b.hull[0].1)).or_default().push(b);
        }
        // rings round the footprint: (how far out, as a share of the reach;
        // how dark), the darkest under the thing, gone at the edge
        const RINGS: [(f64, f64); 3] = [(0.0, 0.85), (0.4, 0.42), (1.0, 0.0)];
        let mat = basic_mat!({ color: 0x000000, vertexColors: true, transparent: true, depthWrite: false });
        mat.m().name = "contact_shadow".into();
        for (_, list) in chunks {
            let (mut pos, mut col, mut idx) = (vec![], vec![], vec![]);
            for b in list {
                let base = (pos.len() / 3) as u32;
                let y = (b.y + OVER) as f32;
                let k = b.hull.len();
                let (cx, cz) = (b.hull.iter().map(|p| p.0).sum::<f64>() / k as f64, b.hull.iter().map(|p| p.1).sum::<f64>() / k as f64);
                pos.extend_from_slice(&[cx as f32, y, cz as f32]);
                col.extend_from_slice(&[0.0, 0.0, 0.0, b.dark as f32]);
                // each corner's way out: between its two sides' outward normals
                let outward: Vec<(f64, f64)> = (0..k)
                    .map(|i| {
                        let (p0, p1, p2) = (b.hull[(i + k - 1) % k], b.hull[i], b.hull[(i + 1) % k]);
                        let n0 = ((p1.1 - p0.1), -(p1.0 - p0.0));
                        let n1 = ((p2.1 - p1.1), -(p2.0 - p1.0));
                        let l0 = n0.0.hypot(n0.1).max(1e-9);
                        let l1 = n1.0.hypot(n1.1).max(1e-9);
                        let (nx, nz) = (n0.0 / l0 + n1.0 / l1, n0.1 / l0 + n1.1 / l1);
                        let l = nx.hypot(nz).max(1e-9);
                        (nx / l, nz / l)
                    })
                    .collect();
                for &(r, d) in &RINGS {
                    for i in 0..k {
                        let (x, z) = (b.hull[i].0 + outward[i].0 * b.reach * r, b.hull[i].1 + outward[i].1 * b.reach * r);
                        pos.extend_from_slice(&[x as f32, y, z as f32]);
                        col.extend_from_slice(&[0.0, 0.0, 0.0, (b.dark * d) as f32]);
                    }
                }
                // the middle's fan, then each ring to the next, faced up
                let k = k as u32;
                for i in 0..k {
                    let i1 = (i + 1) % k;
                    idx.extend_from_slice(&[base, base + 1 + i1, base + 1 + i]);
                }
                for ring in 0..RINGS.len() as u32 - 1 {
                    let (r0, r1) = (base + 1 + ring * k, base + 1 + (ring + 1) * k);
                    for i in 0..k {
                        let i1 = (i + 1) % k;
                        idx.extend_from_slice(&[r0 + i, r0 + i1, r1 + i1, r0 + i, r1 + i1, r1 + i]);
                    }
                }
            }
            let n = pos.len() / 3;
            let mut g = Geometry {
                position: pos,
                normal: [0.0f32, 1.0, 0.0].repeat(n),
                color: col,
                color_size: 4,
                index: Some(idx),
                ..Default::default()
            };
            g.compute_bounding_box();
            g.compute_bounding_sphere();
            let m = mesh(Geo::from_geometry(g), &mat);
            m.set_name("contact_shadows");
            self.world.add(&m);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_footprint_is_the_outline_of_what_lies_there() {
        // a square with a point inside it and one on a side
        let pts = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.4, 0.5), (0.5, 0.0)];
        let h = hull_of(pts);
        assert_eq!(h.len(), 4, "{h:?}");
        // anticlockwise from above: each turn the same way
        let k = h.len();
        for i in 0..k {
            let (o, a, b) = (h[i], h[(i + 1) % k], h[(i + 2) % k]);
            assert!((a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0) > 0.0);
        }
    }
}
