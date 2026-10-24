//! three.js-compatible math (r160 semantics), all in f64 like JS numbers.
#![allow(non_snake_case, clippy::too_many_arguments)]


/// Anything numeric that a JS call site might pass (literals default to i32).
pub trait Num: Copy {
    fn f(self) -> f64;
}
macro_rules! num_impl { ($($t:ty),*) => { $(impl Num for $t { #[inline] fn f(self) -> f64 { self as f64 } })* } }
num_impl!(f64, f32, i8, i16, i32, i64, u8, u16, u32, u64, usize, isize);
impl Num for bool { #[inline] fn f(self) -> f64 { if self { 1.0 } else { 0.0 } } }

pub const PI: f64 = std::f64::consts::PI;
pub const TAU: f64 = std::f64::consts::TAU;

#[inline] pub fn clamp(v: f64, a: f64, b: f64) -> f64 { if v < a { a } else if v > b { b } else { v } }
#[inline] pub fn lerp(a: f64, b: f64, t: f64) -> f64 { a + (b - a) * t }
#[inline] pub fn deg_to_rad(d: f64) -> f64 { d * PI / 180.0 }
#[inline] pub fn rad_to_deg(r: f64) -> f64 { r * 180.0 / PI }
#[inline] pub fn smoothstep(x: f64, min: f64, max: f64) -> f64 {
    if x <= min { return 0.0; }
    if x >= max { return 1.0; }
    let x = (x - min) / (max - min);
    x * x * (3.0 - 2.0 * x)
}
/// JS `Math.round` (half rounds toward +∞).
#[inline] pub fn js_round(x: f64) -> f64 { (x + 0.5).floor() }
/// JS `Math.sign`.
#[inline] pub fn js_sign(x: f64) -> f64 { if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { x } }
/// JS `x % y` (sign of dividend), same as Rust's `%` on f64.
#[inline] pub fn js_mod(x: f64, y: f64) -> f64 { x % y }
/// three's MathUtils.euclideanModulo
#[inline] pub fn euclidean_modulo(n: f64, m: f64) -> f64 { ((n % m) + m) % m }
/// JS ToInt32 (`x | 0`).
#[inline] pub fn to_i32(x: f64) -> i32 {
    if !x.is_finite() { return 0; }
    let t = x.trunc();
    let m = t.rem_euclid(4294967296.0);
    (m as u64 as u32) as i32
}
/// JS ToUint32 (`x >>> 0`).
#[inline] pub fn to_u32(x: f64) -> u32 { to_i32(x) as u32 }

// ---------------------------------------------------------------- Vec2
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 { pub x: f64, pub y: f64 }
impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self { Self { x, y } }
    pub fn set(&mut self, x: impl Num, y: impl Num) -> &mut Self { self.x = x.f(); self.y = y.f(); self }
    pub fn set_scalar(&mut self, s: impl Num) -> &mut Self { let s = s.f(); self.x = s; self.y = s; self }
    pub fn copy(&mut self, o: &Vec2) -> &mut Self { *self = *o; self }
    pub fn length(&self) -> f64 { (self.x * self.x + self.y * self.y).sqrt() }
    pub fn add(&mut self, o: &Vec2) -> &mut Self { self.x += o.x; self.y += o.y; self }
    pub fn sub(&mut self, o: &Vec2) -> &mut Self { self.x -= o.x; self.y -= o.y; self }
    pub fn multiply_scalar(&mut self, s: impl Num) -> &mut Self { let s = s.f(); self.x *= s; self.y *= s; self }
    pub fn normalize(&mut self) -> &mut Self { let l = self.length(); let l = if l == 0.0 { 1.0 } else { l }; self.x /= l; self.y /= l; self }
    pub fn distance_to(&self, o: &Vec2) -> f64 { ((self.x - o.x).powi(2) + (self.y - o.y).powi(2)).sqrt() }
    pub fn dot(&self, o: &Vec2) -> f64 { self.x * o.x + self.y * o.y }
}
pub fn v2(x: impl Num, y: impl Num) -> Vec2 { Vec2::new(x.f(), y.f()) }

// ---------------------------------------------------------------- Vec3
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 { pub x: f64, pub y: f64, pub z: f64 }
pub fn v3(x: impl Num, y: impl Num, z: impl Num) -> Vec3 { Vec3::new(x.f(), y.f(), z.f()) }
impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } }
    pub fn set(&mut self, x: impl Num, y: impl Num, z: impl Num) -> &mut Self { self.x = x.f(); self.y = y.f(); self.z = z.f(); self }
    pub fn set_scalar(&mut self, s: impl Num) -> &mut Self { let s = s.f(); self.x = s; self.y = s; self.z = s; self }
    pub fn set_x(&mut self, v: impl Num) -> &mut Self { self.x = v.f(); self }
    pub fn set_y(&mut self, v: impl Num) -> &mut Self { self.y = v.f(); self }
    pub fn set_z(&mut self, v: impl Num) -> &mut Self { self.z = v.f(); self }
    pub fn get(&self, i: usize) -> f64 { match i { 0 => self.x, 1 => self.y, _ => self.z } }
    pub fn set_i(&mut self, i: usize, v: f64) { match i { 0 => self.x = v, 1 => self.y = v, _ => self.z = v } }
    pub fn copy(&mut self, o: &Vec3) -> &mut Self { *self = *o; self }
    pub fn add(&mut self, o: &Vec3) -> &mut Self { self.x += o.x; self.y += o.y; self.z += o.z; self }
    pub fn add_scalar(&mut self, s: impl Num) -> &mut Self { let s = s.f(); self.x += s; self.y += s; self.z += s; self }
    pub fn add_vectors(&mut self, a: &Vec3, b: &Vec3) -> &mut Self { self.x = a.x + b.x; self.y = a.y + b.y; self.z = a.z + b.z; self }
    pub fn add_scaled_vector(&mut self, v: &Vec3, s: impl Num) -> &mut Self { let s = s.f(); self.x += v.x * s; self.y += v.y * s; self.z += v.z * s; self }
    pub fn sub(&mut self, o: &Vec3) -> &mut Self { self.x -= o.x; self.y -= o.y; self.z -= o.z; self }
    pub fn sub_vectors(&mut self, a: &Vec3, b: &Vec3) -> &mut Self { self.x = a.x - b.x; self.y = a.y - b.y; self.z = a.z - b.z; self }
    pub fn multiply(&mut self, o: &Vec3) -> &mut Self { self.x *= o.x; self.y *= o.y; self.z *= o.z; self }
    pub fn multiply_scalar(&mut self, s: impl Num) -> &mut Self { let s = s.f(); self.x *= s; self.y *= s; self.z *= s; self }
    pub fn divide_scalar(&mut self, s: impl Num) -> &mut Self { self.multiply_scalar(1.0 / s.f()) }
    pub fn negate(&mut self) -> &mut Self { self.x = -self.x; self.y = -self.y; self.z = -self.z; self }
    pub fn dot(&self, o: &Vec3) -> f64 { self.x * o.x + self.y * o.y + self.z * o.z }
    pub fn length_sq(&self) -> f64 { self.x * self.x + self.y * self.y + self.z * self.z }
    pub fn length(&self) -> f64 { self.length_sq().sqrt() }
    pub fn manhattan_length(&self) -> f64 { self.x.abs() + self.y.abs() + self.z.abs() }
    pub fn normalize(&mut self) -> &mut Self { let l = self.length(); self.divide_scalar(if l == 0.0 { 1.0 } else { l }) }
    pub fn set_length(&mut self, l: impl Num) -> &mut Self { let l = l.f(); self.normalize().multiply_scalar(l) }
    pub fn lerp(&mut self, o: &Vec3, t: impl Num) -> &mut Self { let t = t.f(); self.x += (o.x - self.x) * t; self.y += (o.y - self.y) * t; self.z += (o.z - self.z) * t; self }
    pub fn lerp_vectors(&mut self, a: &Vec3, b: &Vec3, t: impl Num) -> &mut Self { let t = t.f(); self.x = a.x + (b.x - a.x) * t; self.y = a.y + (b.y - a.y) * t; self.z = a.z + (b.z - a.z) * t; self }
    pub fn cross(&mut self, o: &Vec3) -> &mut Self { let a = *self; self.cross_vectors(&a, o) }
    pub fn cross_vectors(&mut self, a: &Vec3, b: &Vec3) -> &mut Self {
        let (ax, ay, az, bx, by, bz) = (a.x, a.y, a.z, b.x, b.y, b.z);
        self.x = ay * bz - az * by; self.y = az * bx - ax * bz; self.z = ax * by - ay * bx; self
    }
    pub fn distance_to(&self, o: &Vec3) -> f64 { self.distance_to_squared(o).sqrt() }
    pub fn distance_to_squared(&self, o: &Vec3) -> f64 { let dx = self.x - o.x; let dy = self.y - o.y; let dz = self.z - o.z; dx * dx + dy * dy + dz * dz }
    pub fn angle_to(&self, o: &Vec3) -> f64 {
        let d = (self.length_sq() * o.length_sq()).sqrt();
        if d == 0.0 { return PI / 2.0; }
        clamp(self.dot(o) / d, -1.0, 1.0).acos()
    }
    pub fn apply_quaternion(&mut self, q: &Quat) -> &mut Self {
        let (vx, vy, vz) = (self.x, self.y, self.z);
        let (qx, qy, qz, qw) = (q.x, q.y, q.z, q.w);
        let tx = 2.0 * (qy * vz - qz * vy);
        let ty = 2.0 * (qz * vx - qx * vz);
        let tz = 2.0 * (qx * vy - qy * vx);
        self.x = vx + qw * tx + qy * tz - qz * ty;
        self.y = vy + qw * ty + qz * tx - qx * tz;
        self.z = vz + qw * tz + qx * ty - qy * tx;
        self
    }
    pub fn apply_euler(&mut self, e: &Euler) -> &mut Self { let q = Quat::from_euler(e); self.apply_quaternion(&q) }
    pub fn apply_axis_angle(&mut self, axis: &Vec3, angle: impl Num) -> &mut Self { let q = Quat::from_axis_angle(axis, angle.f()); self.apply_quaternion(&q) }
    pub fn apply_matrix4(&mut self, m: &Mat4) -> &mut Self {
        let (x, y, z) = (self.x, self.y, self.z);
        let e = &m.e;
        let w = 1.0 / (e[3] * x + e[7] * y + e[11] * z + e[15]);
        self.x = (e[0] * x + e[4] * y + e[8] * z + e[12]) * w;
        self.y = (e[1] * x + e[5] * y + e[9] * z + e[13]) * w;
        self.z = (e[2] * x + e[6] * y + e[10] * z + e[14]) * w;
        self
    }
    pub fn apply_matrix3(&mut self, m: &Mat3) -> &mut Self {
        let (x, y, z) = (self.x, self.y, self.z);
        let e = &m.e;
        self.x = e[0] * x + e[3] * y + e[6] * z;
        self.y = e[1] * x + e[4] * y + e[7] * z;
        self.z = e[2] * x + e[5] * y + e[8] * z;
        self
    }
    pub fn transform_direction(&mut self, m: &Mat4) -> &mut Self {
        let (x, y, z) = (self.x, self.y, self.z);
        let e = &m.e;
        self.x = e[0] * x + e[4] * y + e[8] * z;
        self.y = e[1] * x + e[5] * y + e[9] * z;
        self.z = e[2] * x + e[6] * y + e[10] * z;
        self.normalize()
    }
    pub fn set_from_matrix_position(&mut self, m: &Mat4) -> &mut Self { self.x = m.e[12]; self.y = m.e[13]; self.z = m.e[14]; self }
    pub fn set_from_matrix_column(&mut self, m: &Mat4, i: usize) -> &mut Self { self.x = m.e[i * 4]; self.y = m.e[i * 4 + 1]; self.z = m.e[i * 4 + 2]; self }
    pub fn set_from_matrix_scale(&mut self, m: &Mat4) -> &mut Self {
        let sx = Vec3::new(m.e[0], m.e[1], m.e[2]).length();
        let sy = Vec3::new(m.e[4], m.e[5], m.e[6]).length();
        let sz = Vec3::new(m.e[8], m.e[9], m.e[10]).length();
        self.x = sx; self.y = sy; self.z = sz; self
    }
    pub fn min(&mut self, o: &Vec3) -> &mut Self { self.x = self.x.min(o.x); self.y = self.y.min(o.y); self.z = self.z.min(o.z); self }
    pub fn max(&mut self, o: &Vec3) -> &mut Self { self.x = self.x.max(o.x); self.y = self.y.max(o.y); self.z = self.z.max(o.z); self }
    pub fn equals(&self, o: &Vec3) -> bool { self == o }
    pub fn project_on_plane(&mut self, n: &Vec3) -> &mut Self { let mut v = *self; v.project_on_vector(n); self.sub(&v) }
    pub fn project_on_vector(&mut self, v: &Vec3) -> &mut Self {
        let d = v.length_sq();
        if d == 0.0 { return self.set(0, 0, 0); }
        let s = v.dot(self) / d;
        *self = *v; self.multiply_scalar(s)
    }
    pub fn reflect(&mut self, n: &Vec3) -> &mut Self { let d = 2.0 * self.dot(n); self.x -= n.x * d; self.y -= n.y * d; self.z -= n.z * d; self }
    // value-returning helpers (Rust-friendly)
    pub fn added(self, o: Vec3) -> Vec3 { Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z) }
    pub fn subbed(self, o: Vec3) -> Vec3 { Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z) }
    pub fn scaled(self, s: f64) -> Vec3 { Vec3::new(self.x * s, self.y * s, self.z * s) }
    pub fn normalized(self) -> Vec3 { let mut v = self; v.normalize(); v }
    pub fn crossed(self, o: Vec3) -> Vec3 { let mut v = self; v.cross(&o); v }
    pub fn to_f32(self) -> [f32; 3] { [self.x as f32, self.y as f32, self.z as f32] }
}
impl std::ops::Add for Vec3 { type Output = Vec3; fn add(self, o: Vec3) -> Vec3 { self.added(o) } }
impl std::ops::Sub for Vec3 { type Output = Vec3; fn sub(self, o: Vec3) -> Vec3 { self.subbed(o) } }
impl std::ops::Mul<f64> for Vec3 { type Output = Vec3; fn mul(self, s: f64) -> Vec3 { self.scaled(s) } }
impl std::ops::Div<f64> for Vec3 { type Output = Vec3; fn div(self, s: f64) -> Vec3 { self.scaled(1.0 / s) } }
impl std::ops::Neg for Vec3 { type Output = Vec3; fn neg(self) -> Vec3 { Vec3::new(-self.x, -self.y, -self.z) } }
impl std::ops::AddAssign for Vec3 { fn add_assign(&mut self, o: Vec3) { self.x += o.x; self.y += o.y; self.z += o.z; } }
impl std::ops::SubAssign for Vec3 { fn sub_assign(&mut self, o: Vec3) { self.x -= o.x; self.y -= o.y; self.z -= o.z; } }
impl std::ops::MulAssign<f64> for Vec3 { fn mul_assign(&mut self, s: f64) { self.x *= s; self.y *= s; self.z *= s; } }

