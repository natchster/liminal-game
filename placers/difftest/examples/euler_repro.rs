// Euler(x, y, 0) -> quaternion -> Euler (what Object3D.clone() does), engine version
use engine::math::*;
fn main() {
    let mut s: u64 = 12345;
    let mut next = || { s ^= s << 13; s ^= s >> 7; s ^= s << 17; (s >> 11) as f64 / (1u64 << 53) as f64 };
    for _ in 0..200000 {
        let x = (next() - 0.5) * 0.15; let y = next() * std::f64::consts::PI * 2.0;
        let q = Quat::from_euler(&Euler::new(x, y, 0.0));
        let mut e = Euler::new(0.0, 0.0, 0.0); e.set_from_quaternion(&q, EulerOrder::XYZ);
        println!("{:e} {:e} {:e} {:e} {:e} {:e} {:e} {:e}", x, y, q.x, q.y, q.z, q.w, e.x, e.z);
    }
}
