//! Batch `t06`'s prop in this crate, made real (`docs/PROPS.md` P3): the
//! bundle of herbs hung to dry (Blender batch `t06`; what the real one is
//! and where its size comes from: `assets/src/blender/props/t06.notes.md`).
//! It keeps its old draws call for call (the comments at each say what a
//! draw now chooses).
use crate::{face_wall, pick, wall_tiles};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;

/// The binding's middle on the wall frame (`t06.py` HB_TIE)
const HB_TIE: (f64, f64, f64) = (0.0, 1.874, 0.026);

fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = prop_part_as_modelled(name)?;
    let m = mesh(geo, mat);
    grp.add(&m);
    Some(m)
}

/// The herbs: sage, rosemary or lavender; each its parts (`t06.py`) and its
/// colours dried
#[derive(Clone, Copy)]
enum Herb {
    Sage(u32),
    Rosemary(u32, u32),
    Lavender(u32, u32),
}

// R662 hanging herbs bundle: 5% chance per level.
pub(crate) fn place_herb_bundles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    // Herbs hung head down from a brass cup hook to dry and left there for
    // years (Blender batch `t06`): five to eight stems tied at their cut ends
    // with cotton string, fanned, dry and dusty. The old colour's draw
    // chooses the herb: garden sage, rosemary, lavender, an older rosemary,
    // a sage gone yellow
    let herb_colors: [u32; 5] = [0x506030, 0x405020, 0x607040, 0x304018, 0x807040];
    let bundle = group();
    let body = group();
    // the wall is 0.008 behind the old group's origin
    body.position().z = -0.008;
    bundle.add(&body);
    put(&body, "Real__herbHook", &phong!({ color: 0x86703e, specular: 0x403420, shininess: 30 }));
    put(&body, "Real__herbTwine", &lambert!({ color: 0xb2a890 }));
    let stem_count = 5 + (rnd() * 4.0).floor() as i32;
    let herb = match pick(&herb_colors) {
        0x506030 => Herb::Sage(0x5e6650),
        0x405020 => Herb::Rosemary(0x5e4c3a, 0x55593c),
        0x607040 => Herb::Lavender(0x6c6a4e, 0x5e5668),
        0x304018 => Herb::Rosemary(0x564434, 0x4a4c34),
        _ => Herb::Sage(0x6e6c52),
    };
    let n = stem_count as f64;
    // each stem dried a little differently: every other one darker
    let shade = |c: u32, k: f64| {
        let [r, gg, b] = [(c >> 16) & 255, (c >> 8) & 255, c & 255].map(|v| (f64::from(v) * k).round() as u32);
        (r << 16) | (gg << 8) | b
    };
    let mats_for = |k: f64| -> (Mat, Mat) {
        match herb {
            Herb::Sage(c) => (lambert!({ color: shade(c, k), side: DoubleSide }), lambert!({ color: shade(c, k), side: DoubleSide })),
            Herb::Rosemary(w, l) | Herb::Lavender(w, l) => (lambert!({ color: shade(w, k), side: DoubleSide }), lambert!({ color: shade(l, k), side: DoubleSide })),
        }
    };
    let both = [mats_for(1.0), mats_for(0.86)];
    for i in 0..stem_count {
        // the old leaves' three draws a stem: how it fans out, how far its
        // head leans out from the wall, how it is turned on itself
        let (u_fan, u_tilt, u_turn) = (rnd(), rnd(), rnd());
        let k = i as f64 - (n - 1.0) / 2.0;
        let stem = group();
        stem.set_rotation_order(EulerOrder::XZY);
        stem.position().set(HB_TIE.0 + k * 0.0028, HB_TIE.1, HB_TIE.2 + 0.003 * f64::from(i % 2) - 0.001);
        stem.rotation().set(-(0.06 + 0.16 * u_tilt), (u_turn - 0.5) * 1.2, k / n * 0.3 + (u_fan - 0.5) * 0.1);
        let v = ["A", "B", "C"][i as usize % 3];
        let mats = &both[i as usize % 2];
        match herb {
            Herb::Sage(_) => {
                put(&stem, &format!("Real__herbSage{v}"), &mats.0);
            }
            Herb::Rosemary(..) => {
                put(&stem, &format!("Real__herbRosemaryWood{v}"), &mats.0);
                put(&stem, &format!("Real__herbRosemaryNeedles{v}"), &mats.1);
            }
            Herb::Lavender(..) => {
                put(&stem, &format!("Real__herbLavenderStem{v}"), &mats.0);
                put(&stem, &format!("Real__herbLavenderSpike{v}"), &mats.1);
            }
        }
        body.add(&stem);
    }
    bundle.rotation().y = face_wall(dx, dz);
    bundle.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.008), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.008));
    g.world.add(&bundle);
}
