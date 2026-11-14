//! Choosing where a thing goes: open and wall tiles, picks and shuffles from the level's stream, salts from a place.
use game::prelude::*;

/// `atan2(-dx, -dz)` with float negation, so a zero component stays `-0.0` (kept on purpose).
pub fn face_wall(dx: i32, dz: i32) -> f64 {
    (-(dx as f64)).atan2(-(dz as f64))
}

/// The index of `c` in `cs` (the old colour a draw picked)
pub fn index_of(cs: &[u32], c: u32) -> usize {
    cs.iter().position(|&x| x == c).unwrap_or(0)
}

/// `wx(x) + (`rnd()` - 0.5) * s, 0, wz(z) + (`rnd()` - 0.5) * s` (x draw first).
pub fn jitter(g: &Game, x: i32, z: i32, s: f64) -> (f64, f64) {
    let px = g.wx(x) + (rnd() - 0.5) * s;
    let pz = g.wz(z) + (rnd() - 0.5) * s;
    (px, pz)
}

/// Every open interior tile, row by row (the `open_tiles` loop every placer repeats).
pub fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                v.push((x, z));
            }
        }
    }
    v
}

/// A uniformly random element (one `rnd()` draw).
pub fn pick<T: Copy>(a: &[T]) -> T {
    a[(rnd() * a.len() as f64).floor() as usize]
}

/// A uniformly random colour from a table of them (one `rnd()` draw)
pub fn pick_hex(a: &[u32]) -> u32 {
    a[(rnd() * a.len() as f64).floor() as usize]
}

/// The repeated `open_tiles` scan plus one random pick.
/// No draw when the list is empty (the placer returns first).
pub fn pick_open(g: &Game) -> Option<(i32, i32)> {
    let mut open: Vec<(i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                open.push((x, z));
            }
        }
    }
    if open.is_empty() {
        return None;
    }
    Some(open[(rnd() * open.len() as f64).floor() as usize])
}

/// The repeated `wall_tiles` scan (first wall neighbour in [[1,0],[-1,0],[0,1],[0,-1]] order)
/// + one draw to pick a tile.
pub fn pick_wall(g: &Game) -> Option<(i32, i32, i32, i32)> {
    let mut wall: Vec<(i32, i32, i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        wall.push((x, z, dx, dz));
                        break;
                    }
                }
            }
        }
    }
    if wall.is_empty() {
        return None;
    }
    Some(wall[(rnd() * wall.len() as f64).floor() as usize])
}

/// A uniformly random element (one `rnd()` draw). index
#[inline]
pub fn ri(len: usize) -> usize {
    (rnd() * len as f64).floor() as usize
}

/// A salt from where a thing is (no draw): its painters' strokes differ
/// from floor to floor
pub fn salt(x: f64, z: f64, k: u32) -> u32 {
    ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
}

/// A salt from where the thing lies (no draw): its wear differs from floor
/// to floor
pub fn salt_at(x: f64, z: f64) -> u32 {
    ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ 0x7f4a_7c15
}

/// A salt from where a thing is (no draw): its painters' strokes differ
/// from floor to floor
pub fn salt_of(x: f64, z: f64) -> u32 {
    let (a, b) = ((x * 1000.0).round() as i64 as u64, (z * 1000.0).round() as i64 as u64);
    let mut h = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 29;
    (h ^ (h >> 32)) as u32
}

/// A seed from values already fixed (a position): no draws
pub fn seed_of(v: &[f64]) -> u32 {
    let mut x: u64 = 0xcbf2_9ce4_8422_2325;
    for a in v {
        for b in ((a * 1.0e4).round() as i64).to_le_bytes() {
            x ^= u64::from(b);
            x = x.wrapping_mul(0x0100_0000_01b3);
        }
    }
    ((x ^ (x >> 32)) as u32) | 1
}

/// In-place Fisher-Yates exactly loops do it.
pub fn shuffle<T>(v: &mut [T]) {
    for i in (1..v.len()).rev() {
        let j = (rnd() * (i + 1) as f64).floor() as usize;
        v.swap(i, j);
    }
}

