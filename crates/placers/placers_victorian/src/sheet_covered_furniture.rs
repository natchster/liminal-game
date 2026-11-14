//! `place_sheet_covered_furniture`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Dust-sheet-covered furniture (floor): 3.5% chance per level. Each
// piece is one sheet of the Watcher's linen thrown over a chair, a table or
// a wingback (`bodies.bin`'s `sheet_*`, `assets/src/creatures/sheeted.py`):
// at a glance one is the other. (The draws of the old box-built shapes are
// kept, in their order, so the placers after this one draw what they did.)
pub(crate) fn place_sheet_covered_furniture(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.965 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let _sheet_colour = ri(4);
    let variant = (rnd() * 3.0).floor() as i32;
    match variant {
        0 => {
            // (three folds: where and how turned)
            for _ in 0..9 {
                rnd();
            }
        }
        1 => {
            // (a fold's place, and 30%: a bump's)
            rnd();
            if rnd() < 0.3 {
                rnd();
                rnd();
            }
        }
        _ => {
            // (four folds)
            for _ in 0..12 {
                rnd();
            }
        }
    }
    let scf = group();
    let name = ["sheet_chair", "sheet_table", "sheet_wingback"][variant.clamp(0, 2) as usize];
    if let Some(rig) = game::entities::body::build_body(name, &game::entities::watcher::material) {
        scf.add(&rig.root);
    }
    // 40% chance: thin dust layer on floor around base.
    if rnd() < 0.4 {
        let dust_mat = lambert!({ color: 0xa89880, transparent: true, opacity: 0.35 });
        let dust = mesh(circle_geo!(0.55, 12), &dust_mat);
        dust.rotation().x = -PI / 2.0;
        dust.position().y = 0.001;
        scf.add(&dust);
    }
    scf.position().set(g.wx(x) + (rnd() - 0.5) * 0.8, 0, g.wz(z) + (rnd() - 0.5) * 0.8);
    scf.rotation().y = rnd() * PI * 2.0;
    g.world.add(&scf);
    // The wingback's sheet breathes, very slightly (as it did)
    if variant >= 2 {
        let breath_freq = (PI * 2.0) / (30.0 + rnd() * 15.0);
        let breath_phase = rnd() * PI * 2.0;
        let sheet = scf.children().into_iter().next();
        if let Some(sheet) = sheet {
            g.animate(move |_g, _dt, t| {
                sheet.position().y = (t * breath_freq + breath_phase).sin() * 0.0025;
            });
        }
    }
}
