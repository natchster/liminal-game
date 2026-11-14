//! `place_post_it_stacks`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Post-it stack: 8% chance per level. 3-5 stacked
// colored sticky notes.
pub(crate) fn place_post_it_stacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let note_colors = [0xf0e050, 0xf0a0c0, 0xa0f0a0, 0xa0d0f0];
    let ink_mat = lambert!({ color: 0x181410 });
    let grp = group();
    // Base stack.
    let n_stack = 3 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_stack {
        let note_mat = lambert!({ color: choice(&note_colors) });
        let note = mesh(pg("PostItStacks__note", box_geo!(0.06, 0.001, 0.06)), &note_mat);
        let px = (rnd() - 0.5) * 0.002;
        let py = 0.0006 + i as f64 * 0.0012;
        let pz = (rnd() - 0.5) * 0.002;
        note.position().set(px, py, pz);
        note.rotation().y = (rnd() - 0.5) * 0.10;
        grp.add(&note);
    }
    // 1-2 loose notes scattered nearby w/ scribbles.
    for i in 0..2 {
        let note_mat = lambert!({ color: choice(&note_colors) });
        let note = mesh(pg("PostItStacks__loose", box_geo!(0.06, 0.001, 0.06)), &note_mat);
        let px = 0.08 + (rnd() - 0.5) * 0.10;
        let pz = (rnd() - 0.5) * 0.14;
        note.position().set(px, 0.001, pz);
        note.rotation().y = rnd() * PI * 2.0;
        grp.add(&note);
        // Ink scribble line on note.
        let scribble = mesh(plane_geo!(0.040, 0.002), &ink_mat);
        scribble.rotation().x = -PI / 2.0;
        let np = note.pos();
        scribble.position().set(np.x, 0.002, np.z);
        scribble.rotation().z = note.rot().y;
        grp.add(&scribble);
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}
