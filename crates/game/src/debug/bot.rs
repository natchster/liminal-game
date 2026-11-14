//! Scripted players for tests and the bench.
use crate::replay::{FrameInput, Quantizer};

/// A restless player: mostly walking, turning, now and then running,
/// crouching or switching the flashlight (a fixed pseudo-random script, its
/// look rounded as the window rounds a mouse). The tests' bot, and the
/// bench's (`--bench-replay`).
pub fn restless(frames: usize, seed: u32) -> Vec<FrameInput> {
    let mut s = u64::from(seed) | 1;
    let mut next = move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s % 1000) as f64 / 1000.0
    };
    let mut out = vec![FrameInput::default(); frames];
    let mut held: Vec<&str> = vec![];
    let mut q = Quantizer::default();
    for (k, f) in out.iter_mut().enumerate() {
        if k % 30 == 0 {
            for h in held.drain(..) {
                f.up(h);
            }
            let r = next();
            let keys: &[&str] = if r < 0.55 {
                &["KeyW"]
            } else if r < 0.7 {
                &["KeyW", "ShiftLeft"]
            } else if r < 0.8 {
                &["KeyA"]
            } else if r < 0.88 {
                &["KeyS"]
            } else {
                &["KeyW", "ControlLeft"]
            };
            for key in keys {
                f.down(key);
                held.push(key);
            }
            if next() < 0.15 {
                f.down("KeyF");
                f.up("KeyF");
            }
        }
        // rounded where it enters, as the window rounds a mouse (`Quantizer`)
        let (dx, dy) = q.look((next() - 0.5) * 18.0, (next() - 0.5) * 2.0);
        f.look(dx, dy);
    }
    out
}