/// Draws `salt % 61` times from the level's stream and drops them: a
/// thing's own stream would start the same on every floor; the salt (from
/// where it is) moves where it starts
pub fn skip(salt: u32) {
    for _ in 0..(salt % 61) {
        let _ = rnd();
    }
}

/// Something of where a thing was put, to vary what its painters paint
/// (a painter's own stream starts the same on every floor)
pub fn vary(x: f64, z: f64, k: f64) -> u32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [x, z, k] {
        h ^= v.to_bits();
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h ^ (h >> 29) ^ (h >> 47)) as u32
}

/// The surface of the wall seen from open tile (`x`, `z`): its arch
/// material's key (`wp:<variant>` a papered wall), as the walls are drawn
pub fn wall_key_at(g: &Game, x: i32, z: i32) -> String {
    if g.layout.is_none() || !g.in_grid(x, z) {
        return "wp:0".into();
    }
    g.wall_mat_key(g.t_idx(x, z))
}

/// `rotation.y = `atan2`(-dx, -dz)` + position on the wall face `CELL * 0.5 - inset` out.
pub fn wall_mount(g: &Game, o: &Obj, x: i32, z: i32, dx: i32, dz: i32, inset: f64) {
    o.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    o.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - inset), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - inset));
}

/// The `wall_tiles` scan: open interior tiles with the first wall neighbour
/// in `[[1,0],[-1,0],[0,1],[0,-1]]` order, as `(x, z, dx, dz)`.
pub fn wall_tiles(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        v.push((x, z, dx, dz));
                        break;
                    }
                }
            }
        }
    }
    v
}

/// Open tiles with their FIRST adjacent wall tile (`tiles[..] == 1`; later sides are skipped):
/// the `wall_tiles` loop of the bathroom placers.
pub fn wall_tiles_first(g: &Game) -> Vec<WallCand> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        v.push(WallCand { x, z, dx, dz });
                        break;
                    }
                }
            }
        }
    }
    v
}

/// Wall-mount position `wx(x) + dx * (CELL * 0.5 - inset)`, `wz(z) + dz * (...)`.
pub fn wall_pos(g: &Game, x: i32, z: i32, dx: i32, dz: i32, inset: f64) -> (f64, f64) {
    (g.wx(x) + dx as f64 * (CELL * 0.5 - inset), g.wz(z) + dz as f64 * (CELL * 0.5 - inset))
}

/// `{ x, z, dx, dz }`: an open tile with a wall (or the grid edge) on side (dx, dz).
#[derive(Clone, Copy, Debug)]
pub struct WallCand {
    pub x: i32,
    pub z: i32,
    pub dx: i32,
    pub dz: i32,
}

/// The candidate scan every wall placer repeats.
pub fn wall_candidates(g: &Game) -> Vec<WallCand> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    v.push(WallCand { x, z, dx, dz });
                }
            }
        }
    }
    v
}

/// The shared wall-mount tail: face the room, sit `inset` off the wall face,
/// slide along the wall by `(`rnd()` - 0.5) * jitter`.
pub fn mount_on_wall(g: &Game, o: &Obj, c: &WallCand, inset: f64, center_y: f64, jitter: f64) {
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - inset);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - inset);
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        let dz = (rnd() - 0.5) * jitter;
        o.position().set(wall_x, center_y, wall_z + dz);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        let dx = (rnd() - 0.5) * jitter;
        o.position().set(wall_x + dx, center_y, wall_z);
    }
}

/// The wall-face point: `wx(c.x) + c.dx * (CELL * 0.5 - inset)`, same for z.
pub fn wall_xz(g: &Game, c: WallCand, inset: f64) -> (f64, f64) {
    (g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - inset), g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - inset))
}

/// The wall-mount tail: face the room, then position, with an optional
/// `(`rnd()` - 0.5) * jitter` slide along the wall.
pub fn mount(o: &Obj, c: WallCand, wall_x: f64, wall_z: f64, center_y: f64, jitter: Option<f64>) {
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        let zz = match jitter {
            Some(j) => wall_z + (rnd() - 0.5) * j,
            None => wall_z,
        };
        o.position().set(wall_x, center_y, zz);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        let xx = match jitter {
            Some(j) => wall_x + (rnd() - 0.5) * j,
            None => wall_x,
        };
        o.position().set(xx, center_y, wall_z);
    }
}

