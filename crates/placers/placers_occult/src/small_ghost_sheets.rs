//! `place_small_ghost_sheets`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_tile;
use placers_kit::place::salt;

// Small ghost sheet: 5% chance per level. A cheesecloth ghost (25 cm),
// the tabletop kind: cheesecloth stiffened over a ball on a bottle, a string
// at its neck, the form taken out; two black felt eyes glued on. The cloth
// gone yellow-grey with dust, a glue run dried in it, a tear or two.
pub(crate) fn place_small_ghost_sheets(g: &mut Game, _dist: &[i32]) {
    g.pp.ghost_positions = vec![]; // reset per level
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let turn = rnd() * PI * 2.0;
    // the old sway (60%: its phase and period) is kept as draws: a
    // stiffened cloth stands still
    if rnd() < 0.60 {
        let _ = rnd();
        let _ = rnd();
    }
    let salt = salt(px, pz, 1);
    let ghost = group();
    put(&ghost, "Real__ghostCloth", &lambert!({ map: &cheesecloth_tex(salt), alphaTest: 0.5, side: DoubleSide }));
    put(&ghost, "Real__ghostEyes", &lambert!({ color: 0x1c1a19 }));
    ghost.position().set(px, 0, pz);
    ghost.rotation().y = turn;
    g.world.add(&ghost);
    // Track for whisper audio.
    let gp = ghost.pos();
    g.pp.ghost_positions.push(v3(gp.x, 0.20, gp.z));
    // the old glow's pulse phase (no light: a cloth does not shine)
    let _ = rnd();
}

/// Cheesecloth stiffened and left (laid on the cloth from its sides and
/// from above alike): yellowed grey-white, its loose weave, a run of the
/// stiffener dried yellow in it, dust, grime toward its hem, a tear or two
/// (cut out: alpha tested)
fn cheesecloth_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 512, false, move |ctx, w, h| {
        let seed = salt ^ 0x6c0;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 6.0, v * 6.0, 4);
            let mut c = mix([0.80, 0.77, 0.69], [0.70, 0.67, 0.58], n);
            // the weave: threads across and along, a little uneven
            let wu = ((u * 210.0 + 0.4 * vn(seed ^ 3, v * 30.0, u * 4.0)) * PI).sin().abs();
            let wv = ((v * 170.0 + 0.4 * vn(seed ^ 5, u * 30.0, v * 4.0)) * PI).sin().abs();
            c = mix(c, [0.60, 0.57, 0.49], (1.0 - wu.min(wv)).powi(6) * 0.25);
            // the stiffener's runs, dried yellow
            let run = sstep(0.62, 0.78, fbm(seed ^ 7, u * 3.0, v * 9.0, 4));
            c = mix(c, [0.72, 0.64, 0.44], run * 0.55);
            // grime toward the hem, dust everywhere
            c = mix(c, [0.48, 0.45, 0.39], (1.0 - sstep(0.0, 0.25, v)) * 0.5);
            let dust = vn(seed ^ 9, u * 300.0, v * 300.0);
            if dust > 0.82 {
                c = mix(c, [0.50, 0.48, 0.44], (dust - 0.82) * 3.0);
            }
            // a tear or two, frayed
            let tear = fbm(seed ^ 11, u * 9.0, v * 9.0, 3) + 0.08 * vn(seed ^ 13, u * 120.0, v * 120.0);
            let a = if tear > 0.79 { 0.0 } else { 1.0 };
            [c[0], c[1], c[2], a]
        });
    })
}
