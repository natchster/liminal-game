//! Batch `t22`'s paper clips, made real (`docs/PROPS.md` P3; the Blender
//! batch `assets/src/blender/props/t22.py`, its notes `t22.notes.md`): a
//! spill of Gem clips, bent wire, as a box's worth lies where it fell. It
//! keeps the old placer's draws, call for call.
use crate::{open_tiles, scatter};
use game::prelude::*;

// R548 paperclip cluster: 10% chance per level. 6-14 Gem clips spilled
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
        clip.position().set(px, 0.0, pz);
        clip.rotation().y = yaw;
        grp.add(&clip);
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}