// ---------------------------------------------------------------- Vec4
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec4 { pub x: f64, pub y: f64, pub z: f64, pub w: f64 }
impl Vec4 {
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self { Self { x, y, z, w } }
    pub fn set(&mut self, x: impl Num, y: impl Num, z: impl Num, w: impl Num) -> &mut Self { self.x = x.f(); self.y = y.f(); self.z = z.f(); self.w = w.f(); self }
}

// ---------------------------------------------------------------- Euler
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EulerOrder { #[default] XYZ, YXZ, ZXY, ZYX, YZX, XZY }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Euler { pub x: f64, pub y: f64, pub z: f64, pub order: EulerOrder }
impl Euler {
    pub const fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z, order: EulerOrder::XYZ } }
    pub fn with_order(x: f64, y: f64, z: f64, order: EulerOrder) -> Self { Self { x, y, z, order } }
    pub fn set(&mut self, x: impl Num, y: impl Num, z: impl Num) -> &mut Self { self.x = x.f(); self.y = y.f(); self.z = z.f(); self }
    pub fn copy(&mut self, o: &Euler) -> &mut Self { *self = *o; self }
    pub fn set_from_quaternion(&mut self, q: &Quat, order: EulerOrder) -> &mut Self {
        let m = Mat4::from_quat(q);
        self.set_from_rotation_matrix(&m, order)
    }
    pub fn set_from_rotation_matrix(&mut self, m: &Mat4, order: EulerOrder) -> &mut Self {
        let te = &m.e;
        let (m11, m12, m13) = (te[0], te[4], te[8]);
        let (m21, m22, m23) = (te[1], te[5], te[9]);
        let (m31, m32, m33) = (te[2], te[6], te[10]);
        match order {
            EulerOrder::XYZ => {
                self.y = clamp(m13, -1.0, 1.0).asin();
                if m13.abs() < 0.9999999 { self.x = (-m23).atan2(m33); self.z = (-m12).atan2(m11); }
                else { self.x = m32.atan2(m22); self.z = 0.0; }
            }
            EulerOrder::YXZ => {
                self.x = (-clamp(m23, -1.0, 1.0)).asin();
                if m23.abs() < 0.9999999 { self.y = m13.atan2(m33); self.z = m21.atan2(m22); }
                else { self.y = (-m31).atan2(m11); self.z = 0.0; }
            }
            EulerOrder::ZXY => {
                self.x = clamp(m32, -1.0, 1.0).asin();
                if m32.abs() < 0.9999999 { self.y = (-m31).atan2(m33); self.z = (-m12).atan2(m22); }
                else { self.y = 0.0; self.z = m21.atan2(m11); }
            }
            EulerOrder::ZYX => {
                self.y = (-clamp(m31, -1.0, 1.0)).asin();
                if m31.abs() < 0.9999999 { self.x = m32.atan2(m33); self.z = m21.atan2(m11); }
                else { self.x = 0.0; self.z = (-m12).atan2(m22); }
            }
            EulerOrder::YZX => {
                self.z = clamp(m21, -1.0, 1.0).asin();
                if m21.abs() < 0.9999999 { self.x = (-m23).atan2(m22); self.y = (-m31).atan2(m11); }
                else { self.x = 0.0; self.y = m13.atan2(m33); }
            }
            EulerOrder::XZY => {
                self.z = (-clamp(m12, -1.0, 1.0)).asin();
                if m12.abs() < 0.9999999 { self.x = m32.atan2(m22); self.y = m13.atan2(m11); }
                else { self.x = (-m23).atan2(m33); self.y = 0.0; }
            }
        }
        self.order = order;
        self
    }
}

