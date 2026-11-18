//! The facility's pit hall (`docs/plan/FACILITY.md` F9): which zone of a
//! floor becomes one, and where its tanks lie. The zone is carved as an open
//! floor; its tanks are cut into it, railed, filled and wetted by the world
//! (`world::pits`, through `world::drops`), from `pit_tanks`.

use super::{Ctx, LevelSchedule, Rect, Zone, fill_rect, make_zone};

/// The archetype a pit hall's zone is (`FACILITY_ARCHETYPES`)
pub const PITS: &str = "pits";
/// The shortest side (tiles) of a hall that can be a pit hall: two rows of
/// tanks and the floor between and round them
pub const PITS_SHORT: i32 = 9;

/// A tank of a pit hall, in its zone's tiles (inclusive)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TankRect {
    pub x0: i32,
    pub z0: i32,
    pub x1: i32,
    pub z1: i32,
}

/// The runs of tanks across a zone's tiles `lo..=hi`, as (first tile, tiles):
/// 12 m tanks (three tiles) a 4 m walkway apart, an 8 m one where two tiles
/// are left, the rest given to the floor round them, half each side. A
/// walkway is kept inside the zone's edge at both ends (its last tile may
/// be the wall a seam puts there).
fn runs(lo: i32, hi: i32) -> Vec<(i32, i32)> {
    let (a0, a1) = (lo + 1, hi - 2);
    let mut left = a1 - a0 + 1;
    let mut sizes = vec![];
    loop {
        let gap = if sizes.is_empty() { 0 } else { 1 };
        if left - gap >= 3 {
            sizes.push(3);
            left -= gap + 3;
        } else if left - gap >= 2 {
            sizes.push(2);
            left -= gap + 2;
        } else {
            break;
        }
    }
    let mut at = a0 + left / 2;
    sizes
        .into_iter()
        .map(|s| {
            let r = (at, s);
            at += s + 1;
            r
        })
        .collect()
}

/// The tanks of a pit hall whose zone is these tiles (inclusive): in rows
/// along both sides, the hall's floor between them and round them (the
/// developer's answer)
pub fn pit_tanks(x0: i32, z0: i32, x1: i32, z1: i32) -> Vec<TankRect> {
    let (xs, zs) = (runs(x0, x1), runs(z0, z1));
    zs.iter().flat_map(|&(z, d)| xs.iter().map(move |&(x, w)| TankRect { x0: x, z0: z, x1: x + w - 1, z1: z + d - 1 })).collect()
}

/// A pit hall (FACILITY F9): on the floors that may have a vault
/// (`vault::facility_floor`) with no anomaly on them (its tanks are drops,
/// and no drop is cut on an anomaly's floor: `world::drops`), the largest
/// of the Habitable Zone's halls wide enough for two rows of tanks (never
/// the spawn's, never the vault), chosen by its shape alone: a floor with
/// none draws as it did
pub(super) fn pick(level: i32, sched: &LevelSchedule, scp: Option<&str>, zones: &mut [Zone], rng: &mut dyn FnMut() -> f64, cell: f64) {
    if scp.is_some() || !super::vault::facility_floor(level, sched, scp) {
        return;
    }
    let pick = zones
        .iter()
        .filter(|z| {
            let (w, d) = (z.x1 - z.x0 + 1, z.z1 - z.z0 + 1);
            let has_spawn = z.x0 <= 1 && z.z0 <= 1;
            !has_spawn && z.arch == "hall" && w.min(d) >= PITS_SHORT
        })
        .max_by_key(|z| ((z.x1 - z.x0 + 1) * (z.z1 - z.z0 + 1), std::cmp::Reverse(z.id)))
        .map(|z| (z.rect(), z.id));
    if let Some((rect, id)) = pick {
        zones[id] = make_zone(PITS, &rect, id, rng, cell);
    }
}

// The facility's pit hall (FACILITY F9): an open floor, its tanks the
// world's, its walls the seams a hall has (`build_seams`: open, it ran into
// the zones round it with no wall)
pub(super) fn carve_pits(c: &mut Ctx, z: Rect) {
    fill_rect(c, z, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tanks_keep_a_walkway_round_and_between_them() {
        for n in 9..=17 {
            let t = pit_tanks(10, 20, 10 + n - 1, 20 + 8);
            assert!(t.len() >= 2, "{n}: {t:?}");
            for a in &t {
                assert!(a.x0 >= 11 && a.x1 <= 10 + n - 3 && a.z0 >= 21 && a.z1 <= 26, "{n}: {a:?} on the edge's walkway");
                for b in &t {
                    if a != b {
                        let apart = a.x1 < b.x0 - 1 || b.x1 < a.x0 - 1 || a.z1 < b.z0 - 1 || b.z1 < a.z0 - 1;
                        assert!(apart, "{n}: {a:?} and {b:?} share no walkway");
                    }
                }
            }
        }
    }
}
