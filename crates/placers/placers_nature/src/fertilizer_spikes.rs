//! `place_fertilizer_spikes`
use crate::shared::fbm_tile;
use game::assets::props::prop_part_as_modelled;
use game::assets::props::prop_part_info;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use game::visuals::pictures::Window;
use placers_kit::garden::put_uv;
use placers_kit::noise::mix;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::paint::rgb as hex;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Fertilizer spike cluster: 8% chance per level.
pub(crate) fn place_fertilizer_spikes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Houseplant fertilizer spikes out of one pack (Blender batch `t06`):
    // the first spike's colour draw chooses the pack's (the white houseplant
    // spikes, a pale grey-green, a pinkish grey, a cream); each later
    // spike's colour draw now chooses whether it lies whole, snapped in two
    // or crumbled
    let spike_colors: [u32; 4] = [0x408040, 0x2060c0, 0xc02020, 0xf0e020];
    let cluster = group();
    let count = 3 + (rnd() * 3.0).floor() as i32;
    let mut pack: Option<Mat> = None;
    let mut down: Vec<Laid> = vec![];
    for i in 0..count {
        let c = pick(&spike_colors);
        let kind = spike_colors.iter().position(|&k| k == c).unwrap_or(0);
        let mat = pack
            .get_or_insert_with(|| {
                let tone = match c {
                    0x408040 => 0xa4ac94,
                    0x2060c0 => 0xd0ccc0,
                    0xc02020 => 0xa89086,
                    _ => 0xc8bea6,
                };
                lambert!({ map: &spike_tex(hex(tone)) })
            })
            .clone();
        let whole = i == 0 || kind < 2;
        // where it lies and how it is turned (the old draws)
        let (sx, sz) = ((rnd() - 0.5) * 0.12, (rnd() - 0.5) * 0.12);
        let turn = rnd() * PI * 2.0;
        // its length along the floor (the spike's middle at the group's
        // origin): the head end, the far end of what lies whole
        let (x0, x1) = (-SPIKE_MID, if whole { SPIKE_L - SPIKE_MID } else { SPIKE_SNAP - SPIKE_MID });
        let dir = (turn.cos(), -turn.sin());
        let a0 = (sx + dir.0 * x0, sz + dir.1 * x0);
        let a1 = (sx + dir.0 * x1, sz + dir.1 * x1);
        // where it crosses one already down it rests on it: it pivots on its
        // end farther from the crossing, raised over the other's top there
        let mut best: Option<(f64, bool)> = None; // (sin of its tilt, pivot at the head)
        for d in &down {
            if let Some((t, u)) = seg_cross(a0, a1, d.a0, d.a1) {
                let len = x1 - x0;
                let at = t * len; // from the head end
                let top = d.top(u);
                let (dist, head) = if at > len / 2.0 { (at, true) } else { (len - at, false) };
                let k = (top / dist.max(0.004)).min(0.42);
                if best.is_none_or(|(b, _)| k > b) {
                    best = Some((k, head));
                }
            }
        }
        let spike = group();
        spike.position().set(sx, 0, sz);
        spike.rotation().y = turn;
        let tilt = group();
        spike.add(&tilt);
        let (sin, head) = best.unwrap_or((0.0, true));
        let pivot = if head { x0 } else { x1 };
        tilt.position().x = pivot;
        tilt.rotation().z = if head { sin.asin() } else { -sin.asin() };
        let inner = group();
        inner.position().x = -SPIKE_MID - pivot;
        tilt.add(&inner);
        // what broke off lies on the floor
        let apart = group();
        apart.position().x = -SPIKE_MID;
        spike.add(&apart);
        let size = 0.02;
        if whole {
            put_uv(&inner, "Real__fertSpike", &mat, world(size));
        } else if kind == 2 {
            put_uv(&inner, "Real__fertSpikeHead", &mat, world(size));
            put_uv(&apart, "Real__fertSpikeTip", &mat, world(size));
        } else {
            put_uv(&inner, "Real__fertSpikeHead", &mat, world(size));
            put_uv(&apart, "Real__fertCrumbs", &mat, world(size));
        }
        down.push(Laid { a0, a1, pivot: if head { a0 } else { a1 }, sin, len: x1 - x0 });
        cluster.add(&spike);
    }
    cluster.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&cluster);
}

/// Laid on at its real scale (`size` m a repeat) from the side each vertex
/// faces most
fn world(size: f64) -> impl Fn(f64, f64, f64, [f64; 3]) -> (f64, f64) {
    move |x, y, z, n| {
        let (ax, ay, az) = (n[0].abs(), n[1].abs(), n[2].abs());
        let (u, v) = if ay >= ax && ay >= az {
            (x, z)
        } else if ax >= az {
            (z, y)
        } else {
            (x, y)
        };
        (u / size, v / size)
    }
}

// ======================================================= fertilizer spikes
/// A spike's compressed granules: chalky, the granules showing, crumbled
/// darker where damp got at it (a repeat every 2 cm)
fn spike_tex(base: [f64; 3]) -> Tex {
    canvas_tex(64, 64, move |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let n = fbm_tile(0x5b1e, u * 8.0, v * 8.0, 8.0, 3);
            let mut c = base.map(|k| k * (0.84 + 0.3 * n));
            let gr = (vn(0x5b1f, u * 32.0, v * 32.0) - 0.6).max(0.0) * 1.5;
            c = mix(c, c.map(|k| k * 0.7), gr);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The spike as modelled (`t06.py` SPIKE_L): its length, where its middle
/// is from its head, where a snapped one broke (0.58 of it)
const SPIKE_L: f64 = 0.052;

const SPIKE_MID: f64 = 0.0275;

const SPIKE_SNAP: f64 = 0.58 * SPIKE_L;

/// A spike (or what is left of it whole) lying in the cluster: its head
/// end and far end on the floor (x, z), the end it pivots on, the sine of
/// its tilt, its length
struct Laid {
    a0: (f64, f64),
    a1: (f64, f64),
    pivot: (f64, f64),
    sin: f64,
    len: f64,
}

impl Laid {
    /// The height of its top at `u` (0 the head end, 1 the far end): its
    /// thickness there (the octagon across its flats, tapering to the
    /// point) over its lift off the floor
    fn top(&self, u: f64) -> f64 {
        let t = u * self.len / SPIKE_L;
        let thick = if t < 0.58 { 0.0079 - 0.0009 * t } else { 0.0074 * (1.0 - 0.72 * ((t - 0.58) / 0.42).powf(1.15)) };
        let p = (self.a0.0 + (self.a1.0 - self.a0.0) * u, self.a0.1 + (self.a1.1 - self.a0.1) * u);
        let lift = (p.0 - self.pivot.0).hypot(p.1 - self.pivot.1) * self.sin;
        lift + thick
    }
}

/// Where segments a0-a1 and b0-b1 cross: the fraction along each
fn seg_cross(a0: (f64, f64), a1: (f64, f64), b0: (f64, f64), b1: (f64, f64)) -> Option<(f64, f64)> {
    let (rx, ry) = (a1.0 - a0.0, a1.1 - a0.1);
    let (sx, sy) = (b1.0 - b0.0, b1.1 - b0.1);
    let den = rx * sy - ry * sx;
    if den.abs() < 1e-12 {
        return None;
    }
    let (qx, qy) = (b0.0 - a0.0, b0.1 - a0.1);
    let t = (qx * sy - qy * sx) / den;
    let u = (qx * ry - qy * rx) / den;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then_some((t, u))
}
