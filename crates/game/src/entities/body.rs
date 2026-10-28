//! A creature's body in the scene: the skeleton of `assets::bodies` as bones,
//! its skinned meshes bound to them and its rigid meshes on them, and the
//! posing helpers every creature's animation uses.
//!
//! Every bone's rest frame is axis-aligned at its head (the body is bound in
//! the pose it was modelled in), so a pose is a rotation per bone relative
//! to rest. The helpers think in the body's frame (+Y up, +Z ahead, its
//! left at +X): [`BodyRig::aim`] turns a bone so that its rest direction (head
//! to tail) points along a direction given in the body's frame, whatever
//! its parents are doing; [`two_bone`] solves a limb toward a target with a
//! pole for the middle joint.
use crate::assets::bodies::{body, body_geometry, BodyDef};
use crate::prelude::*;
use std::collections::HashMap;

/// A body built into the scene.
#[derive(Clone, Debug)]
pub struct BodyRig {
    /// The group that carries the body's position and heading
    pub root: Obj,
    pub bones: Vec<Obj>,
    pub def: &'static BodyDef,
    /// Each bone's rest direction (unit, body frame) and length
    pub rest_dir: Vec<Vec3>,
    pub rest_len: Vec<f64>,
    /// Skinned meshes (their bind follows the root, `sync`)
    pub skins: Vec<Obj>,
    /// Every mesh by its name
    pub meshes: HashMap<String, Obj>,
}

fn v(a: [f64; 3]) -> Vec3 {
    v3(a[0], a[1], a[2])
}

/// The rotation taking unit `a` onto unit `b`.
pub fn from_to(a: &Vec3, b: &Vec3) -> Quat {
    let mut q = Quat::IDENTITY;
    q.set_from_unit_vectors(a, b);
    q
}

pub fn qmul(a: &Quat, b: &Quat) -> Quat {
    let mut q = *a;
    q.multiply(b);
    q
}

pub fn qinv(a: &Quat) -> Quat {
    let mut q = *a;
    q.invert();
    q
}

pub fn qrot(q: &Quat, p: &Vec3) -> Vec3 {
    let mut r = *p;
    r.apply_quaternion(q);
    r
}

pub fn axis_angle(axis: &Vec3, a: f64) -> Quat {
    Quat::from_axis_angle(axis, a)
}

/// Build body `name` with materials from `mat(slot)` (`skin`, `eye`, ...).
pub fn build_body(name: &str, mat: &dyn Fn(&str) -> Mat) -> Option<BodyRig> {
    let def = body(name)?;
    let root = group();
    root.set_name(name);
    let mut bones: Vec<Obj> = Vec::with_capacity(def.bones.len());
    let (mut rest_dir, mut rest_len) = (vec![], vec![]);
    for b in &def.bones {
        let o = bone();
        o.set_name(&b.name);
        let (head, tail) = (v(b.head), v(b.tail));
        match b.parent {
            Some(p) => {
                let ph = v(def.bones[p].head);
                o.set_pos(head.x - ph.x, head.y - ph.y, head.z - ph.z);
                bones[p].add(&o);
            }
            None => {
                o.set_pos(head.x, head.y, head.z);
                root.add(&o);
            }
        }
        let mut d = v3(tail.x - head.x, tail.y - head.y, tail.z - head.z);
        let l = d.length();
        if l > 1e-6 {
            d.multiply_scalar(1.0 / l);
        } else {
            d = v3(0.0, 1.0, 0.0);
        }
        rest_dir.push(d);
        rest_len.push(l);
        bones.push(o);
    }
    root.update_matrix_world(true);
    let mut skins = vec![];
    let mut meshes = HashMap::new();
    for m in &def.meshes {
        let geo = body_geometry(name, &m.name)?;
        let o = match m.bone {
            Some(b) => {
                let o = mesh(&geo, mat(&m.material));
                bones[b].add(&o);
                o
            }
            None => {
                let sm = skinned_mesh(&geo, mat(&m.material));
                root.add(&sm);
                root.update_matrix_world(true);
                bind_skeleton(&sm, bones.clone());
                skins.push(sm.clone());
                sm
            }
        };
        o.set_name(&m.name);
        meshes.insert(m.name.clone(), o);
    }
    Some(BodyRig { root, bones, def, rest_dir, rest_len, skins, meshes })
}

impl BodyRig {
    pub fn ix(&self, name: &str) -> Option<usize> {
        self.def.bone(name)
    }

    /// Every bone back to rest.
    pub fn rest(&self) {
        for b in &self.bones {
            b.set_quat(&Quat::IDENTITY);
        }
    }

