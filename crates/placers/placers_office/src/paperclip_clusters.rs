//! `place_paperclip_clusters`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Paperclip cluster: 10% chance per level. 6-14 Gem clips spilled
// from a box: No. 1 clips of plated steel wire (35 mm, three loops of one
// wire), one or two pulled open into an S by a fidgeting hand, now and
// then a jumbo; most still bright, some dulled, a few rusted through
// their plating.
pub(crate) fn place_paperclip_clusters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let props = game::assets::props::prop_part_as_modelled;
    let (Some(gem), Some(pulled), Some(jumbo)) = (props("Real__paperclipGem"), props("Real__paperclipOpen"), props("Real__paperclipJumbo"))
    else {
        return;
    };
    // plated steel wire: bright, dulled by years of air, rusted through
    let finishes = [
        phong!({ color: 0x8a8a8c, specular: 0x5a5a5a, shininess: 70 }),
        phong!({ color: 0x67665f, specular: 0x262626, shininess: 30 }),
        lambert!({ color: 0x6a4228 }),
    ];
    let grp = group();
    let n_clips = 6 + (rnd() * 9.0).floor() as i32;
    // Two by two, a clip and the next are the same clip with the same
    // finish (the odd last one too): none is the only one of its kind, so
    // every clip is drawn instanced, as the old ones were
    let mut kind: Option<(usize, usize)> = None;
    // each clip down so far: its middle, its turn, its half length and width,
    // and how high it lies
    let mut down: Vec<(f64, f64, f64, f64, f64, f64)> = vec![];
    for i in 0..n_clips {
        let yaw = rnd() * PI * 2.0;
        let px = (rnd() - 0.5) * 0.15;
        let pz = (rnd() - 0.5) * 0.12;
        // the old two scale draws: which clip, and its finish (the second
        // of two takes the first's)
        let shape = rnd();
        let finish = rnd();
        if (i % 2 == 0 && i + 1 < n_clips) || kind.is_none() {
            let k = if shape < 0.78 {
                0
            } else if shape < 0.92 {
                1
            } else {
                2
            };
            let f = if finish < 0.62 {
                0
            } else if finish < 0.90 {
                1
            } else {
                2
            };
            kind = Some((k, f));
        }
        let (k, f) = kind.unwrap_or((0, 0));
        let clip = mesh([&gem, &pulled, &jumbo][k].clone(), &finishes[f]);
        // a clip falling on others lies on them, a wire's thickness up for
        // each it lies across (all on the floor, they went through one
        // another in the heap: M8)
        let (hl, hw) = [(0.0175, 0.0047), (0.0144, 0.0131), (0.024, 0.0063)][k];
        let y = down
            .iter()
            .filter(|q| rects_overlap((px, pz, yaw, hl, hw), (q.0, q.1, q.2, q.3, q.4)))
            .map(|q| q.5 + CLIP_WIRE)
            .fold(0.0, f64::max);
        down.push((px, pz, yaw, hl, hw, y));
        clip.position().set(px, y, pz);
        clip.rotation().y = yaw;
        grp.add(&clip);
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

/// A clip's wire, across (`party/small_everyday_things.py` CLIP_WIRE_R)
const CLIP_WIRE: f64 = 0.0011;

/// Whether two clips' outlines overlap: each its middle (x, z), its turn,
/// its half length and half width (separating axes of two turned rects)
fn rects_overlap(a: (f64, f64, f64, f64, f64), b: (f64, f64, f64, f64, f64)) -> bool {
    let axes = |r: (f64, f64, f64, f64, f64)| {
        let (s, c) = r.2.sin_cos();
        [(c, -s), (s, c)]
    };
    let reach = |r: (f64, f64, f64, f64, f64), ax: (f64, f64)| {
        let [u, v] = axes(r);
        r.3 * (u.0 * ax.0 + u.1 * ax.1).abs() + r.4 * (v.0 * ax.0 + v.1 * ax.1).abs()
    };
    let (dx, dz) = (b.0 - a.0, b.1 - a.1);
    axes(a).into_iter().chain(axes(b)).all(|ax| (dx * ax.0 + dz * ax.1).abs() <= reach(a, ax) + reach(b, ax))
}