// ---------------------------------------------------------------- Quat
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat { pub x: f64, pub y: f64, pub z: f64, pub w: f64 }
impl Default for Quat { fn default() -> Self { Quat::IDENTITY } }
impl Quat {
    pub const IDENTITY: Quat = Quat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self { Self { x, y, z, w } }
    pub fn set(&mut self, x: impl Num, y: impl Num, z: impl Num, w: impl Num) -> &mut Self { self.x = x.f(); self.y = y.f(); self.z = z.f(); self.w = w.f(); self }
    pub fn copy(&mut self, o: &Quat) -> &mut Self { *self = *o; self }
    pub fn identity(&mut self) -> &mut Self { *self = Quat::IDENTITY; self }
    pub fn from_euler(e: &Euler) -> Quat { let mut q = Quat::IDENTITY; q.set_from_euler(e); q }
    pub fn set_from_euler(&mut self, e: &Euler) -> &mut Self {
        let (c1, c2, c3) = ((e.x / 2.0).cos(), (e.y / 2.0).cos(), (e.z / 2.0).cos());
        let (s1, s2, s3) = ((e.x / 2.0).sin(), (e.y / 2.0).sin(), (e.z / 2.0).sin());
        match e.order {
            EulerOrder::XYZ => { self.x = s1 * c2 * c3 + c1 * s2 * s3; self.y = c1 * s2 * c3 - s1 * c2 * s3; self.z = c1 * c2 * s3 + s1 * s2 * c3; self.w = c1 * c2 * c3 - s1 * s2 * s3; }
            EulerOrder::YXZ => { self.x = s1 * c2 * c3 + c1 * s2 * s3; self.y = c1 * s2 * c3 - s1 * c2 * s3; self.z = c1 * c2 * s3 - s1 * s2 * c3; self.w = c1 * c2 * c3 + s1 * s2 * s3; }
            EulerOrder::ZXY => { self.x = s1 * c2 * c3 - c1 * s2 * s3; self.y = c1 * s2 * c3 + s1 * c2 * s3; self.z = c1 * c2 * s3 + s1 * s2 * c3; self.w = c1 * c2 * c3 - s1 * s2 * s3; }
            EulerOrder::ZYX => { self.x = s1 * c2 * c3 - c1 * s2 * s3; self.y = c1 * s2 * c3 + s1 * c2 * s3; self.z = c1 * c2 * s3 - s1 * s2 * c3; self.w = c1 * c2 * c3 + s1 * s2 * s3; }
            EulerOrder::YZX => { self.x = s1 * c2 * c3 + c1 * s2 * s3; self.y = c1 * s2 * c3 + s1 * c2 * s3; self.z = c1 * c2 * s3 - s1 * s2 * c3; self.w = c1 * c2 * c3 - s1 * s2 * s3; }
            EulerOrder::XZY => { self.x = s1 * c2 * c3 - c1 * s2 * s3; self.y = c1 * s2 * c3 - s1 * c2 * s3; self.z = c1 * c2 * s3 + s1 * s2 * c3; self.w = c1 * c2 * c3 + s1 * s2 * s3; }
        }
        self
    }
    pub fn from_axis_angle(axis: &Vec3, angle: f64) -> Quat {
        let h = angle / 2.0; let s = h.sin();
        Quat::new(axis.x * s, axis.y * s, axis.z * s, h.cos())
    }
    pub fn set_from_axis_angle(&mut self, axis: &Vec3, angle: impl Num) -> &mut Self { *self = Quat::from_axis_angle(axis, angle.f()); self }
    pub fn set_from_rotation_matrix(&mut self, m: &Mat4) -> &mut Self {
        let te = &m.e;
        let (m11, m12, m13, m21, m22, m23, m31, m32, m33) = (te[0], te[4], te[8], te[1], te[5], te[9], te[2], te[6], te[10]);
        let trace = m11 + m22 + m33;
        if trace > 0.0 {
            let s = 0.5 / (trace + 1.0).sqrt();
            self.w = 0.25 / s; self.x = (m32 - m23) * s; self.y = (m13 - m31) * s; self.z = (m21 - m12) * s;
        } else if m11 > m22 && m11 > m33 {
            let s = 2.0 * (1.0 + m11 - m22 - m33).sqrt();
            self.w = (m32 - m23) / s; self.x = 0.25 * s; self.y = (m12 + m21) / s; self.z = (m13 + m31) / s;
        } else if m22 > m33 {
            let s = 2.0 * (1.0 + m22 - m11 - m33).sqrt();
            self.w = (m13 - m31) / s; self.x = (m12 + m21) / s; self.y = 0.25 * s; self.z = (m23 + m32) / s;
        } else {
            let s = 2.0 * (1.0 + m33 - m11 - m22).sqrt();
            self.w = (m21 - m12) / s; self.x = (m13 + m31) / s; self.y = (m23 + m32) / s; self.z = 0.25 * s;
        }
        self
    }
    pub fn set_from_unit_vectors(&mut self, from: &Vec3, to: &Vec3) -> &mut Self {
        let mut r = from.dot(to) + 1.0;
        if r < f64::EPSILON {
            r = 0.0;
            if from.x.abs() > from.z.abs() { self.x = -from.y; self.y = from.x; self.z = 0.0; self.w = r; }
            else { self.x = 0.0; self.y = -from.z; self.z = from.y; self.w = r; }
        } else {
            self.x = from.y * to.z - from.z * to.y;
            self.y = from.z * to.x - from.x * to.z;
            self.z = from.x * to.y - from.y * to.x;
            self.w = r;
        }
        self.normalize()
    }
    pub fn length(&self) -> f64 { (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt() }
    pub fn normalize(&mut self) -> &mut Self {
        let l = self.length();
        if l == 0.0 { *self = Quat::IDENTITY; } else { let l = 1.0 / l; self.x *= l; self.y *= l; self.z *= l; self.w *= l; }
        self
    }
    pub fn invert(&mut self) -> &mut Self { self.x = -self.x; self.y = -self.y; self.z = -self.z; self }
    pub fn conjugate(&mut self) -> &mut Self { self.invert() }
    pub fn dot(&self, o: &Quat) -> f64 { self.x * o.x + self.y * o.y + self.z * o.z + self.w * o.w }
    pub fn multiply(&mut self, q: &Quat) -> &mut Self { let a = *self; self.multiply_quaternions(&a, q) }
    pub fn premultiply(&mut self, q: &Quat) -> &mut Self { let a = *self; self.multiply_quaternions(q, &a) }
    pub fn multiply_quaternions(&mut self, a: &Quat, b: &Quat) -> &mut Self {
        let (qax, qay, qaz, qaw) = (a.x, a.y, a.z, a.w);
        let (qbx, qby, qbz, qbw) = (b.x, b.y, b.z, b.w);
        self.x = qax * qbw + qaw * qbx + qay * qbz - qaz * qby;
        self.y = qay * qbw + qaw * qby + qaz * qbx - qax * qbz;
        self.z = qaz * qbw + qaw * qbz + qax * qby - qay * qbx;
        self.w = qaw * qbw - qax * qbx - qay * qby - qaz * qbz;
        self
    }
    pub fn angle_to(&self, q: &Quat) -> f64 { 2.0 * clamp(self.dot(q), -1.0, 1.0).abs().acos() }
    pub fn rotate_towards(&mut self, q: &Quat, step: f64) -> &mut Self {
        let angle = self.angle_to(q);
        if angle == 0.0 { return self; }
        let t = (step / angle).min(1.0);
        self.slerp(q, t)
    }
    pub fn slerp(&mut self, qb: &Quat, t: impl Num) -> &mut Self {
        let t = t.f();
        if t == 0.0 { return self; }
        if t == 1.0 { *self = *qb; return self; }
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        let mut cos_half = w * qb.w + x * qb.x + y * qb.y + z * qb.z;
        if cos_half < 0.0 { self.w = -qb.w; self.x = -qb.x; self.y = -qb.y; self.z = -qb.z; cos_half = -cos_half; }
        else { *self = *qb; }
        if cos_half >= 1.0 { self.w = w; self.x = x; self.y = y; self.z = z; return self; }
        let sqr_sin = 1.0 - cos_half * cos_half;
        if sqr_sin <= f64::EPSILON {
            let s = 1.0 - t;
            self.w = s * w + t * self.w; self.x = s * x + t * self.x; self.y = s * y + t * self.y; self.z = s * z + t * self.z;
            return self.normalize();
        }
        let sin_half = sqr_sin.sqrt();
        let half = sin_half.atan2(cos_half);
        let ra = ((1.0 - t) * half).sin() / sin_half;
        let rb = (t * half).sin() / sin_half;
        self.w = w * ra + self.w * rb; self.x = x * ra + self.x * rb; self.y = y * ra + self.y * rb; self.z = z * ra + self.z * rb;
        self
    }
    pub fn slerp_quaternions(&mut self, a: &Quat, b: &Quat, t: impl Num) -> &mut Self { *self = *a; self.slerp(b, t) }
}

// ---------------------------------------------------------------- Mat3
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat3 { pub e: [f64; 9] }
impl Default for Mat3 { fn default() -> Self { Mat3 { e: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0] } } }
impl Mat3 {
    pub fn get_normal_matrix(&mut self, m: &Mat4) -> &mut Self {
        self.set_from_matrix4(m).invert().transpose()
    }
    pub fn set_from_matrix4(&mut self, m: &Mat4) -> &mut Self {
        let me = &m.e;
        self.e = [me[0], me[1], me[2], me[4], me[5], me[6], me[8], me[9], me[10]];
        self
    }
    pub fn invert(&mut self) -> &mut Self {
        let te = &mut self.e;
        let (n11, n21, n31, n12, n22, n32, n13, n23, n33) = (te[0], te[1], te[2], te[3], te[4], te[5], te[6], te[7], te[8]);
        let t11 = n33 * n22 - n32 * n23; let t12 = n32 * n13 - n33 * n12; let t13 = n23 * n12 - n22 * n13;
        let det = n11 * t11 + n21 * t12 + n31 * t13;
        if det == 0.0 { *te = [0.0; 9]; return self; }
        let di = 1.0 / det;
        te[0] = t11 * di; te[1] = (n31 * n23 - n33 * n21) * di; te[2] = (n32 * n21 - n31 * n22) * di;
        te[3] = t12 * di; te[4] = (n33 * n11 - n31 * n13) * di; te[5] = (n31 * n12 - n32 * n11) * di;
        te[6] = t13 * di; te[7] = (n21 * n13 - n23 * n11) * di; te[8] = (n22 * n11 - n21 * n12) * di;
        self
    }
    pub fn transpose(&mut self) -> &mut Self {
        let m = &mut self.e;
        m.swap(1, 3); m.swap(2, 6); m.swap(5, 7);
        self
    }
}

