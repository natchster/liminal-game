//! `place_ceiling_speakers`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::shuffle;

// Ceiling speakers: 25% chance per level, 1-2 units flush on the ceiling.
pub(crate) fn place_ceiling_speakers(g: &mut Game, _dist: &[i32]) {
    g.pp.speaker_positions.clear(); // reset per level
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut open = open_tiles(g);
    shuffle(&mut open);
    let grille_mat = lambert!({ color: 0xd8d0c0 });
    let rim_mat = lambert!({ color: 0x7a6a54 });
    let mut i = 0;
    while i < count && i < open.len() {
        let (x, z) = open[i];
        let speaker = group();
        let base = mesh(pg("CeilingSpeakers__trim", cyl_geo!(0.11, 0.11, 0.02, 12)), &rim_mat);
        speaker.add(&base);
        let grille = mesh(pg("CeilingSpeakers__grille", cyl_geo!(0.09, 0.09, 0.005, 12)), &grille_mat);
        grille.position().y = -0.012;
        speaker.add(&grille);
        // Small grille perforations suggested by concentric ring.
        let ring = mesh(ring_geo!(0.045, 0.055, 12), &rim_mat);
        ring.rotation().x = PI / 2.0;
        ring.position().y = -0.0155; // 1 mm under the grille's face (bottom −0.0145)
        speaker.add(&ring);
        let ox = (rnd() - 0.5) * 1.4;
        let oz = (rnd() - 0.5) * 1.4;
        speaker.position().set(g.wx(x) + ox, WALL_H - 0.01, g.wz(z) + oz); // trim top on the ceiling
        g.world.add(&speaker);
        // Track for PA-announcement audio events.
        let sp = speaker.pos();
        g.pp.speaker_positions.push(v3(sp.x, sp.y, sp.z));
        i += 1;
    }
}
