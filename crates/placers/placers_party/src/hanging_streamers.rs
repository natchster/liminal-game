//! `place_hanging_streamers`: Hanging streamers: 8% chance per level. Lengths of crepe paper streamer (1.75 in, Blender batch `t12`) hanging from the ceiling, each from the bit of masking tape that held it when it was a swag across the room: twisted as it was hung, the crepe stretched and sagged, the ends torn; faded, water-marked, dusty. Some still stir in the draught.
use crate::shared::put_with;
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Hanging streamers: 8% chance per level. Lengths of crepe paper
/// streamer (1.75 in, Blender batch `t12`) hanging from the ceiling, each
/// from the bit of masking tape that held it when it was a swag across the
/// room: twisted as it was hung, the crepe stretched and sagged, the ends
/// torn; faded, water-marked, dusty. Some still stir in the draught.
pub(crate) fn place_hanging_streamers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let streamer_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let grp = group();
    let n_streamers = 3 + (rnd() * 4.0).floor() as i32;
    let crepe = crepe_tex();
    let tape = lambert!({ color: 0x8e7b58 });
    for _ in 0..n_streamers {
        // the old draws, in their order: its colour, its length, where under
        // the tape's spot it hangs, its turn, and whether it stirs
        let c = pick(&streamer_colors);
        let k = streamer_colors.iter().position(|&s| s == c).unwrap_or(0);
        let len = 0.9 + rnd() * 0.8;
        let sx = (rnd() - 0.5) * 0.35;
        let sz = (rnd() - 0.5) * 0.35;
        let turn = rnd() * PI * 2.0;
        // the modelled length nearest the old one
        let kind = (0..4).min_by(|&a, &b| (STR_LEN[a] - len).abs().total_cmp(&(STR_LEN[b] - len).abs())).unwrap_or(0);
        let hang = group();
        hang.position().set(sx, WALL_H, sz);
        hang.rotation().y = turn;
        grp.add(&hang);
        let mat = lambert!({ map: &crepe, color: CREPE[k], side: DoubleSide });
        let off = (k as f64) * 0.37 + sx * 3.0;
        put_with(&hang, ["Real__streamerA", "Real__streamerB", "Real__streamerC", "Real__streamerD"][kind], &mat, |gg| {
            // the grain runs across the strip, so along its length (down)
            let n = gg.position.len() / 3;
            for i in 0..n {
                let (x, y, z) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 1]), f64::from(gg.position[i * 3 + 2]));
                gg.uv[i * 2] = ((x + z) * 4.0 + off) as f32;
                gg.uv[i * 2 + 1] = (-y * 2.0 + off) as f32;
            }
        });
        // the tape across its end, under it against the ceiling
        let t = group();
        t.rotation().y = -(STR_HEAD[kind].to_radians() + PI / 2.0);
        t.position().y = -0.0008;
        hang.add(&t);
        if let Some(m) = put(&t, "Real__posterTape", &tape) {
            m.rotation().x = PI / 2.0;
        }
        if rnd() < 0.50 {
            // it stirs, barely, about its tape
            let sway_phase = rnd() * PI * 2.0;
            let sway_period = 6.0 + rnd() * 6.0;
            let s = hang.clone();
            g.animate(move |_g, _dt, t| {
                s.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.012;
            });
        }
    }
    grp.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

// --------------------------------------------------------- the streamers
/// The streamers as modelled (`party/what_a_party_room_leaves.py` STR_KINDS): each one's length, and
/// the way it ran from its tape (deg round y from +x) when it was a swag
const STR_LEN: [f64; 4] = [1.62, 1.36, 1.12, 0.92];

const STR_HEAD: [f64; 4] = [12.0, -20.0, 5.0, -8.0];

/// Crepe paper after years: each old colour faded (crepe's dyes are
/// fugitive)
const CREPE: [u32; 6] = [0xb0707e, 0x6d7fa6, 0xcbb878, 0x7c9a76, 0x8f7495, 0xc597ac];

/// Crepe paper's grain over its colour (multiplied): the creping across
/// the strip, faded and water-marked in patches, dusty (u across, v along:
/// 0.5 m a repeat)
fn crepe_tex() -> Tex {
    canvas_tex(64, 512, |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let y = v * 0.5;
            // the creping: fine ridges, uneven, a shade darker in their folds
            let r = (y * 520.0 + vn(11, u * 4.0, y * 60.0) * 2.0).sin() * 0.5 + 0.5;
            let mut c = 0.90 - 0.07 * r * (0.6 + 0.4 * vn(12, u * 8.0, y * 30.0));
            // faded pale in patches, a tide line where water ran down it
            c += 0.08 * sstep(0.55, 0.85, fbm(13, u * 2.0, y * 6.0, 3));
            let tide = (y * 2.0 + 0.15 * fbm(14, u * 3.0, y * 4.0, 2)).rem_euclid(1.0);
            c -= 0.10 * (1.0 - sstep(0.0, 0.025, (tide - 0.62).abs())) * sstep(0.4, 0.6, vn(15, 1.0, y * 2.0));
            let cc = [c, c * 0.985, c * 0.96];
            let cc = mix(cc, [0.70, 0.67, 0.61], 0.15 * fbm(16, u * 6.0, y * 20.0, 3));
            [cc[0], cc[1], cc[2], 1.0]
        });
    })
}
