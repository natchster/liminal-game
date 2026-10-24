// sin/cos/atan2/pow bit patterns for JS comparison (V8 uses fdlibm ports; Rust std uses the platform libm)
fn main() {
    let mut s: u64 = 99;
    let mut next = || { s ^= s << 13; s ^= s >> 7; s ^= s << 17; (s >> 11) as f64 / (1u64 << 53) as f64 };
    for _ in 0..100000 {
        let a = (next() - 0.5) * 20.0; let b = next() * 3.0;
        println!("{:e} {:e} {:e} {:e} {:e} {:e} {:e} {:e}", a, b, a.sin(), a.cos(), a.atan2(b), b.powf(2.4), a.tan(), (b * 0.3).asin());
    }
}