// ---------------------------------------------------------------- Mat4 (column-major like three)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 { pub e: [f64; 16] }
impl Default for Mat4 { fn default() -> Self { Mat4::IDENTITY } }
impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 { e: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0] };
    pub fn new() -> Self { Mat4::IDENTITY }
    /// Row-major arguments like three's `Matrix4.set`.
    pub fn set(&mut self, n11: f64, n12: f64, n13: f64, n14: f64, n21: f64, n22: f64, n23: f64, n24: f64, n31: f64, n32: f64, n33: f64, n34: f64, n41: f64, n42: f64, n43: f64, n44: f64) -> &mut Self {
        self.e = [n11, n21, n31, n41, n12, n22, n32, n42, n13, n23, n33, n43, n14, n24, n34, n44];
        self
    }
    pub fn identity(&mut self) -> &mut Self { *self = Mat4::IDENTITY; self }
    pub fn copy(&mut self, o: &Mat4) -> &mut Self { *self = *o; self }
    pub fn from_quat(q: &Quat) -> Mat4 { Mat4::compose(&Vec3::ZERO, q, &Vec3::new(1.0, 1.0, 1.0)) }
    pub fn make_rotation_from_quaternion(&mut self, q: &Quat) -> &mut Self { *self = Mat4::from_quat(q); self }
    pub fn make_rotation_from_euler(&mut self, e: &Euler) -> &mut Self { *self = Mat4::from_quat(&Quat::from_euler(e)); self }
    pub fn compose(p: &Vec3, q: &Quat, s: &Vec3) -> Mat4 {
        let (x, y, z, w) = (q.x, q.y, q.z, q.w);
        let (x2, y2, z2) = (x + x, y + y, z + z);
        let (xx, xy, xz) = (x * x2, x * y2, x * z2);
        let (yy, yz, zz) = (y * y2, y * z2, z * z2);
        let (wx, wy, wz) = (w * x2, w * y2, w * z2);
        let (sx, sy, sz) = (s.x, s.y, s.z);
        Mat4 { e: [
            (1.0 - (yy + zz)) * sx, (xy + wz) * sx, (xz - wy) * sx, 0.0,
            (xy - wz) * sy, (1.0 - (xx + zz)) * sy, (yz + wx) * sy, 0.0,
            (xz + wy) * sz, (yz - wx) * sz, (1.0 - (xx + yy)) * sz, 0.0,
            p.x, p.y, p.z, 1.0,
        ] }
    }
    pub fn compose_into(&mut self, p: &Vec3, q: &Quat, s: &Vec3) -> &mut Self { *self = Mat4::compose(p, q, s); self }
    pub fn decompose(&self) -> (Vec3, Quat, Vec3) {
        let te = &self.e;
        let mut sx = Vec3::new(te[0], te[1], te[2]).length();
        let sy = Vec3::new(te[4], te[5], te[6]).length();
        let sz = Vec3::new(te[8], te[9], te[10]).length();
        if self.determinant() < 0.0 { sx = -sx; }
        let pos = Vec3::new(te[12], te[13], te[14]);
        let mut m = *self;
        let (isx, isy, isz) = (1.0 / sx, 1.0 / sy, 1.0 / sz);
        m.e[0] *= isx; m.e[1] *= isx; m.e[2] *= isx;
        m.e[4] *= isy; m.e[5] *= isy; m.e[6] *= isy;
        m.e[8] *= isz; m.e[9] *= isz; m.e[10] *= isz;
        let mut q = Quat::IDENTITY;
        q.set_from_rotation_matrix(&m);
        (pos, q, Vec3::new(sx, sy, sz))
    }
    pub fn multiply(&mut self, m: &Mat4) -> &mut Self { let a = *self; *self = Mat4::mul(&a, m); self }
    pub fn premultiply(&mut self, m: &Mat4) -> &mut Self { let a = *self; *self = Mat4::mul(m, &a); self }
    pub fn multiply_matrices(&mut self, a: &Mat4, b: &Mat4) -> &mut Self { *self = Mat4::mul(a, b); self }
    pub fn mul(a: &Mat4, b: &Mat4) -> Mat4 {
        let ae = &a.e; let be = &b.e;
        let mut te = [0.0; 16];
        for r in 0..4 {
            for c in 0..4 {
                te[c * 4 + r] = ae[r] * be[c * 4] + ae[4 + r] * be[c * 4 + 1] + ae[8 + r] * be[c * 4 + 2] + ae[12 + r] * be[c * 4 + 3];
            }
        }
        Mat4 { e: te }
    }
    pub fn determinant(&self) -> f64 {
        let te = &self.e;
        let (n11, n12, n13, n14) = (te[0], te[4], te[8], te[12]);
        let (n21, n22, n23, n24) = (te[1], te[5], te[9], te[13]);
        let (n31, n32, n33, n34) = (te[2], te[6], te[10], te[14]);
        let (n41, n42, n43, n44) = (te[3], te[7], te[11], te[15]);
        n41 * (n14 * n23 * n32 - n13 * n24 * n32 - n14 * n22 * n33 + n12 * n24 * n33 + n13 * n22 * n34 - n12 * n23 * n34)
            + n42 * (n11 * n23 * n34 - n11 * n24 * n33 + n14 * n21 * n33 - n13 * n21 * n34 + n13 * n24 * n31 - n14 * n23 * n31)
            + n43 * (n11 * n24 * n32 - n11 * n22 * n34 - n14 * n21 * n32 + n12 * n21 * n34 + n14 * n22 * n31 - n12 * n24 * n31)
            + n44 * (-n13 * n22 * n31 - n11 * n23 * n32 + n11 * n22 * n33 + n13 * n21 * n32 - n12 * n21 * n33 + n12 * n23 * n31)
    }
    pub fn invert(&mut self) -> &mut Self {
        let te = self.e;
        let (n11, n21, n31, n41) = (te[0], te[1], te[2], te[3]);
        let (n12, n22, n32, n42) = (te[4], te[5], te[6], te[7]);
        let (n13, n23, n33, n43) = (te[8], te[9], te[10], te[11]);
        let (n14, n24, n34, n44) = (te[12], te[13], te[14], te[15]);
        let t11 = n23 * n34 * n42 - n24 * n33 * n42 + n24 * n32 * n43 - n22 * n34 * n43 - n23 * n32 * n44 + n22 * n33 * n44;
        let t12 = n14 * n33 * n42 - n13 * n34 * n42 - n14 * n32 * n43 + n12 * n34 * n43 + n13 * n32 * n44 - n12 * n33 * n44;
        let t13 = n13 * n24 * n42 - n14 * n23 * n42 + n14 * n22 * n43 - n12 * n24 * n43 - n13 * n22 * n44 + n12 * n23 * n44;
        let t14 = n14 * n23 * n32 - n13 * n24 * n32 - n14 * n22 * n33 + n12 * n24 * n33 + n13 * n22 * n34 - n12 * n23 * n34;
        let det = n11 * t11 + n21 * t12 + n31 * t13 + n41 * t14;
        if det == 0.0 { self.e = [0.0; 16]; return self; }
        let d = 1.0 / det;
        let e = &mut self.e;
        e[0] = t11 * d;
        e[1] = (n24 * n33 * n41 - n23 * n34 * n41 - n24 * n31 * n43 + n21 * n34 * n43 + n23 * n31 * n44 - n21 * n33 * n44) * d;
        e[2] = (n22 * n34 * n41 - n24 * n32 * n41 + n24 * n31 * n42 - n21 * n34 * n42 - n22 * n31 * n44 + n21 * n32 * n44) * d;
        e[3] = (n23 * n32 * n41 - n22 * n33 * n41 - n23 * n31 * n42 + n21 * n33 * n42 + n22 * n31 * n43 - n21 * n32 * n43) * d;
        e[4] = t12 * d;
        e[5] = (n13 * n34 * n41 - n14 * n33 * n41 + n14 * n31 * n43 - n11 * n34 * n43 - n13 * n31 * n44 + n11 * n33 * n44) * d;
        e[6] = (n14 * n32 * n41 - n12 * n34 * n41 - n14 * n31 * n42 + n11 * n34 * n42 + n12 * n31 * n44 - n11 * n32 * n44) * d;
        e[7] = (n12 * n33 * n41 - n13 * n32 * n41 + n13 * n31 * n42 - n11 * n33 * n42 - n12 * n31 * n43 + n11 * n32 * n43) * d;
        e[8] = t13 * d;
        e[9] = (n14 * n23 * n41 - n13 * n24 * n41 - n14 * n21 * n43 + n11 * n24 * n43 + n13 * n21 * n44 - n11 * n23 * n44) * d;
        e[10] = (n12 * n24 * n41 - n14 * n22 * n41 + n14 * n21 * n42 - n11 * n24 * n42 - n12 * n21 * n44 + n11 * n22 * n44) * d;
        e[11] = (n13 * n22 * n41 - n12 * n23 * n41 - n13 * n21 * n42 + n11 * n23 * n42 + n12 * n21 * n43 - n11 * n22 * n43) * d;
        e[12] = t14 * d;
        e[13] = (n13 * n24 * n31 - n14 * n23 * n31 + n14 * n21 * n33 - n11 * n24 * n33 - n13 * n21 * n34 + n11 * n23 * n34) * d;
        e[14] = (n14 * n22 * n31 - n12 * n24 * n31 - n14 * n21 * n32 + n11 * n24 * n32 + n12 * n21 * n34 - n11 * n22 * n34) * d;
        e[15] = (n12 * n23 * n31 - n13 * n22 * n31 + n13 * n21 * n32 - n11 * n23 * n32 - n12 * n21 * n33 + n11 * n22 * n33) * d;
        self
    }
    pub fn inverted(&self) -> Mat4 { let mut m = *self; m.invert(); m }
    pub fn transpose(&mut self) -> &mut Self {
        let te = &mut self.e;
        te.swap(1, 4); te.swap(2, 8); te.swap(6, 9); te.swap(3, 12); te.swap(7, 13); te.swap(11, 14);
        self
    }
    pub fn make_translation(&mut self, x: impl Num, y: impl Num, z: impl Num) -> &mut Self { self.set(1.0, 0.0, 0.0, x.f(), 0.0, 1.0, 0.0, y.f(), 0.0, 0.0, 1.0, z.f(), 0.0, 0.0, 0.0, 1.0) }
    pub fn make_scale(&mut self, x: impl Num, y: impl Num, z: impl Num) -> &mut Self { self.set(x.f(), 0.0, 0.0, 0.0, 0.0, y.f(), 0.0, 0.0, 0.0, 0.0, z.f(), 0.0, 0.0, 0.0, 0.0, 1.0) }
    pub fn make_rotation_x(&mut self, t: impl Num) -> &mut Self { let (c, s) = (t.f().cos(), t.f().sin()); self.set(1.0, 0.0, 0.0, 0.0, 0.0, c, -s, 0.0, 0.0, s, c, 0.0, 0.0, 0.0, 0.0, 1.0) }
    pub fn make_rotation_y(&mut self, t: impl Num) -> &mut Self { let (c, s) = (t.f().cos(), t.f().sin()); self.set(c, 0.0, s, 0.0, 0.0, 1.0, 0.0, 0.0, -s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0) }
    pub fn make_rotation_z(&mut self, t: impl Num) -> &mut Self { let (c, s) = (t.f().cos(), t.f().sin()); self.set(c, -s, 0.0, 0.0, s, c, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0) }
    pub fn make_rotation_axis(&mut self, axis: &Vec3, angle: impl Num) -> &mut Self {
        let a = angle.f(); let c = a.cos(); let s = a.sin(); let t = 1.0 - c;
        let (x, y, z) = (axis.x, axis.y, axis.z); let (tx, ty) = (t * x, t * y);
        self.set(tx * x + c, tx * y - s * z, tx * z + s * y, 0.0, tx * y + s * z, ty * y + c, ty * z - s * x, 0.0, tx * z - s * y, ty * z + s * x, t * z * z + c, 0.0, 0.0, 0.0, 0.0, 1.0)
    }
    pub fn make_basis(&mut self, x: &Vec3, y: &Vec3, z: &Vec3) -> &mut Self { self.set(x.x, y.x, z.x, 0.0, x.y, y.y, z.y, 0.0, x.z, y.z, z.z, 0.0, 0.0, 0.0, 0.0, 1.0) }
    pub fn extract_rotation(&mut self, m: &Mat4) -> &mut Self {
        let me = &m.e;
        let sx = 1.0 / Vec3::new(me[0], me[1], me[2]).length();
        let sy = 1.0 / Vec3::new(me[4], me[5], me[6]).length();
        let sz = 1.0 / Vec3::new(me[8], me[9], me[10]).length();
        self.e = [me[0] * sx, me[1] * sx, me[2] * sx, 0.0, me[4] * sy, me[5] * sy, me[6] * sy, 0.0, me[8] * sz, me[9] * sz, me[10] * sz, 0.0, 0.0, 0.0, 0.0, 1.0];
        self
    }
    /// three's Matrix4.lookAt(eye, target, up): rotation whose +Z points from target to eye.
    pub fn look_at(&mut self, eye: &Vec3, target: &Vec3, up: &Vec3) -> &mut Self {
        let mut z = *eye; z.sub(target);
        if z.length_sq() == 0.0 { z.z = 1.0; }
        z.normalize();
        let mut x = *up; x.cross(&z);
        if x.length_sq() == 0.0 {
            if up.z.abs() == 1.0 { z.x += 0.0001; } else { z.z += 0.0001; }
            z.normalize();
            x = *up; x.cross(&z);
        }
        x.normalize();
        let mut y = z; y.cross(&x);
        let te = &mut self.e;
        te[0] = x.x; te[4] = y.x; te[8] = z.x;
        te[1] = x.y; te[5] = y.y; te[9] = z.y;
        te[2] = x.z; te[6] = y.z; te[10] = z.z;
        self
    }
    pub fn get_max_scale_on_axis(&self) -> f64 {
        let te = &self.e;
        let sx = te[0] * te[0] + te[1] * te[1] + te[2] * te[2];
        let sy = te[4] * te[4] + te[5] * te[5] + te[6] * te[6];
        let sz = te[8] * te[8] + te[9] * te[9] + te[10] * te[10];
        sx.max(sy).max(sz).sqrt()
    }
    pub fn get_position(&self) -> Vec3 { Vec3::new(self.e[12], self.e[13], self.e[14]) }
    pub fn set_position(&mut self, x: impl Num, y: impl Num, z: impl Num) -> &mut Self { self.e[12] = x.f(); self.e[13] = y.f(); self.e[14] = z.f(); self }
    pub fn to_f32(&self) -> [f32; 16] { let mut o = [0f32; 16]; for i in 0..16 { o[i] = self.e[i] as f32; } o }
}
impl std::ops::Mul for Mat4 { type Output = Mat4; fn mul(self, o: Mat4) -> Mat4 { Mat4::mul(&self, &o) } }