/// `o.position.set(wx(x) + (r - 0.5) * s, 0, wz(z) + (r - 0.5) * s); o.rotation.y = r * 2π`
pub fn scatter(g: &Game, o: &Obj, x: i32, z: i32, spread: f64) {
    let px = g.wx(x) + (rnd() - 0.5) * spread;
    let pz = g.wz(z) + (rnd() - 0.5) * spread;
    o.position().set(px, 0, pz);
    o.rotation().y = rnd() * PI * 2.0;
}

/// The wall-mount tail: face the room, then position with a
/// `(`rnd()` - 0.5) * jitter` slide along the wall.
pub fn mount_slid(o: &Obj, c: WallCand, wall_x: f64, wall_z: f64, center_y: f64, jitter: f64) {
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        let zz = wall_z + (rnd() - 0.5) * jitter;
        o.position().set(wall_x, center_y, zz);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        let xx = wall_x + (rnd() - 0.5) * jitter;
        o.position().set(xx, center_y, wall_z);
    }
}

/// `o.position.set(wx(x) + (r - 0.5) * s, y, wz(z) + (r - 0.5) * s); o.rotation.y = r * 2π`
pub fn scatter_y(g: &Game, o: &Obj, x: i32, z: i32, spread: f64, y: f64) {
    let px = g.wx(x) + (rnd() - 0.5) * spread;
    let pz = g.wz(z) + (rnd() - 0.5) * spread;
    o.position().set(px, y, pz);
    o.rotation().y = rnd() * PI * 2.0;
}

/// `o.position.x = wx(x) + ...; o.position.z = wz(z) + ...; o.rotation.y = ...` (y kept)
pub fn scatter_xz_turned(g: &Game, o: &Obj, x: i32, z: i32, spread: f64) {
    o.position().x = g.wx(x) + (rnd() - 0.5) * spread;
    o.position().z = g.wz(z) + (rnd() - 0.5) * spread;
    o.rotation().y = rnd() * PI * 2.0;
}

/// A uniformly random tile (one `rnd()` draw).
pub fn pick_tile(t: &[(i32, i32)]) -> (i32, i32) {
    t[(rnd() * t.len() as f64).floor() as usize]
}

/// The wall-hung panel placement shared by the window / portrait / crucifix:
/// returns nothing, sets rotation.y and position (one draw for the slide).
pub fn hang_on_wall(g: &Game, o: &Obj, c: &WallCand, inset: f64, center_y: f64) {
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - inset);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - inset);
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        o.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        o.position().set(wall_x + (rnd() - 0.5) * 0.6, center_y, wall_z);
    }
}

/// Position `o` at `(wx(x) + (rnd() - 0.5) * s, 0, wz(z) + (rnd() - 0.5) * s)`, x drawn first.
pub fn scatter_pos(g: &Game, o: &Obj, x: i32, z: i32, s: f64) {
    let px = g.wx(x) + (rnd() - 0.5) * s;
    let pz = g.wz(z) + (rnd() - 0.5) * s;
    o.position().set(px, 0, pz);
}

/// Set only x and z: `wx(x) + (rnd() - 0.5) * s`, then `wz(z) + (rnd() - 0.5) * s`.
pub fn scatter_xz(g: &Game, o: &Obj, x: i32, z: i32, s: f64) {
    let px = g.wx(x) + (rnd() - 0.5) * s;
    o.position().x = px;
    let pz = g.wz(z) + (rnd() - 0.5) * s;
    o.position().z = pz;
}

/// A uniformly random colour (one `rnd()` draw).
#[inline]
pub fn pick_color(c: &[u32]) -> u32 {
    c[(rnd() * c.len() as f64).floor() as usize]
}

/// The shared wall-mount branch (rotation from the wall side, one random
/// draw for the offset along the wall).
pub fn mount_slid_ref(o: &Obj, c: &WallCand, wall_x: f64, wall_z: f64, y: f64, spread: f64) {
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        o.position().set(wall_x, y, wall_z + (rnd() - 0.5) * spread);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        o.position().set(wall_x + (rnd() - 0.5) * spread, y, wall_z);
    }
}
