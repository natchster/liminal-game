//! `place_twigs`
use crate::shared::bark_tex;
use game::assets::props::prop_part_as_modelled;
use game::assets::props::prop_part_info;
use game::prelude::*;
use placers_kit::geometry::frame;
use placers_kit::geometry::part;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Small twigs: 7% chance per level. Oak twiglets the tree shed (Blender
// batch `t05`): grey-brown bark with its lenticels, the buds at the nodes
// and the tip's cluster, a side shoot, a snapped end's pale wood; one lying
// across another rests on it.
pub(crate) fn place_twigs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let twig_colors = [0x503020, 0x604030, 0x402010, 0x705040];
    // the bark's tint by the old colour: weathered grey-brown, brown, dark
    // with damp, bleached
    const TINT: [u32; 4] = [0xc4bcb0, 0xd2b8a0, 0x9a8e80, 0xe2dcd2];
    let cluster = group();
    let count = 3 + (rnd() * 4.0).floor() as i32;
    let bark = bark_tex();
    let wood = lambert!({ color: 0x9c8a6c });
    let mut mats: [Option<Mat>; 4] = Default::default();
    let mut laid: Vec<Laid> = vec![];
    for _i in 0..count {
        // the old twig's draws: its colour (the bark's tint), its length
        // (which twiglet, and its size within 12%), its radius (kept for its
        // draw), its turn, where it lies, its roll (kept for its draw)
        let col = choice(&twig_colors);
        let k = twig_colors.iter().position(|&c| c == col).unwrap_or(0);
        let len = 0.040 + rnd() * 0.060;
        let _ = rnd();
        let yaw = rnd() * PI * 2.0;
        let (tx, tz) = ((rnd() - 0.5) * 0.16, (rnd() - 0.5) * 0.16);
        let _ = rnd();
        let kind = (((len - 0.040) / 0.060 * 6.0).floor() as usize).min(5);
        let (name, wood_name, plen) = TWIGS[kind];
        let Some(geo) = part(name) else { continue };
        let s = (len / plen).clamp(0.88, 1.12);
        let (lo, hi) = frame(name);
        let me = Laid { c: (tx, tz), d: (yaw.cos(), -yaw.sin()), hl: (hi[0] - lo[0]) / 2.0 * s, top: hi[1] * s, tilt: None };
        // across one already down: it rests on it, its far end on the floor
        let mut tilt: Option<(f64, f64)> = None;
        for o in &laid {
            if let Some(p) = cross2(me.end(-1.0), me.end(1.0), o.end(-1.0), o.end(1.0)) {
                let h = o.height_at(p);
                let (dm, dp) = {
                    let (a, b) = (me.end(-1.0), me.end(1.0));
                    ((p.0 - a.0).hypot(p.1 - a.1), (p.0 - b.0).hypot(p.1 - b.1))
                };
                let (sign, d) = if dp > dm { (1.0, dp) } else { (-1.0, dm) };
                let th = (h / d.max(0.01)).atan().min(0.5);
                if tilt.is_none_or(|(_, t)| th > t) {
                    tilt = Some((sign, th));
                }
            }
        }
        let mat = mats[k].get_or_insert_with(|| lambert!({ map: &bark, color: TINT[k] })).clone();
        let t = group();
        let m = mesh(geo, &mat);
        m.scale().set_scalar(s);
        t.add(&m);
        if let Some(wg) = wood_name.and_then(part) {
            let w = mesh(wg, &wood);
            w.scale().set_scalar(s);
            t.add(&w);
        }
        match tilt {
            None => {
                t.position().set(tx, 0, tz);
                t.rotation().y = yaw;
                cluster.add(&t);
            }
            Some((sign, th)) => {
                let e = me.end(sign);
                let pivot = group();
                pivot.position().set(e.0, 0, e.1);
                pivot.rotation().y = yaw;
                let lean = group();
                lean.rotation().z = -sign * th;
                t.position().x = -sign * me.hl;
                lean.add(&t);
                pivot.add(&lean);
                cluster.add(&pivot);
            }
        }
        laid.push(Laid { tilt, ..me });
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// ================================================================== twigs
/// The twiglets (`t05.py` TWIGS): the part, its snapped end's wood if it has
/// one, its length as modelled
const TWIGS: [(&str, Option<&str>, f64); 6] = [
    ("Real__twigA", Some("Real__twigWoodA"), 0.046),
    ("Real__twigB", None, 0.056),
    ("Real__twigC", Some("Real__twigWoodC"), 0.066),
    ("Real__twigD", None, 0.076),
    ("Real__twigE", None, 0.086),
    ("Real__twigF", Some("Real__twigWoodF"), 0.098),
];

/// A twig laid down: where it lies, its turn, its half length, how high its
/// top is, and (when it rests across another) its pivot end and its tilt
struct Laid {
    c: (f64, f64),
    d: (f64, f64),
    hl: f64,
    top: f64,
    tilt: Option<(f64, f64)>,
}

impl Laid {
    fn end(&self, sign: f64) -> (f64, f64) {
        (self.c.0 + sign * self.hl * self.d.0, self.c.1 + sign * self.hl * self.d.1)
    }
    /// How high its top is over the floor at (x, z) on it
    fn height_at(&self, p: (f64, f64)) -> f64 {
        match self.tilt {
            None => self.top,
            Some((sign, th)) => {
                let e = self.end(sign);
                self.top + (p.0 - e.0).hypot(p.1 - e.1) * th.tan()
            }
        }
    }
}

/// Where segments a0-a1 and b0-b1 cross, if they do
fn cross2(a0: (f64, f64), a1: (f64, f64), b0: (f64, f64), b1: (f64, f64)) -> Option<(f64, f64)> {
    let (rx, rz) = (a1.0 - a0.0, a1.1 - a0.1);
    let (sx, sz) = (b1.0 - b0.0, b1.1 - b0.1);
    let den = rx * sz - rz * sx;
    if den.abs() < 1e-12 {
        return None;
    }
    let t = ((b0.0 - a0.0) * sz - (b0.1 - a0.1) * sx) / den;
    let u = ((b0.0 - a0.0) * rz - (b0.1 - a0.1) * rx) / den;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then(|| (a0.0 + rx * t, a0.1 + rz * t))
}
