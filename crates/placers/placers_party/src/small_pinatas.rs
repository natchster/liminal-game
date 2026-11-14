//! `place_small_pinatas`: Small piñata: 5% chance per level. A small star piñata (Blender batch `t12`): a papier-mâché ball and seven card cones under rows of fringed tissue, tissue streamers at the tips, hung by its cotton cord from a cup hook in the ceiling; split open on one side where it was hit (empty, its torn edge of newsprint, the dark inside), a cone crushed; the tissue faded, the fringe lifted and torn. It turns, barely, on its cord.
use crate::shared::TISSUE;
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small piñata: 5% chance per level. A small star piñata (Blender
/// batch `t12`): a papier-mâché ball and seven card cones under rows of
/// fringed tissue, tissue streamers at the tips, hung by its cotton cord
/// from a cup hook in the ceiling; split open on one side where it was hit
/// (empty, its torn edge of newsprint, the dark inside), a cone crushed;
/// the tissue faded, the fringe lifted and torn. It turns, barely, on its
/// cord.
pub(crate) fn place_small_pinatas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old body and accent colours: the ball's tissue and the cones'
    let pin_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let cb = pick(&pin_colors);
    let kb = pin_colors.iter().position(|&c| c == cb).unwrap_or(0);
    let ca = pick(&pin_colors);
    let ka = pin_colors.iter().position(|&c| c == ca).unwrap_or(0);
    let pinata = group();
    // the old string's length: the cord nearest it
    let string_len = 0.4 + rnd() * 0.4;
    let kind = (0..3).min_by(|&a, &b| (PN_CORD[a] - string_len).abs().total_cmp(&(PN_CORD[b] - string_len).abs())).unwrap_or(0);
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    pinata.position().set(px, 0, pz);
    pinata.rotation().y = rnd() * PI * 2.0;
    // it hangs from the hook: what sways turns about it
    let hang = group();
    hang.position().y = WALL_H;
    pinata.add(&hang);
    put(&hang, "Real__pinataHook", &phong!({ color: 0x5a4a2c, specular: 0x7a6a44, shininess: 40 }));
    put(&hang, ["Real__pinataCordS", "Real__pinataCordM", "Real__pinataCordL"][kind], &lambert!({ color: 0xa69c86 }));
    let body = group();
    body.position().y = -PN_CORD[kind];
    hang.add(&body);
    let body_tissue = lambert!({ color: TISSUE[kb], side: DoubleSide });
    put(&body, "Real__pinataFringeBody", &body_tissue);
    put(&body, "Real__pinataTails", &body_tissue);
    let cone_tissue = lambert!({ color: TISSUE[ka], side: DoubleSide });
    put(&body, "Real__pinataFringeCones", &cone_tissue);
    put(&body, "Real__pinataTailsB", &cone_tissue);
    // the split: newsprint papier-mâché torn, the hollow dark
    put(&body, "Real__pinataBroken", &lambert!({ color: 0x8a857a }));
    put(&body, "Real__pinataInside", &lambert!({ color: 0x24211c }));
    g.world.add(&pinata);
    // the old sway: a slow turn on its cord
    if rnd() < 0.60 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 10.0 + rnd() * 8.0;
        let p = hang.clone();
        g.animate(move |_g, _dt, t| {
            p.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.012;
        });
    }
}

// ------------------------------------------------------------- the piñata
/// The piñata's cords as modelled (`party/what_a_party_room_leaves.py` PN_CORDS): how far below its
/// hook the ball's tie hangs
const PN_CORD: [f64; 3] = [0.45, 0.60, 0.75];
