//! The pit hall's tanks (`docs/plan/FACILITY.md` F9, the developer's
//! answers): a hall of open concrete tanks with water standing in them, in
//! rows with the hall's floor between them and round them
//! (`level::archgen::pit_tanks`), a railed catwalk across some, and columns
//! rising out of their water to the slab, as a covered reservoir's do.
//!
//! Each tank is one of the floor's drops (`world::drops`), so it is cut,
//! railed, kept clear and stopped at its edge as they are; what is its own
//! is its water's level and what stands in it. Its one wrong thing (the
//! developer's answer): every tank's walls, and its columns, are wet a hand
//! or more above its water, as if the level had just fallen, and nothing
//! drains: no outlet runs, no sound of draining, no drip (`LORE.md`, *The
//! Pit Halls*). Its levels are its own, tank by tank. Planned with the
//! floor's drops, from a stream of its own (no draw of the level moves).

use crate::base::rng::SplitMix64;
use crate::level::archgen::{PITS, pit_tanks};
use crate::prelude::*;
use crate::world::drops::{Bottom, Drop};
use crate::world::lining::Face;

/// How far below the floor a tank's water stands (m), from and to
pub const WATER: [f64; 2] = [1.3, 2.1];
/// How far above its water a tank's walls are wet (m): a hand or more (a
/// hand's breadth, 10 cm, was a line across a tank: the renders)
pub const WET: [f64; 2] = [0.18, 0.42];
/// Wet concrete's shade against dry (water in its pores darkens it to
/// well under half)
pub const WET_SHADE: f64 = 0.4;
/// The wet's soft top edge (m), where it wicks into the dry
const FRONT: f64 = 0.025;
/// A tank's water's colour: murky, a little green
const WATER_COLOR: u32 = 0x353a32;
/// A column's side (m): a reservoir's square concrete columns
pub const COLUMN_W: f64 = 0.7;
/// The share of tanks with a side three tiles across that a catwalk crosses
const CATWALK_SHARE: f64 = 0.5;
/// The tanks' own stream
const PIT_SALT: u64 = 0x5A1D_7C3E_0F92_B64D;
/// The placers whose things are water going somewhere or come from
/// somewhere (a puddle, a drip's, a drain, a leak, the streaks down a wall,
/// a broken window's wet foot): none is left in a pit hall, where nothing
/// drains (`Game::clear_pits_of_water`)
pub const WET_PLACERS: [&str; 6] = [
    "place_floor_puddles",
    "place_water_puddles",
    "place_floor_drain_grates",
    "place_wall_pipe_leaks",
    "place_wall_water_streaks",
    "place_broken_window_frames",
];
/// The floor's ambient sounds of water falling (`AMBIENT_EVENTS`): none is
/// anchored in a pit hall
pub const WET_SIGNATURES: [&str; 2] = ["drip", "dripHeavy"];

/// What is a pit hall's tank's own (see the module)
#[derive(Clone, Debug)]
pub struct Tank {
    /// How far below the floor its water stands (m)
    pub water: f64,
    /// How far above its water its walls and columns are wet (m)
    pub wet: f64,
    /// Its columns' middles, in world metres (x, z)
    pub columns: Vec<(f64, f64)>,
}