// ---------------------------------------------------------------- Color
/// Linear-sRGB working colour, like three r160 with ColorManagement enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color { pub r: f64, pub g: f64, pub b: f64 }

#[inline] pub fn srgb_to_linear(c: f64) -> f64 { if c < 0.04045 { c * 0.0773993808 } else { (c * 0.9478672986 + 0.0521327014).powf(2.4) } }
#[inline] pub fn linear_to_srgb(c: f64) -> f64 { if c < 0.0031308 { c * 12.92 } else { 1.055 * c.powf(0.41666) - 0.055 } }

impl Color {
    pub const fn rgb_linear(r: f64, g: f64, b: f64) -> Self { Color { r, g, b } }
    /// `new THREE.Color(0xRRGGBB)`
    pub fn hex(h: u32) -> Self { let mut c = Color::default(); c.set_hex(h); c }
    pub fn white() -> Self { Color { r: 1.0, g: 1.0, b: 1.0 } }
    pub fn set_hex(&mut self, h: impl Num) -> &mut Self {
        let h = h.f() as u32;
        self.r = srgb_to_linear(((h >> 16) & 255) as f64 / 255.0);
        self.g = srgb_to_linear(((h >> 8) & 255) as f64 / 255.0);
        self.b = srgb_to_linear((h & 255) as f64 / 255.0);
        self
    }
    pub fn get_hex(&self) -> u32 {
        let c = |v: f64| (clamp(linear_to_srgb(v) * 255.0, 0.0, 255.0)).round() as u32;
        (c(self.r) << 16) | (c(self.g) << 8) | c(self.b)
    }
    /// `setRGB(r, g, b)` (values already in the linear working space)
    pub fn set_rgb(&mut self, r: impl Num, g: impl Num, b: impl Num) -> &mut Self { self.r = r.f(); self.g = g.f(); self.b = b.f(); self }
    pub fn set_scalar(&mut self, s: impl Num) -> &mut Self { let s = s.f(); self.r = s; self.g = s; self.b = s; self }
    /// `setHSL(h, s, l)` — in the working space (no conversion), as in three r160.
    pub fn set_hsl(&mut self, h: impl Num, s: impl Num, l: impl Num) -> &mut Self {
        let h = euclidean_modulo(h.f(), 1.0); let s = clamp(s.f(), 0.0, 1.0); let l = clamp(l.f(), 0.0, 1.0);
        if s == 0.0 { self.r = l; self.g = l; self.b = l; }
        else {
            let p = if l <= 0.5 { l * (1.0 + s) } else { l + s - l * s };
            let q = 2.0 * l - p;
            self.r = hue2rgb(q, p, h + 1.0 / 3.0);
            self.g = hue2rgb(q, p, h);
            self.b = hue2rgb(q, p, h - 1.0 / 3.0);
        }
        self
    }
    pub fn get_hsl(&self) -> (f64, f64, f64) {
        let (r, g, b) = (self.r, self.g, self.b);
        let max = r.max(g).max(b); let min = r.min(g).min(b);
        let l = (min + max) / 2.0;
        if min == max { return (0.0, 0.0, l); }
        let delta = max - min;
        let s = if l <= 0.5 { delta / (max + min) } else { delta / (2.0 - max - min) };
        let h = if max == r { (g - b) / delta + if g < b { 6.0 } else { 0.0 } } else if max == g { (b - r) / delta + 2.0 } else { (r - g) / delta + 4.0 };
        (h / 6.0, s, l)
    }
    pub fn offset_hsl(&mut self, h: impl Num, s: impl Num, l: impl Num) -> &mut Self {
        let (hh, ss, ll) = self.get_hsl();
        self.set_hsl(hh + h.f(), ss + s.f(), ll + l.f())
    }
    /// CSS string (`'#rgb'`, `'#rrggbb'`, `'rgb()'`, `'hsl()'`, names) — sRGB, converted to linear.
    pub fn set_style(&mut self, s: &str) -> &mut Self {
        if let Some([r, g, b, _]) = crate::css::parse_css_color(s) {
            self.r = srgb_to_linear(r); self.g = srgb_to_linear(g); self.b = srgb_to_linear(b);
        }
        self
    }
    pub fn copy(&mut self, o: &Color) -> &mut Self { *self = *o; self }
    pub fn lerp(&mut self, o: &Color, t: impl Num) -> &mut Self { let t = t.f(); self.r += (o.r - self.r) * t; self.g += (o.g - self.g) * t; self.b += (o.b - self.b) * t; self }
    pub fn lerp_colors(&mut self, a: &Color, b: &Color, t: impl Num) -> &mut Self { *self = *a; self.lerp(b, t) }
    pub fn multiply_scalar(&mut self, s: impl Num) -> &mut Self { let s = s.f(); self.r *= s; self.g *= s; self.b *= s; self }
    pub fn multiply(&mut self, o: &Color) -> &mut Self { self.r *= o.r; self.g *= o.g; self.b *= o.b; self }
    pub fn add(&mut self, o: &Color) -> &mut Self { self.r += o.r; self.g += o.g; self.b += o.b; self }
    pub fn add_scalar(&mut self, s: impl Num) -> &mut Self { let s = s.f(); self.r += s; self.g += s; self.b += s; self }
    pub fn sub(&mut self, o: &Color) -> &mut Self { self.r = (self.r - o.r).max(0.0); self.g = (self.g - o.g).max(0.0); self.b = (self.b - o.b).max(0.0); self }
    pub fn equals(&self, o: &Color) -> bool { self == o }
    pub fn to_f32(&self) -> [f32; 3] { [self.r as f32, self.g as f32, self.b as f32] }
    /// `getHexString()` (6 lowercase hex digits, sRGB)
    pub fn get_hex_string(&self) -> String { format!("{:06x}", self.get_hex()) }
    pub fn get_style(&self) -> String {
        let c = |v: f64| (clamp(linear_to_srgb(v) * 255.0, 0.0, 255.0)).round() as u32;
        format!("rgb({},{},{})", c(self.r), c(self.g), c(self.b))
    }
}
fn hue2rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * 6.0 * (2.0 / 3.0 - t); }
    p
}

