//! The camera's own marks, over the frame: a phone's sensor grain. It
//! belongs to the eye that films, not to the place (the game has its own),
//! so it lives in the set's binary. (Not the engine's colour fringe: it
//! takes red and blue from the frame before the grade and green after it,
//! and parts them into magenta wherever the grade moves bright values.)
//! The grain is a fixed tile of hashed noise moved each frame by a hash of
//! the frame's number: no random stream.
use engine::render::{OBlend, OverlayImage, OverlayOp};

const N: usize = 256;

pub struct Lens {
    grain: Vec<u8>,
    id: u64,
}

fn hash(i: u64) -> u64 {
    let mut h = i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^ (h >> 29)
}

impl Lens {
    pub fn new() -> Lens {
        let mut grain = vec![0u8; N * N * 4];
        for (i, p) in grain.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            // two hashed samples summed: a softer, less salt-and-pepper grain
            let a = (hash(i as u64 * 2 + 1) >> 56) as u32;
            let b = (hash(i as u64 * 2 + 2) >> 56) as u32;
            let v = ((a + b) / 2) as u8;
            *p = [v, v, v, 255];
        }
        Lens { grain, id: 0x6a61_6861_6d75_6e01 }
    }

    /// The marks for frame `frame` of a `w` x `h` view.
    pub fn overlays(&self, frame: u64, w: usize, h: usize) -> Vec<OverlayOp<'_>> {
        let k = hash(frame + 17);
        let off = [((k & 0xffff) as f32) / 65536.0, (((k >> 16) & 0xffff) as f32) / 65536.0];
        vec![
            OverlayOp::Image {
                image: OverlayImage { id: self.id, version: 1, width: N as u32, height: N as u32, premul_rgba: &self.grain },
                rect: [0.0, 0.0, 1.0, 1.0],
                opacity: 0.075,
                blend: OBlend::Overlay,
                tile: [w as f32 / N as f32 / 1.5, h as f32 / N as f32 / 1.5],
                offset: off,
                smooth: true,
            },
        ]
    }
}
