//! `rnd()`: the game's random source, with a swappable seeded stream.
//!
//! While a level is built (`with_seed`), `rnd()` draws from a mulberry32
//! generator seeded for that level, so a (run seed, level) pair always
//! rebuilds the same level. `rnd()` draws from the innermost `with_seed`
//! stream if one is active, else from the native generator. Canvas textures
//! and other cached, history-dependent work draw with `with_native` so they
//! never shift the level's stream.

use std::cell::RefCell;

/// mulberry32: the seeded level-build generator. Its output sequence is
/// fixed bit for bit (the recorded fixtures depend on it).
#[derive(Clone, Copy, Debug)]
pub struct Mulberry32 {
    t: u32,
}
impl Mulberry32 {
    pub fn new(seed: u32) -> Self {
        Mulberry32 { t: seed }
    }
    #[inline]
    pub fn next(&mut self) -> f64 {
        self.t = self.t.wrapping_add(0x6D2B79F5);
        let mut x = self.t;
        x = (x ^ (x >> 15)).wrapping_mul(x | 1);
        x ^= x.wrapping_add((x ^ (x >> 7)).wrapping_mul(x | 61));
        ((x ^ (x >> 14)) as f64) / 4294967296.0
    }
}

struct Xorshift {
    s: [u64; 2],
}
impl Xorshift {
    fn next(&mut self) -> f64 {
        let mut s1 = self.s[0];
        let s0 = self.s[1];
        self.s[0] = s0;
        s1 ^= s1 << 23;
        s1 ^= s1 >> 17;
        s1 ^= s0;
        s1 ^= s0 >> 26;
        self.s[1] = s1;
        ((self.s[0].wrapping_add(self.s[1])) >> 11) as f64 / (1u64 << 53) as f64
    }
}

struct RngState {
    stack: Vec<Option<Mulberry32>>,
    native: Xorshift,
    trace: u64,
}

thread_local! {
    static RNG: RefCell<RngState> = RefCell::new(RngState {
        stack: vec![],
        native: {
            let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x1234_5678);
            let mut x = Xorshift { s: [t ^ 0x9E37_79B9_7F4A_7C15, t.rotate_left(29) ^ 0xD1B5_4A32_D192_ED03] };
            for _ in 0..8 { x.next(); }
            x
        },
        trace: 0,
    });
}

#[inline]
pub fn rnd() -> f64 {
    RNG.with(|r| {
        let mut r = r.borrow_mut();
        let r = &mut *r;
        match r.stack.last_mut() {
            Some(Some(m)) => {
                r.trace += 1;
                m.next()
            }
            _ => r.native.next(),
        }
    })
}
/// The native generator, whatever stream is active.
pub fn native_rnd() -> f64 {
    RNG.with(|r| r.borrow_mut().native.next())
}

struct Pop;
impl Drop for Pop {
    fn drop(&mut self) {
        RNG.with(|r| {
            r.borrow_mut().stack.pop();
        });
    }
}

/// Run `f` with `rnd()` drawing from mulberry32(seed).
pub fn with_seed<R>(seed: u32, f: impl FnOnce() -> R) -> R {
    RNG.with(|r| r.borrow_mut().stack.push(Some(Mulberry32::new(seed))));
    let _g = Pop;
    f()
}
/// Run `f` with `rnd()` back on the native generator (canvas textures, caches).
pub fn with_native<R>(f: impl FnOnce() -> R) -> R {
    RNG.with(|r| r.borrow_mut().stack.push(None));
    let _g = Pop;
    f()
}
/// Number of seeded draws so far (for the rand trace).
pub fn seeded_draws() -> u64 {
    RNG.with(|r| r.borrow().trace)
}

/// `rnd() * a`
#[inline]
pub fn rand1(a: f64) -> f64 {
    rnd() * a
}
/// `a + rnd() * (b - a)`
#[inline]
pub fn rand2(a: f64, b: f64) -> f64 {
    a + rnd() * (b - a)
}
/// A random integer in `a..=b`: `floor(rand2(a, b + 1))`
#[inline]
pub fn irand(a: i64, b: i64) -> i64 {
    (a as f64 + rnd() * ((b + 1) as f64 - a as f64)).floor() as i64
}
/// A random element of `a`
#[inline]
pub fn choice<T: Clone>(a: &[T]) -> T {
    a[(rnd() * a.len() as f64).floor() as usize].clone()
}
/// A random index `floor(rnd() * len)`, clamped to the last index
#[inline]
pub fn pick_index(len: usize) -> usize {
    ((rnd() * len as f64).floor() as usize).min(len.saturating_sub(1))
}

/// splitmix64: a small local stream for per-level choices that must be
/// stable per (run seed, level) without drawing from `rnd()` (seed it from
/// `current_seed` and a salt of its own).
#[derive(Clone, Copy, Debug)]
pub struct SplitMix64 {
    s: u64,
}
impl SplitMix64 {
    pub fn new(seed: u32, salt: u64) -> Self {
        SplitMix64 { s: seed as u64 ^ salt }
    }
    /// Uniform in [0, 1)
    pub fn next(&mut self) -> f64 {
        self.s = self.s.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.s;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (z >> 11) as f64 / (1u64 << 53) as f64
    }
    /// Uniform in [a, b)
    pub fn range(&mut self, a: f64, b: f64) -> f64 {
        a + self.next() * (b - a)
    }
}

pub fn seed_for_level(rs: u32, n: i32) -> u32 {
    rs.wrapping_add((n as u32).wrapping_mul(0x9E3779B9))
}