/// Things a JS call site may use as a colour: hex numbers, Colors, CSS strings.
pub trait IntoColor { fn into_color(self) -> Color; }
impl IntoColor for i32 { fn into_color(self) -> Color { Color::hex(self as u32) } }
impl IntoColor for u32 { fn into_color(self) -> Color { Color::hex(self) } }
impl IntoColor for i64 { fn into_color(self) -> Color { Color::hex(self as u32) } }
impl IntoColor for f64 { fn into_color(self) -> Color { Color::hex(self as u32) } }
impl IntoColor for Color { fn into_color(self) -> Color { self } }
impl IntoColor for &Color { fn into_color(self) -> Color { *self } }
impl IntoColor for &str { fn into_color(self) -> Color { let mut c = Color::default(); c.set_style(self); c } }
impl IntoColor for String { fn into_color(self) -> Color { let mut c = Color::default(); c.set_style(&self); c } }
impl IntoColor for &String { fn into_color(self) -> Color { let mut c = Color::default(); c.set_style(self); c } }
pub fn color(c: impl IntoColor) -> Color { c.into_color() }

// ---------------------------------------------------------------- Box3 / Sphere
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Box3 { pub min: Vec3, pub max: Vec3 }
impl Default for Box3 { fn default() -> Self { Box3::empty() } }
impl Box3 {
    pub fn empty() -> Self { Box3 { min: Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY), max: Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY) } }
    pub fn new(min: Vec3, max: Vec3) -> Self { Box3 { min, max } }
    pub fn make_empty(&mut self) -> &mut Self { *self = Box3::empty(); self }
    pub fn is_empty(&self) -> bool { self.max.x < self.min.x || self.max.y < self.min.y || self.max.z < self.min.z }
    pub fn expand_by_point(&mut self, p: &Vec3) -> &mut Self { self.min.min(p); self.max.max(p); self }
    pub fn union(&mut self, b: &Box3) -> &mut Self { self.min.min(&b.min); self.max.max(&b.max); self }
    pub fn get_size(&self) -> Vec3 { if self.is_empty() { Vec3::ZERO } else { self.max - self.min } }
    pub fn get_center(&self) -> Vec3 { if self.is_empty() { Vec3::ZERO } else { (self.min + self.max) * 0.5 } }
    pub fn contains_point(&self, p: &Vec3) -> bool { !(p.x < self.min.x || p.x > self.max.x || p.y < self.min.y || p.y > self.max.y || p.z < self.min.z || p.z > self.max.z) }
    pub fn intersects_box(&self, b: &Box3) -> bool { !(b.max.x < self.min.x || b.min.x > self.max.x || b.max.y < self.min.y || b.min.y > self.max.y || b.max.z < self.min.z || b.min.z > self.max.z) }
    pub fn expand_by_scalar(&mut self, s: f64) -> &mut Self { self.min.add_scalar(-s); self.max.add_scalar(s); self }
    pub fn apply_matrix4(&mut self, m: &Mat4) -> &mut Self {
        if self.is_empty() { return self; }
        let pts = [
            Vec3::new(self.min.x, self.min.y, self.min.z), Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.min.z), Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.min.z), Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.min.z), Vec3::new(self.max.x, self.max.y, self.max.z),
        ];
        self.make_empty();
        for mut p in pts { p.apply_matrix4(m); self.expand_by_point(&p); }
        self
    }
    /// `Box3.setFromObject(object)` (precise = false: geometry boxes × world matrices)
    pub fn set_from_object(&mut self, o: &crate::scene::Obj) -> &mut Self {
        self.make_empty();
        o.update_world_matrix(false, false);
        o.update_matrix_world(false);
        let mut acc = Box3::empty();
        // three's expandByObject: the InstancedMesh's own box, every geometry's
        // box, and the sprites' shared unit plane
        o.traverse(&mut |c| {
            let local = if let Some(b) = c.instanced_bounding_box() { Some(b) }
                else if let Some(g) = c.try_geo() { Some(g.bounding_box()) }
                else if c.is_sprite() { Some(Box3::new(Vec3::new(-0.5, -0.5, 0.0), Vec3::new(0.5, 0.5, 0.0))) }
                else { None };
            if let Some(mut b) = local {
                b.apply_matrix4(&c.matrix_world());
                acc.union(&b);
            }
        });
        *self = acc;
        self
    }
    pub fn get_bounding_sphere(&self) -> Sphere {
        if self.is_empty() { return Sphere { center: Vec3::ZERO, radius: -1.0 }; }
        Sphere { center: self.get_center(), radius: self.get_size().length() * 0.5 }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sphere { pub center: Vec3, pub radius: f64 }
impl Default for Sphere { fn default() -> Self { Sphere { center: Vec3::ZERO, radius: -1.0 } } }
impl Sphere {
    pub fn apply_matrix4(&mut self, m: &Mat4) -> &mut Self { self.center.apply_matrix4(m); self.radius *= m.get_max_scale_on_axis(); self }
    pub fn is_empty(&self) -> bool { self.radius < 0.0 }
    /// three's `Sphere.expandByPoint`
    pub fn expand_by_point(&mut self, p: &Vec3) -> &mut Self {
        if self.is_empty() { self.center = *p; self.radius = 0.0; return self; }
        let v = *p - self.center;
        let l2 = v.length_sq();
        if l2 > self.radius * self.radius {
            let l = l2.sqrt();
            let delta = (l - self.radius) * 0.5;
            self.center = self.center + v * (delta / l);
            self.radius += delta;
        }
        self
    }
    /// three's `Sphere.union`
    pub fn union(&mut self, s: &Sphere) -> &mut Self {
        if s.is_empty() { return self; }
        if self.is_empty() { *self = *s; return self; }
        if self.center == s.center { self.radius = self.radius.max(s.radius); } else {
            let mut d = s.center - self.center; d.set_length(s.radius);
            self.expand_by_point(&(s.center + d));
            self.expand_by_point(&(s.center - d));
        }
        self
    }
}

/// three's `Spherical` (radius, phi from +Y, theta around Y from +Z)
pub fn set_from_spherical_coords(radius: f64, phi: f64, theta: f64) -> Vec3 {
    let s = phi.sin() * radius;
    Vec3::new(s * theta.sin(), phi.cos() * radius, s * theta.cos())
}
