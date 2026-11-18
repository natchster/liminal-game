//! The facility's own archetypes (`docs/plan/FACILITY.md` F8-F9), kept out of
//! the old game's table (`ARCHETYPES`, held to its recording by
//! `archgen_tables`): the vault, its gallery, how a floor comes to have one
//! and how it is carved. Its arch, lining, lamps and draught are the world's
//! (`world::vaults`).

use super::{Archetype, Ctx, LevelSchedule, Rect, Zone, fill_rect, make_zone};

/// The facility's own archetypes (`docs/plan/FACILITY.md` F8-F9): not the
/// old game's table (`ARCHETYPES`, held to its recording by
/// `archgen_tables`), and found by `archetype` after it. No floor's
/// schedule weighs them: `generate_layout` turns a zone into one by its
/// shape, on floors from depth `VAULT_DEPTH`.
pub static FACILITY_ARCHETYPES: [Archetype; 1] = [
    // one long arched chamber (the real galleries: 9-12.5 m wide, 6.4-7.5 m
    // high, `docs/research/facility.md` 2.2); its arch, its lining and its
    // draught are the world's (`world::vaults`)
    Archetype {
        key: "vault",
        title: "THE VAULTS",
        lore: "",
        ceil: [6.4, 7.0],
        wall: "solid",
        t: [4.0, 4.0],
        protrude: None,
        pillar: [4.0, 4.0],
        jamb: [0.0, 0.0],
        jamb_chance: 0.0,
        lintel: [2.9, 3.2],
        low_h: None,
        wall_mat: "concrete",
        floor_mat: "concrete",
        ceil_mat: "slab",
        light_every: 3,
        no_spawn: true,
    },
];

/// The archetype a vault zone is (`FACILITY_ARCHETYPES`)
pub const VAULT: &str = "vault";
/// The rooms' depth from which a floor may have a vault (game level 9)
pub const VAULT_DEPTH: i32 = 6;

/// A vault's gallery in its zone's tiles (inclusive): along x or z, from
/// `a0` to `a1` along it and `c0` to `c1` across. Two tiles across a zone
/// narrower than seven, three otherwise, in its middle; a tile of wall
/// left at each end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gallery {
    pub along_x: bool,
    pub a0: i32,
    pub a1: i32,
    pub c0: i32,
    pub c1: i32,
}

impl Gallery {
    /// Tile (`x`, `z`) is the gallery's
    pub fn has(&self, x: i32, z: i32) -> bool {
        let (a, c) = if self.along_x { (x, z) } else { (z, x) };
        a >= self.a0 && a <= self.a1 && c >= self.c0 && c <= self.c1
    }
}

/// The gallery a vault zone of these tiles is carved with (`carve_vault`)
pub fn vault_gallery(x0: i32, z0: i32, x1: i32, z1: i32) -> Gallery {
    let along_x = x1 - x0 >= z1 - z0;
    let (lo, hi, s0, s1) = if along_x { (x0, x1, z0, z1) } else { (z0, z1, x0, x1) };
    let short = s1 - s0 + 1;
    let gw = if short >= 7 { 3 } else { 2 };
    let c0 = s0 + (short - gw) / 2;
    Gallery { along_x, a0: lo + 1, a1: hi - 1, c0, c1: c0 + gw - 1 }
}

/// A vault (FACILITY F8): from `VAULT_DEPTH`, on a floor of the rooms' own
/// mix known for the facility's places (the Maze, the Poolrooms, the Lobby
/// and the offices keep their floors: rule 8 and the plan's scope), the
/// longest zone of the halls or the tunnels that is long and narrow enough
/// for a gallery (never the spawn's) becomes one, chosen by its shape alone:
/// a floor with none draws as it did
pub(super) fn pick(level: i32, sched: &LevelSchedule, scp: Option<&str>, zones: &mut [Zone], rng: &mut dyn FnMut() -> f64, cell: f64) {
    let dominant = sched.dominant;
    if level >= VAULT_DEPTH
        && !sched.patchwork
        && !matches!(scp, Some("store" | "house"))
        && matches!(dominant, "hall" | "tunnels" | "electrical" | "pillars")
    {
        let pick = zones
            .iter()
            .filter(|z| {
                let (w, d) = (z.x1 - z.x0 + 1, z.z1 - z.z0 + 1);
                let (long, short) = (w.max(d), w.min(d));
                let has_spawn = z.x0 <= 1 && z.z0 <= 1;
                !has_spawn && matches!(z.arch, "hall" | "tunnels") && (5..=10).contains(&short) && long >= 10
            })
            .max_by_key(|z| ((z.x1 - z.x0).max(z.z1 - z.z0), std::cmp::Reverse(z.id)))
            .map(|z| (z.rect(), z.id));
        if let Some((rect, id)) = pick {
            zones[id] = make_zone(VAULT, &rect, id, rng, cell);
        }
    }
}

// The facility's vault (FACILITY F8): walls, one arched gallery along the
// zone's long side (`vault_gallery`), blind at both ends, and a side
// passage or two from it to the zone's edge
pub(super) fn carve_vault(c: &mut Ctx, z: Rect) {
    fill_rect(c, z, 1);
    let g = vault_gallery(z.x0, z.z0, z.x1, z.z1);
    let at = |a: i32, k: i32| if g.along_x { (a, k) } else { (k, a) };
    for a in g.a0..=g.a1 {
        for k in g.c0..=g.c1 {
            let (x, zz) = at(a, k);
            c.set(x, zz, 0);
        }
    }
    // its ends kept whole where the floor can be joined another way
    for a in [g.a0 - 1, g.a1 + 1] {
        for k in g.c0..=g.c1 {
            let (x, zz) = at(a, k);
            let i = c.idx(x, zz);
            c.hard[i] = true;
        }
    }
    // the passages: one, two along a long gallery, each in its own stretch
    // of it and off to one side, out to the zone's edge
    let (s0, s1) = if g.along_x { (z.z0, z.z1) } else { (z.x0, z.x1) };
    let n = if g.a1 - g.a0 >= 12 { 2 } else { 1 };
    let stretch = (g.a1 - g.a0 + 1) / n;
    for k in 0..n {
        let lo = g.a0 + 1 + k * stretch;
        let hi = (g.a0 + (k + 1) * stretch - 2).max(lo);
        let a = c.irange(lo, hi);
        let (from, to) = if c.r() < 0.5 { (s0, g.c0 - 1) } else { (g.c1 + 1, s1) };
        for kk in from..=to {
            let (x, zz) = at(a, kk);
            c.set(x, zz, 0);
        }
    }
}