    /// Bone `i`'s rotation in the body's frame (its parents' poses composed).
    pub fn body_rot(&self, i: usize) -> Quat {
        let mut chain = vec![];
        let mut k = Some(i);
        while let Some(j) = k {
            chain.push(j);
            k = self.def.bones[j].parent;
        }
        let mut q = Quat::IDENTITY;
        for &j in chain.iter().rev() {
            q = qmul(&q, &self.bones[j].quaternion());
        }
        q
    }

    /// Bone `i`'s head in the body's frame, as posed.
    pub fn head(&self, i: usize) -> Vec3 {
        let mut chain = vec![];
        let mut k = Some(i);
        while let Some(j) = k {
            chain.push(j);
            k = self.def.bones[j].parent;
        }
        // walk down from the root: each head is its parent's head plus the
        // parent's rotation of the rest offset
        let mut pos = Vec3::default();
        let mut rot = Quat::IDENTITY;
        let mut prev: Option<usize> = None;
        for &j in chain.iter().rev() {
            let off = match prev {
                Some(p) => {
                    let (h, ph) = (v(self.def.bones[j].head), v(self.def.bones[p].head));
                    v3(h.x - ph.x, h.y - ph.y, h.z - ph.z)
                }
                None => {
                    let p = self.bones[j].position();
                    v3(p.x, p.y, p.z)
                }
            };
            let o = qrot(&rot, &off);
            pos = v3(pos.x + o.x, pos.y + o.y, pos.z + o.z);
            rot = qmul(&rot, &self.bones[j].quaternion());
            prev = Some(j);
        }
        pos
    }

    /// Bone `i`'s tail in the body's frame, as posed.
    pub fn tail(&self, i: usize) -> Vec3 {
        let h = self.head(i);
        let d = qrot(&self.body_rot(i), &self.rest_dir[i]);
        let l = self.rest_len[i];
        v3(h.x + d.x * l, h.y + d.y * l, h.z + d.z * l)
    }

    /// Set bone `i`'s local rotation so that it points along `dir` (body
    /// frame, need not be unit), turned `twist` radians about itself.
    pub fn aim(&self, i: usize, dir: Vec3, twist: f64) {
        let mut d = dir;
        let l = d.length();
        if l < 1e-9 {
            return;
        }
        d.multiply_scalar(1.0 / l);
        let parent = self.def.bones[i].parent.map(|p| self.body_rot(p)).unwrap_or(Quat::IDENTITY);
        let world = qmul(&from_to(&self.rest_dir[i], &d), &axis_angle(&self.rest_dir[i], twist));
        self.bones[i].set_quat(&qmul(&qinv(&parent), &world));
    }

    /// Turn bone `i` by `q` (body frame) on top of its parents, as if it
    /// were at rest relative to them: for heads, jaws, a spine's bend.
    pub fn turn(&self, i: usize, q: &Quat) {
        self.bones[i].set_quat(q);
    }

    /// The rigs move with their root: keep each skin's bind in step.
    pub fn sync(&self) {
        for s in &self.skins {
            crate::entities::sync_skin_bind(s);
        }
    }
}

/// A limb of two bones from `root` toward `target` (body frame), lengths
/// `a` and `b`, the middle joint bending toward `pole` → the directions of
/// the two bones. The target is pulled in when out of reach.
pub fn two_bone(root: Vec3, target: Vec3, pole: Vec3, a: f64, b: f64) -> (Vec3, Vec3) {
    let mut t = v3(target.x - root.x, target.y - root.y, target.z - root.z);
    let mut d = t.length();
    let reach = (a + b) * 0.999;
    let near = (a - b).abs() + 1e-4;
    if d > reach {
        t.multiply_scalar(reach / d);
        d = reach;
    } else if d < near {
        if d < 1e-9 {
            t = v3(0.0, -near, 0.0);
        } else {
            t.multiply_scalar(near / d);
        }
        d = near;
    }
    let mut u = t;
    u.multiply_scalar(1.0 / d);
    // the bend plane: the pole's part across the limb
    let mut p = v3(pole.x - root.x, pole.y - root.y, pole.z - root.z);
    let along = p.dot(&u);
    p = v3(p.x - u.x * along, p.y - u.y * along, p.z - u.z * along);
    if p.length() < 1e-9 {
        p = v3(0.0, 0.0, 1.0);
        let al = p.dot(&u);
        p = v3(p.x - u.x * al, p.y - u.y * al, p.z - u.z * al);
    }
    p.normalize();
    // the middle joint by the law of cosines
    let x = (a * a - b * b + d * d) / (2.0 * d);
    let h = (a * a - x * x).max(0.0).sqrt();
    let mid = v3(u.x * x + p.x * h, u.y * x + p.y * h, u.z * x + p.z * h);
    let lower = v3(t.x - mid.x, t.y - mid.y, t.z - mid.z);
    (mid, lower)
}