impl Game {
    /// Plan the pit halls' tanks (see the module): each a drop with its
    /// water, its wet band, its catwalk and its columns, cut as the floor's
    /// drops are (kept only where every open tile is still reached)
    pub(crate) fn plan_pits(&mut self) {
        let Some(l) = self.layout.as_ref() else { return };
        let zones: Vec<(usize, i32, i32, i32, i32)> =
            l.zones.iter().enumerate().filter(|(_, z)| z.arch == PITS).map(|(i, z)| (i, z.x0, z.z0, z.x1, z.z1)).collect();
        let (w, h) = (self.w as f64, self.h as f64);
        let at = |u: f64, v: f64| ((u - w / 2.0) * CELL, (v - h / 2.0) * CELL);
        for (zi, x0, z0, x1, z1) in zones {
            let mut r = SplitMix64::new(self.current_seed, PIT_SALT ^ (zi as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
            for t in pit_tanks(x0, z0, x1, z1) {
                let (bw, bh) = (t.x1 - t.x0 + 1, t.z1 - t.z0 + 1);
                let (crossed, way, level, band) = (r.next(), r.next(), r.next(), r.next());
                // a catwalk along the middle of a side three tiles across
                // (`Drop::catwalk`: along x at a row, or along z at a column)
                let along_z = (false, t.x0 + 1);
                let along_x = (true, t.z0 + 1);
                let catwalk = match (bw, bh) {
                    _ if crossed >= CATWALK_SHARE => None,
                    (3, 3) => Some(if way < 0.5 { along_z } else { along_x }),
                    (3, _) => Some(along_z),
                    (_, 3) => Some(along_x),
                    _ => None,
                };
                // (landing on the floor at both ends)
                let catwalk = catwalk.filter(|&(ax, k)| {
                    let ends = if ax { [(t.x0 - 1, k), (t.x1 + 1, k)] } else { [(k, t.z0 - 1), (k, t.z1 + 1)] };
                    ends.iter().all(|&(ex, ez)| self.in_grid(ex, ez) && self.tiles[self.t_idx(ex, ez)] == 0)
                });
                // its columns: either side of its catwalk, on the line
                // across it through the tank's middle; else across its
                // longer side, or one in the middle of a small one
                let (cu, cv) = ((t.x0 + t.x1 + 1) as f64 / 2.0, (t.z0 + t.z1 + 1) as f64 / 2.0);
                let across_x = vec![at(t.x0 as f64 + 0.5, cv), at(t.x1 as f64 + 0.5, cv)];
                let across_z = vec![at(cu, t.z0 as f64 + 0.5), at(cu, t.z1 as f64 + 0.5)];
                let columns = match catwalk {
                    Some((false, _)) => across_x,
                    Some((true, _)) => across_z,
                    None if bw == 3 => across_x,
                    None if bh == 3 => across_z,
                    None => vec![at(cu, cv)],
                };
                let tank = Tank { water: WATER[0] + (WATER[1] - WATER[0]) * level, wet: WET[0] + (WET[1] - WET[0]) * band, columns };
                let d = Drop { x0: t.x0, z0: t.z0, x1: t.x1, z1: t.z1, bottom: Bottom::Water, catwalk, tank: Some(tank) };
                self.cut_drop(d);
            }
        }
    }
}

impl Game {
    /// World (`x`, `z`) is on a pit hall's floor (its zone's tiles)
    pub fn in_pit_hall(&self, x: f64, z: f64) -> bool {
        let (tx, tz) = (self.tx(x), self.tz(z));
        self.in_grid(tx, tz) && self.zone_arch_of(self.t_idx(tx, tz)) == PITS
    }

    /// Take out of the pit halls what settling left there of water going
    /// somewhere (`WET_PLACERS`, from the world's child `start`), and the
    /// places their sounds came from: nothing there drains or drips. After
    /// the placers, so none of their draws moves.
    pub(crate) fn clear_pits_of_water(&mut self, start: usize) {
        let gone: Vec<Obj> = self
            .world
            .children()
            .into_iter()
            .skip(start)
            .filter(|top| top.user().placer.as_deref().is_some_and(|p| WET_PLACERS.contains(&p)))
            .filter(|top| {
                let mut bb = Box3::empty();
                bb.set_from_object(top);
                !bb.is_empty() && self.in_pit_hall((bb.min.x + bb.max.x) / 2.0, (bb.min.z + bb.max.z) / 2.0)
            })
            .collect();
        for top in gone {
            self.world.remove(&top);
        }
        let puddles: Vec<Vec3> = self.pp.puddle_positions.iter().copied().filter(|p| !self.in_pit_hall(p.x, p.z)).collect();
        let drains: Vec<Vec3> = self.pp.drain_grate_positions.iter().copied().filter(|p| !self.in_pit_hall(p.x, p.z)).collect();
        self.pp.puddle_positions = puddles;
        self.pp.drain_grate_positions = drains;
    }

    /// Whether a pit hall hangs a high-bay over its open tile (`x`, `z`)
    /// (`build_world`'s panels; None off a pit hall): over its floor
    /// between the tanks, at every crossing of its walkways and, where it
    /// is not rotting (`dense`), over the middle of each tank's side too, 8
    /// m apart down every walkway (the rooms' grid of 3 or 6 tiles fell on
    /// its walkways, 4 apart, now and then: the hall went dim)
    pub(crate) fn pit_lamp(&self, x: i32, z: i32, dense: bool) -> Option<bool> {
        let zn = self.zone_ref(x, z)?;
        if zn.arch != PITS {
            return None;
        }
        let tanks = pit_tanks(zn.x0, zn.z0, zn.x1, zn.z1);
        let col = |x: i32| tanks.iter().any(|t| x >= t.x0 && x <= t.x1);
        let row = |z: i32| tanks.iter().any(|t| z >= t.z0 && z <= t.z1);
        // a walkway beside a tank (the ring's tile beside the tanks, not
        // the one by the wall)
        let walk_x = !col(x) && (col(x - 1) || col(x + 1));
        let walk_z = !row(z) && (row(z - 1) || row(z + 1));
        let mid_x = tanks.iter().any(|t| t.x1 - t.x0 == 2 && x == t.x0 + 1);
        let mid_z = tanks.iter().any(|t| t.z1 - t.z0 == 2 && z == t.z0 + 1);
        Some(match (walk_x, walk_z) {
            (true, true) => true,
            (true, false) => dense && mid_z,
            (false, true) => dense && mid_x,
            _ => false,
        })
    }
}

/// A tank's face from `a` to `b` (two points on one level, the face turned
/// along `n`) down from `top`: dry down to its wet band, the band (its top
/// edge soft) down into its water
pub(crate) fn tank_face(dry: &mut Face, wet: &mut Face, a: Vec3, b: Vec3, n: Vec3, top: f64, t: &Tank) {
    let uv = move |p: &Vec3| ((n.z * p.x - n.x * p.z) / 4.0, p.y / 4.0);
    let line = -t.water + t.wet;
    let front = line - FRONT;
    let under = -t.water - 0.05;
    let q = |ya: f64, yb: f64| [Vec3::new(a.x, ya, a.z), Vec3::new(b.x, ya, b.z), Vec3::new(b.x, yb, b.z), Vec3::new(a.x, yb, a.z)];
    dry.quad(q(top, line), n, uv, None);
    wet.quad(q(line, front), n, uv, Some([1.0, 1.0, WET_SHADE, WET_SHADE]));
    wet.quad(q(front, under), n, uv, Some([WET_SHADE; 4]));
}

/// A tank's water: still and murky, the hall's light dull on its face and
/// the lamps' gloss on it, so its walls' wet band stands darker than it
/// (the drops' water, near black, swallowed the band)
pub(crate) fn water_mat() -> Mat {
    phong!({ color: WATER_COLOR, specular: 0x4a4a46, shininess: 40 })
}

/// A tank's column at (`x`, `z`), from its water up to `ceil` (into the
/// slab a little), wet above its water as the tank's walls are
pub(crate) fn tank_column(dry: &mut Face, wet: &mut Face, (x, z): (f64, f64), ceil: f64, t: &Tank) {
    let s = COLUMN_W / 2.0;
    let top = ceil + 0.02;
    let sides = [
        (Vec3::new(x - s, 0.0, z + s), Vec3::new(x + s, 0.0, z + s), Vec3::new(0.0, 0.0, 1.0)),
        (Vec3::new(x + s, 0.0, z - s), Vec3::new(x - s, 0.0, z - s), Vec3::new(0.0, 0.0, -1.0)),
        (Vec3::new(x + s, 0.0, z + s), Vec3::new(x + s, 0.0, z - s), Vec3::new(1.0, 0.0, 0.0)),
        (Vec3::new(x - s, 0.0, z - s), Vec3::new(x - s, 0.0, z + s), Vec3::new(-1.0, 0.0, 0.0)),
    ];
    for (a, b, n) in sides {
        tank_face(dry, wet, a, b, n, top, t);
    }
}

/// The wet band's material: its wall's, shaded by its vertices, with a
/// wet face's sheen
pub(crate) fn wet_mat(dry: &Mat) -> Mat {
    let m = dry.duplicate();
    {
        let mut d = m.m();
        d.vertex_colors = true;
        d.kind = MatKind::Phong;
        d.specular = Color::hex(0x3a3a36);
        d.shininess = 60.0;
    }
    m
}
