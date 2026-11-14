//! `place_props`: the weather-driven extra props, what a floor's weather
//! leaves: puddles on the wet floors, a fallen ceiling tile on the derelict
//! ones, cobwebs on the still ones, a cut cable sparking on the electric ones
//! and candles on the haunted ones (a hand blows one out, and it stays out:
//! `docs/design/LORE.md`, *What still burns*). Part of
//! `place_props_tail` (core_props/tail.rs); the things are modelled in
//! Blender batches `r05` (the cobwebs) and `r10` (the rest).
//!
//! Each haunted candle's state is `g.haunted_candles[i]`
//! (`records::CandleState`): extinguished → `!lit`; `f["fadeT"]` is how far
//! its flame has faded since. The candle's animator and its interactable
//! read and write that record by index, so both share one state.
use crate::base::numeric::hypot2;
use crate::base::records::*;
use crate::base::tables::*;
use crate::core_props::seam::*;
use crate::core_props::*;
use crate::prelude::*;

/// Tile `c`'s centre jittered by ±1 in x and z (x drawn first), at height `y`
pub(crate) fn jitter_pos(g: &Game, c: &Cand, y: f64) -> Vec3 {
    let x = g.wx(c.x) + rand2(-1.0, 1.0);
    let z = g.wz(c.z) + rand2(-1.0, 1.0);
    v3(x, y, z)
}
/// The wall spot's position pushed `d` out along its normal
pub(crate) fn off_wall(spot: &WallSpot, d: f64) -> Vec3 {
    let mut p = spot.pos;
    p.add_scaled_vector(&spot.normal, d);
    p
}
/// Turn `o` toward the camera, staying level (it looks at the camera's x / z at its own height)
pub(crate) fn look_at_level(o: &Obj, cam: &Obj) {
    let c = cam.pos();
    let y = o.pos().y;
    o.look_at(c.x, y, c.z);
}

// ---- candle state accessors (g.haunted_candles[ci])
fn c_out(g: &Game, ci: usize) -> bool {
    g.haunted_candles.get(ci).map(|c| !c.lit).unwrap_or(false)
}
fn set_out(g: &mut Game, ci: usize, out: bool) {
    if let Some(c) = g.haunted_candles.get_mut(ci) {
        c.lit = !out;
    }
}
fn cf(g: &Game, ci: usize, k: &str) -> f64 {
    g.haunted_candles.get(ci).and_then(|c| c.f.get(k).copied()).unwrap_or(0.0)
}
fn set_cf(g: &mut Game, ci: usize, k: &str, v: f64) {
    if let Some(c) = g.haunted_candles.get_mut(ci) {
        c.f.insert(k.to_string(), v);
    }
}

/// one smoke layer of the blow-out puff
#[derive(Clone, Copy, Debug)]
struct SmokeLayer {
    rise: f64,
    wobble: f64,
    phase: f64,
    xoff: f64,
    life: f64,
    scale_xk: f64,
    scale_yk: f64,
    droop: f64,
}
const SMOKE_LAYERS_BASE: [SmokeLayer; 3] = [
    SmokeLayer { rise: 0.38, wobble: 3.2, phase: 0.0, xoff: 0.00, life: 4.5, scale_xk: 1.5, scale_yk: 1.8, droop: 0.03 },
    SmokeLayer { rise: 0.44, wobble: 2.6, phase: 1.7, xoff: 0.04, life: 5.2, scale_xk: 1.2, scale_yk: 1.6, droop: 0.02 },
    SmokeLayer { rise: 0.32, wobble: 3.9, phase: 3.1, xoff: -0.03, life: 3.9, scale_xk: 1.8, scale_yk: 2.1, droop: 0.025 },
];
const SMOKE_EXTRA_WET: SmokeLayer =
    SmokeLayer { rise: 0.28, wobble: 2.2, phase: 4.4, xoff: 0.02, life: 6.0, scale_xk: 2.2, scale_yk: 2.4, droop: 0.10 };

impl Game {
    /// WEATHER-DRIVEN EXTRA PROPS: each weather adds a distinct
    /// visual signature; decoration only.
    pub(crate) fn tail_weather_extras(&mut self, cx: &mut PropsCtx) {
        let weather = self.weather_key();
        if weather == "wet" {
            self.tail_wet_puddles(cx);
        }
        if weather == "derelict" {
            self.tail_derelict_debris(cx);
        }
        if weather == "still" {
            self.tail_still_cobwebs(cx);
        }
        if weather == "electric" {
            self.tail_electric_wires(cx);
        }
        if weather == "haunted" {
            self.tail_haunted_candles(cx);
        }
    }

    /// WET: water standing on the floor, or soaked into it, as batch `r07`
    /// paints a puddle (`place_floor_puddles`, `docs/research/props-remodelling/marks_on_the_rooms_surfaces.md`): on carpet a
    /// dark patch soaked in, its edge soft and ragged, a darker rim where the
    /// dirt wicked out to it; on a hard floor a dark film, its edge sharp and
    /// lobed, a damp margin round it and a sheen of the lights; lit as the
    /// floor under it is
    fn tail_wet_puddles(&mut self, cx: &mut PropsCtx) {
        // the loop bound is deliberately re-drawn every iteration
        let mut i = 0;
        while i < irand(4, 7) {
            let Some(c) = s_take(self, cx, false) else { break };
            // the old decal's size (a plane 0.6-1.2 m square): as much water,
            // a little longer than wide (by its draw's own digits)
            let size = rand2(0.6, 1.2);
            let long = 1.0 + 0.3 * (size * 977.0).fract();
            let (rx, rz) = (size / 2.5 / long.sqrt(), size / 2.5 * long.sqrt());
            let puddle = mesh(plane_geo!(rx * 2.5, rz * 2.5), puddle_mat(rx, rz, floor_kind(self, c.x, c.z)));
            puddle.rotation().x = -PI / 2.0;
            puddle.rotation().z = rand2(0.0, PI * 2.0);
            puddle.set_pos_v(&jitter_pos(self, &c, 0.003));
            // none on the pool's water (its room is made after: `tail_pool_room`);
            // its draws made all the same
            let reach = rx.max(rz) * 1.25;
            let p = puddle.pos();
            let on_pool = self.pool_plan.as_ref().is_some_and(|pl| {
                let (x0, x1) = (self.wx(pl.x) - CELL / 2.0, self.wx(pl.x + pl.pw - 1) + CELL / 2.0);
                let (z0, z1) = (self.wz(pl.z) - CELL / 2.0, self.wz(pl.z + pl.ph - 1) + CELL / 2.0);
                p.x + reach > x0 && p.x - reach < x1 && p.z + reach > z0 && p.z - reach < z1
            });
            if !on_pool {
                self.world.add(&puddle);
            }
            i += 1;
        }
    }

    /// DERELICT: a tile of the suspended ceiling fallen and broken on the
    /// floor (Blender batch `r10`): three of the four pieces a 610 mm
    /// mineral-fibre tile broke into, slid apart where they landed (one
    /// sometimes face down), a bent length of the grid's cross tee in the
    /// gap the fourth left, crumbs of the tile (batch `r07`'s bits) along the
    /// breaks, and the white dust it threw out round it
    fn tail_derelict_debris(&mut self, cx: &mut PropsCtx) {
        use crate::assets::props::prop_part_as_modelled;
        // the painted face (its fissures, the paint yellowed, the leak that
        // brought it down) over the whole tile; the fibre of its core and back
        let face_mat = lambert!({ map: &tile_face_tex() });
        let core_mat = lambert!({ map: &tile_core_tex() });
        // the tee's galvanized steel, and the white cap over its face
        let steel_mat = phong!({ color: 0x9a9c96, specular: 0x3c3c38, shininess: 26 });
        let cap_mat = lambert!({ color: 0xc6c1b2 });
        // batch r07's crumbs of mineral fibre
        let bit_mat = lambert!({ color: 0xb0a998 });
        // the four pieces, each mapped where it was in the tile, and its middle there
        let pieces: Vec<Option<(Geo, Geo, f64, f64)>> = ["A", "B", "C", "D"]
            .iter()
            .map(|k| {
                let core = prop_part_as_modelled(&format!("Real__derelictTilePiece{k}"))?;
                let face = prop_part_as_modelled(&format!("Real__derelictTileFace{k}"))?;
                let bb = core.bounding_box();
                let (mx, mz) = ((bb.min.x + bb.max.x) / 2.0, (bb.min.z + bb.max.z) / 2.0);
                Some((planar_uv(&core, 0.15, 0.0), planar_uv(&face, TILE_SIDE, 0.5), mx, mz))
            })
            .collect();
        // the tile's corners, in the order of the pieces (each piece holds one)
        let corners: [(f64, f64); 4] = [(1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0)];
        // the breaks between them run from near the middle out along -z, +x, +z, -x
        let breaks: [(f64, f64); 4] = [(0.0, -1.0), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)];
        let mut i = 0;
        while i < irand(3, 6) {
            let Some(c) = s_take(self, cx, false) else { break };
            let g = group();
            g.set_pos_v(&jitter_pos(self, &c, 0.0));
            g.rotation().y = rand1(PI * 2.0);
            let mut missing = 0usize;
            // where each piece lies on the floor (to keep the tee off them)
            let mut lying: Vec<Rect2> = vec![];
            for s in 0..3 {
                // the old shard's draws: its size, its place and its turn
                let (w, d) = (rand2(0.25, 0.5), rand2(0.20, 0.4));
                let (px, pz) = (rand2(-0.25, 0.25), rand2(-0.25, 0.25));
                let (rx, ry, rz) = (rand2(-0.6, 0.6), rand2(0.0, 7.0), rand2(-0.6, 0.6));
                // the first's size: which piece is missing (still up there, or crumbled)
                if s == 0 {
                    missing = (((w - 0.25) / 0.25) * 4.0).floor().clamp(0.0, 3.0) as usize;
                }
                let q = (missing + 1 + s) % 4;
                if let Some((core, face, mx, mz)) = &pieces[q] {
                    // slid out a little from where it was in the tile, turned a
                    // little; one in five landed face down
                    let piece = group();
                    let down = d > 0.36;
                    let (at_x, at_z, turn) = (mx * 1.3 + px * 0.12, mz * 1.3 + pz * 0.12, (ry - 3.5) * 0.035);
                    piece.position().set(at_x, if down { TILE_THICK } else { 0.0 }, at_z);
                    piece.rotation().y = turn;
                    let bb = core.bounding_box();
                    // (turned over, it lies turned the other way)
                    let lies = if down { -turn } else { turn };
                    lying.push(Rect2::new(at_x, at_z, lies, (bb.max.x - bb.min.x) / 2.0, (bb.max.z - bb.min.z) / 2.0));
                    if down {
                        piece.rotation().x = PI;
                    }
                    for (geo, mat) in [(core, &core_mat), (face, &face_mat)] {
                        let m = mesh(geo.clone(), mat);
                        m.position().set(-mx, 0, -mz);
                        piece.add(&m);
                    }
                    g.add(&piece);
                }
                // a crumb out along the break past the piece, beyond the tile's edge
                if let Some(bit) =
                    prop_part_as_modelled(["Real__ceilingCracksTileBitA", "Real__ceilingCracksTileBitB", "Real__ceilingCracksTileBitC"][s])
                {
                    let (bx, bz) = breaks[(q + 1) % 4];
                    let r = 0.42 + 0.12 * (rx + 0.6) / 1.2;
                    let crumb = mesh(bit.clone(), &bit_mat);
                    crumb.position().set(bx * r - bz * rz * 0.04, -bit.bounding_box().min.y, bz * r + bx * rz * 0.04);
                    crumb.rotation().y = ry;
                    g.add(&crumb);
                }
            }
            // the old rebar's draws: its length, its place and its turn
            let len = rand2(0.35, 0.55);
            let (tx, _lift, tz) = (rand2(-0.15, 0.15), rand2(0.18, 0.30), rand2(-0.15, 0.15));
            let (_tilt, spin, side) = (rand2(-0.6, 0.6), rand2(0.0, 7.0), rand2(-0.6, 0.6));
            // the cross tee: in the missing piece's quarter, across it, on either side
            let v = if len < 0.45 { "A" } else { "B" };
            if let (Some(steel), Some(cap)) =
                (prop_part_as_modelled(&format!("Real__derelictTeeSteel{v}")), prop_part_as_modelled(&format!("Real__derelictTeeCap{v}")))
            {
                let (sx, sz) = corners[missing];
                // its length across the quarter's diagonal (x along (-sz, sx)), a little off it
                let yaw = (-sx).atan2(-sz) + (spin - 3.5) * 0.05 + if side < 0.0 { PI } else { 0.0 };
                // out along the diagonal until it is clear of the pieces
                let bb = steel.bounding_box();
                let (hx, hz) = ((bb.max.x - bb.min.x) / 2.0, (bb.max.z - bb.min.z) / 2.0);
                let mut out = 0.28;
                let at = |out: f64| (sx * out + tx * 0.15, sz * out + tz * 0.15);
                while out < 0.6 && lying.iter().any(|r| r.overlaps(&Rect2::new(at(out).0, at(out).1, yaw, hx + 0.01, hz + 0.01))) {
                    out += 0.015;
                }
                let tee = group();
                tee.position().set(at(out).0, 0, at(out).1);
                tee.rotation().y = yaw;
                tee.add(&mesh(steel, &steel_mat));
                tee.add(&mesh(cap, &cap_mat));
                g.add(&tee);
            }
            // the dust it threw out round it
            let dust = mesh(plane_geo!(1.5, 1.5), mark_mat(&debris_dust_tex()));
            dust.rotation().x = -PI / 2.0;
            dust.rotation().z = spin;
            dust.position().y = 0.003;
            g.add(&dust);
            self.world.add(&g);
            i += 1;
        }
    }

    /// STILL: dusty cobwebs left where the walls meet the ceiling: in a
    /// room's corner where two walls do, or along a wall's top where there
    /// is no corner (Blender batch `r05`)
    fn tail_still_cobwebs(&mut self, cx: &mut PropsCtx) {
        use crate::assets::props::prop_part_as_modelled;
        // the sheets: silk gone grey with dust, torn, sagging; the threads
        // catch the light
        let sheet_mat = lambert!({ map: &cobweb_silk_tex(), vertexColors: true, transparent: true, depthWrite: false });
        let strand_mat = phong!({ color: 0xb8b4aa, specular: 0x3a3936, shininess: 30, transparent: true, opacity: 0.6, depthWrite: false });
        let web_of = |sheet: &str, strands: &str, corner: bool| -> Obj {
            let w = group();
            if let Some(geo) = prop_part_as_modelled(sheet) {
                w.add(&mesh(cobweb_sheet_geo(&geo, corner), &sheet_mat));
            }
            if let Some(geo) = prop_part_as_modelled(strands) {
                w.add(&mesh(cobweb_lit(&geo, corner, false), &strand_mat));
            }
            w
        };
        // the loop bound is deliberately re-drawn every iteration
        let mut i = 0;
        while i < irand(4, 8) {
            let Some(c) = s_take(self, cx, true) else { break };
            if let Some(spot) = s_wall_adj_spot(self, c.x, c.z) {
                // the old quarter turn's draw: which web, and which side's corner is looked for first
                let k = irand(0, 3);
                // the wall's way (from the tile into it) and its two sides along it
                let (dx, dz) = (-spot.normal.x.round() as i32, -spot.normal.z.round() as i32);
                let sides = if k & 2 == 0 { [(dz, -dx), (-dz, dx)] } else { [(-dz, dx), (dz, -dx)] };
                // a corner: the side is a wall that meets this one (the tile between them a wall too)
                let corner = sides
                    .into_iter()
                    .find(|&(px, pz)| self.is_wall_tile(c.x + px, c.z + pz) && self.is_wall_tile(c.x + px + dx, c.z + pz + dz));
                let g = group();
                match corner {
                    Some((px, pz)) => {
                        let web = if k & 1 == 0 {
                            web_of("Real__cobwebCornerSheetA", "Real__cobwebCornerStrandsA", true)
                        } else {
                            web_of("Real__cobwebCornerSheetB", "Real__cobwebCornerStrandsB", true)
                        };
                        // the web's frame: one wall behind it (+z out of it), the other to its
                        // left (+x out of it); turned so both are the corner's walls
                        let (ax, az) = (-dx as f64, -dz as f64); // out of this wall
                        let (bx, bz) = (-px as f64, -pz as f64); // out of the side wall
                        let yaw = if (az - bx).abs() + (-ax - bz).abs() < 1e-6 { ax.atan2(az) } else { bx.atan2(bz) };
                        // the corner at the tile's corner; the group stands in the middle of the
                        // web, so settling finds both walls' faces beside it (its probes either
                        // side of the group stay on the web's own stretch of each wall)
                        let mut bb = Box3::default();
                        bb.set_from_object(&web);
                        let (in_x, in_z) = ((bb.min.x + bb.max.x) / 2.0, (bb.min.z + bb.max.z) / 2.0);
                        let half = CELL / 2.0;
                        let (ex, ez) = (self.wx(c.x) + (dx + px) as f64 * half, self.wz(c.z) + (dz + pz) as f64 * half);
                        g.set_pos(ex + ax * in_z + bx * in_x, WALL_H, ez + az * in_z + bz * in_x);
                        g.rotation().y = yaw;
                        web.position().set(-in_x, 0, -in_z);
                        g.add(&web);
                    }
                    None => {
                        // along the wall's top; a partition's end or a slim pillar is narrow: the small
                        // web; a pillar's face that is not takes the big one, all its width
                        let (sx, sz) = sides[0];
                        let (wx, wz) = (c.x + dx, c.z + dz);
                        let runs = self.is_wall_tile(wx + sx, wz + sz) || self.is_wall_tile(wx - sx, wz - sz);
                        let pillar = !runs && !self.is_wall_tile(wx + dx, wz + dz);
                        let wide = runs || self.zone_ref(wx, wz).is_none_or(|zn| zn.wall != "thin" || pillar && zn.pillar >= 0.7);
                        let web = if wide && (k & 1 == 0 || pillar) {
                            web_of("Real__cobwebEdgeSheetA", "Real__cobwebEdgeStrandsA", false)
                        } else {
                            web_of("Real__cobwebEdgeSheetB", "Real__cobwebEdgeStrandsB", false)
                        };
                        // the spot stands 0.11 m out from the wall's face (as the old decal did)
                        g.set_pos_v(&off_wall(&spot, 0.05));
                        g.position().y = WALL_H;
                        g.rotation().y = spot.yaw;
                        web.position().z = -0.11;
                        g.add(&web);
                    }
                }
                self.world.add(&g);
            }
            i += 1;
        }
    }

    /// ELECTRIC: a cut length of building cable lying slack on the floor
    /// (Blender batch `r10`: NM-B 12/2 with ground, its jacket yellow or
    /// white), its end bent up and stripped to its conductors; now and then
    /// an arc jumps between their bared ends, a brief blue-white flash, and,
    /// rarely, flares.
    fn tail_electric_wires(&mut self, cx: &mut PropsCtx) {
        use crate::assets::props::prop_part_as_modelled;
        // the jacket: 12 AWG's yellow, or 14's white, gone dull with dust
        let yellow = lambert!({ color: 0xb39b4d });
        let white = lambert!({ color: 0xc3beb0 });
        let black_ins = lambert!({ color: 0x2c2b29 });
        let white_ins = lambert!({ color: 0xbdb9ad });
        let copper = phong!({ color: 0x5c3421, specular: 0x8a5836, shininess: 40 });
        let paper = lambert!({ color: 0x9a8868 });
        // the arc's flash: a soft blue-white glow, white at its heart
        let spark_tex = canvas_tex_opt(64, 64, false, |ctx, w, h| {
            ctx.clear_rect(0, 0, w, h);
            let grad = ctx.create_radial_gradient(w / 2.0, h / 2.0, 0, w / 2.0, h / 2.0, w / 2.0);
            grad.add_color_stop(0, "rgba(255,255,255,1)");
            grad.add_color_stop(0.08, "rgba(236,243,255,0.95)");
            grad.add_color_stop(0.22, "rgba(160,195,255,0.65)");
            grad.add_color_stop(0.5, "rgba(105,150,255,0.24)");
            grad.add_color_stop(1, "rgba(90,130,255,0)");
            ctx.set_fill_style(&grad);
            ctx.fill_rect(0, 0, w, h);
        });
        let mut i = 0;
        while i < irand(2, 4) {
            let Some(c) = s_take(self, cx, false) else { break };
            let g = group();
            g.set_pos_v(&jitter_pos(self, &c, 0.0));
            g.rotation().y = rand1(PI * 2.0);
            // the old three curled wires' draws: each one's place and turn
            let mut turns = [0.0; 3];
            for t in turns.iter_mut() {
                let _ = (rand2(-0.05, 0.05), rand2(-0.05, 0.05));
                let (_, ry, _) = (rand2(-0.25, 0.25), rand2(0.0, 7.0), rand2(0.0, 7.0));
                *t = ry;
            }
            // the old copper tip's lean (kept for its draw)
            let _ = rand2(-0.6, 0.6);
            // which length (how it lies), and which cable
            let sheath = if turns[0] < 3.5 { "Real__electricCableSheathA" } else { "Real__electricCableSheathB" };
            let jacket = if turns[1] < 4.9 { &yellow } else { &white };
            for (name, mat) in [
                (sheath, jacket),
                ("Real__electricCableCoreBlack", &black_ins),
                ("Real__electricCableCoreWhite", &white_ins),
                ("Real__electricCableCopper", &copper),
                ("Real__electricCablePaper", &paper),
            ] {
                if let Some(geo) = prop_part_as_modelled(name) {
                    g.add(&mesh(geo, mat));
                }
            }
            // the arc between the bared ends (usually dark, flashes on)
            let spark = mesh(
                plane_geo!(0.11, 0.11),
                basic_mat!({
                    map: spark_tex.clone(), transparent: true, opacity: 0,
                    blending: AdditiveBlending, depthWrite: false,
                }),
            );
            spark.set_pos(CABLE_SPARK[0], CABLE_SPARK[1], CABLE_SPARK[2]);
            g.add(&spark);
            // small point light over the bared ends that pulses with the spark
            let spark_light = point_light!(0xdfe6ff, 0, 3, 2.0);
            spark_light.set_pos(0, 0.14, 0);
            g.add(&spark_light);
            self.world.add(&g);
            // Sparks fire on a randomised schedule; ~1/40 flashes escalate into a
            // full flare: white screen flash, crackle + thump, a brief sanity dip.
            let mut next_flash = rand2(0.5, 2.5);
            let mut flash_t = 0.0;
            let mut boom = false;
            self.animate(move |gm, dt, _t| {
                next_flash -= dt;
                if next_flash <= 0.0 {
                    next_flash = rand2(0.8, 3.5);
                    flash_t = rand2(0.06, 0.14);
                    let gp = g.pos();
                    let dp = hypot2(gp.x - gm.player.pos.x, gp.z - gm.player.pos.z);
                    boom = rnd() < 0.025 && dp < 22.0; // rare + audible
                    if boom {
                        // Big flare: extend visible flash, cue white screen flash + audio
                        flash_t = 0.28;
                        spark_flare(gm, &g, dp);
                    } else if dp < 14.0 && gm.ac.is_some() {
                        // brief crackle only if the player is close-ish
                        gm.noise_burst(0.05, 4200.0, 3.0, clamp(0.09 - dp * 0.005, 0.02, 0.09), "highpass", None, 0.0, 0.0);
                    }
                }
                if flash_t > 0.0 {
                    flash_t -= dt;
                    spark.mat().set_opacity(if boom { 1.0 } else { 0.9 });
                    spark_light.set_intensity(if boom { 5.5 } else { 1.6 });
                    // boom sparks scale up dramatically
                    let s = if boom { 1.0 + (0.28 - flash_t) * 4.0 } else { 1.0 };
                    spark.scale().set(s, s, s);
                    spark.look_at_v(&gm.camera.pos());
                } else {
                    spark.mat().set_opacity(0.0);
                    spark_light.set_intensity(0.0);
                    spark.scale().set(1, 1, 1);
                    boom = false;
                }
            });
            i += 1;
        }
    }

    /// HAUNTED: pillar candles burnt down on the floor (Blender batch
    /// `r10`: a 3 in pillar, its crater, the wax run down its side and spread
    /// round its foot, the wick), lit, each with a blow-out interactable. A
    /// candle blown out stays out: nothing you carry makes a flame
    /// (`docs/design/LORE.md`, *What still burns*). None where `flames_here` is
    /// false: their draws and their tiles kept.
    fn tail_haunted_candles(&mut self, cx: &mut PropsCtx) {
        use crate::assets::props::prop_part_as_modelled;
        if !flames_here(self) {
            let mut i = 0;
            while i < irand(2, 4) {
                let Some(c) = s_take(self, cx, false) else { break };
                jitter_pos(self, &c, 0.0);
                i += 1;
            }
            return;
        }
        // ivory paraffin, dusty; its pool of melted wax clearer and glossy
        let wax_mat = lambert!({ color: 0xcdc3aa });
        let pool_mat = phong!({ color: 0xb09c76, specular: 0x4c473c, shininess: 70 });
        // the flame: a teardrop's soft glow, blue at its foot, and its hot core
        let flame_mat = basic_mat!({
            map: flame_tex(false), transparent: true, opacity: 0.9,
            blending: AdditiveBlending, depthWrite: false,
        });
        let flame_core_mat = basic_mat!({
            map: flame_tex(true), transparent: true, opacity: 0.85,
            blending: AdditiveBlending, depthWrite: false,
        });
        // the two candles as they burnt: the wax lit through (its normals
        // turned up toward the flame), its pool, the pool's top
        let candles: Vec<Option<(Geo, Geo, f64)>> =
            [("Real__hauntedCandleBodyA", "Real__hauntedCandlePoolA"), ("Real__hauntedCandleBodyB", "Real__hauntedCandlePoolB")]
                .iter()
                .map(|(b, p)| {
                    let body = prop_part_as_modelled(b)?;
                    let pool = prop_part_as_modelled(p)?;
                    let top = pool.bounding_box().max.y;
                    Some((wax_lit(&body), pool, top))
                })
                .collect();
        let wick_geo = prop_part_as_modelled("Real__hauntedCandleWick").unwrap_or_else(|| box_geo!(0.002, 0.012, 0.002));
        let flame_geo = plane_geo!(0.028, 0.066);
        flame_geo.translate(0, 0.033, 0);
        let core_geo = plane_geo!(0.0105, 0.031);
        core_geo.translate(0, 0.0155, 0);
        let mut i = 0;
        while i < irand(2, 4) {
            let Some(c) = s_take(self, cx, false) else { break };
            let g = group();
            g.set_pos_v(&jitter_pos(self, &c, 0.0));
            // which candle, and which way round: from where it stands (no draw of its own)
            let at = g.pos();
            let k = spot_hash(at.x, at.z);
            let (body, pool, top) = candles[usize::from(k >= 0.55)]
                .clone()
                .unwrap_or_else(|| (cyl_geo!(0.038, 0.038, 0.1, 12), box_geo!(0.06, 0.001, 0.06), 0.1));
            g.rotation().y = (k * 997.0).fract() * PI * 2.0;
            g.add(&mesh(body, &wax_mat));
            g.add(&mesh(pool, &pool_mat));
            // the wick, a material of its own
            let wick = mesh(wick_geo.clone(), lambert!({ color: 0x1a1408, emissive: 0x000000 }));
            wick.position().y = top - 0.0004;
            g.add(&wick);
            // the flame over the wick's curled tip, turned to the eye about its own upright, and its hot core
            let flame = mesh(flame_geo.clone(), &flame_mat);
            flame.position().set(0.003, top + 0.0015, 0);
            g.add(&flame);
            let flame_core = mesh(core_geo.clone(), &flame_core_mat);
            flame_core.position().set(0.003, top + 0.006, 0);
            g.add(&flame_core);
            // warm point light
            let glow = point_light!(0xffa050, 1.4, 2.6, 1.8);
            glow.position().y = top + 0.15;
            g.add(&glow);
            // the wax that ran and spread on the floor round its foot, thin, soft at its edge
            let melt = mesh(plane_geo!(0.26, 0.26), wax_spill_mat());
            melt.rotation().x = -PI / 2.0;
            melt.position().y = 0.002;
            g.add(&melt);
            self.world.add(&g);
            let off = i as f64 * 0.9;
            // per-candle state shared by the animator and the interactable
            let ci = self.haunted_candles.len();
            let mut st = CandleState { lit: true, ..Default::default() };
            st.f.insert("fadeT".to_string(), 0.0);
            self.haunted_candles.push(st);
            let gp = g.pos();
            {
                let (glow, flame, flame_core) = (glow.clone(), flame.clone(), flame_core.clone());
                self.animate(move |gm, dt, t| candle_frame(gm, ci, off, &glow, &flame, &flame_core, dt, t));
            }
            // Interactable: press E near the candle to blow it out; it stays
            // out (nothing you carry makes a flame)
            let mut use_pos = gp;
            use_pos.y = EYE;
            self.interact(use_pos, 0.9, "[E] BLOW OUT", move |gm| !c_out(gm, ci), move |gm| candle_blow_out(gm, ci, gp));
            i += 1;
        }
    }
}

/// The rare spark flare (a spark's boom): audio, the white screen flash, the
/// sanity dip and the scorch decal (8 s fade, or 10% permanent with smoke).
fn spark_flare(g: &mut Game, sg: &Obj, dp: f64) {
    if g.ac.is_some() {
        g.noise_burst(0.15, 2800.0, 2.0, 0.22, "bandpass", None, 0.0, 0.0);
        g.noise_burst(0.10, 5000.0, 3.0, 0.14, "highpass", None, 0.02, 0.0);
        g.blip(90.0, 0.4, "sine", 0.20, Some(40.0), 0.02, 0.0);
    }
    // Full-screen white flash via the HUD's existing flash element
    g.ui.flash.set_style(
        "background",
        "radial-gradient(ellipse at center, rgba(255,255,240,0.7) 0%, rgba(255,240,200,0.35) 60%, transparent 100%)",
    );
    g.ui.flash.set_opacity(1.0);
    g.schedule(120.0, |g| {
        g.ui.flash.set_opacity(0.0);
        // restore the standard hurt-flash gradient after the pulse
        g.schedule(350.0, |g| {
            g.ui.flash.set_style("background", "radial-gradient(ellipse at center, rgba(120,0,0,0.55) 0%, rgba(60,0,0,0.9) 100%)");
        });
    });
    // brief sanity dip if close enough that it startles the player
    if dp < 10.0 {
        g.player.sanity = clamp(g.player.sanity - 4.0, 0.0, SAN_MAX);
    }
    g.show_msg("SPARKS FLARED!", 1.2);
    // Post-boom scorch decal on the floor beneath the wire bundle
    let scorch_tex = canvas_tex_opt(64, 64, false, |ctx, w, h| {
        ctx.set_fill_style("rgba(0,0,0,0)");
        ctx.fill_rect(0, 0, w, h);
        // charred core + faint burn ring
        for _ in 0..8 {
            let (cx, cy) = (w / 2.0 + rand2(-8.0, 8.0), h / 2.0 + rand2(-8.0, 8.0));
            let grad = ctx.create_radial_gradient(cx, cy, 1, cx, cy, rand2(12.0, 24.0));
            grad.add_color_stop(0, &format!("rgba(10,8,6,{})", rand2(0.65, 0.85)));
            grad.add_color_stop(1, "rgba(10,8,6,0)");
            ctx.set_fill_style(&grad);
            ctx.fill_rect(0, 0, w, h);
        }
        // a few outward sooty streaks
        ctx.set_stroke_style("rgba(20,15,10,0.55)");
        ctx.set_line_width(1.5);
        for s in 0..6 {
            let a = (s as f64 / 6.0) * PI * 2.0 + rand2(-0.2, 0.2);
            let r = rand2(18.0, 30.0);
            ctx.begin_path();
            ctx.move_to(w / 2.0, h / 2.0);
            ctx.line_to(w / 2.0 + a.cos() * r, h / 2.0 + a.sin() * r);
            ctx.stroke();
        }
    });
    let gp = sg.pos();
    let scorch = mesh(plane_geo!(0.60, 0.60), basic_mat!({ map: scorch_tex.clone(), transparent: true, depthWrite: false, opacity: 0.9 }));
    scorch.rotation().x = -PI / 2.0;
    scorch.rotation().z = rand2(0.0, PI * 2.0);
    scorch.set_pos(gp.x, 0.008, gp.z);
    g.world.add(&scorch);
    // 10% chance the scorch is permanent (rest of the level), otherwise it fades over 8 s
    if rnd() < 0.10 {
        scorch.mat().set_opacity(0.65);
        // brief smoke wisp so the "just happened" beat still lands
        let ps_smoke = mesh(
            plane_geo!(0.16, 0.32),
            basic_mat!({
                color: 0xa8a8a8, transparent: true, opacity: 0.42,
                blending: AdditiveBlending, depthWrite: false,
            }),
        );
        ps_smoke.set_pos(gp.x, 0.30, gp.z);
        g.world.add(&ps_smoke);
        let mut ps_t = 0.0;
        let sg1 = sg.clone();
        g.animate(move |g, sdt, _| {
            ps_t += sdt;
            let p = ps_t / 2.4;
            if p >= 1.0 {
                g.world.remove(&ps_smoke);
                return;
            }
            ps_smoke.position().y = 0.30 + p * 0.55;
            ps_smoke.position().x = sg1.pos().x + (ps_t * 3.8).sin() * 0.04;
            ps_smoke.scale().set(1.0 + p * 1.4, 1.0 + p * 1.6, 1.0);
            ps_smoke.mat().set_opacity(0.42 * (1.0 - p) * (1.0 - p * 0.4));
            look_at_level(&ps_smoke, &g.camera);
        });
        // secondary near-ground puff: quick fade, half the wisp's life
        let ps_puff = mesh(
            plane_geo!(0.22, 0.14),
            basic_mat!({
                color: 0x808080, transparent: true, opacity: 0.55,
                blending: AdditiveBlending, depthWrite: false,
            }),
        );
        // ±5 cm jitter so overlapping scorches don't stack identical puffs
        let puff_x_off = rand2(-0.05, 0.05);
        let puff_z_off = rand2(-0.05, 0.05);
        ps_puff.set_pos(gp.x + puff_x_off, 0.10, gp.z + puff_z_off);
        g.world.add(&ps_puff);
        let mut puff_t = 0.0;
        g.animate(move |g, sdt, _| {
            puff_t += sdt;
            let q = puff_t / 1.0;
            if q >= 1.0 {
                g.world.remove(&ps_puff);
                return;
            }
            ps_puff.position().y = 0.10 + q * 0.12;
            ps_puff.scale().set(1.0 + q * 1.8, 1.0 + q * 1.5, 1.0);
            ps_puff.mat().set_opacity(0.55 * (1.0 - q * q)); // fast quadratic fade
            look_at_level(&ps_puff, &g.camera);
        });
        // scorch cluster: 2-3 smaller charred fragments around the main scorch
        let fragment_count = irand(2, 3);
        for _ in 0..fragment_count {
            let frag_mat = basic_mat!({ map: scorch_tex.clone(), transparent: true, depthWrite: false, opacity: 0.35 });
            let frag = mesh(plane_geo!(0.22, 0.22), frag_mat);
            frag.rotation().x = -PI / 2.0;
            let angle = rand2(0.0, PI * 2.0);
            // radial "blast" alignment + small orientation jitter
            frag.rotation().z = angle + rand2(-0.3, 0.3);
            let dist = rand2(0.35, 0.55);
            // distance taper: fragments further out are smaller
            let dist_norm = (dist - 0.35) / 0.20;
            let size_scale = 1.0 - dist_norm * 0.35;
            frag.scale().set(size_scale, size_scale, 1.0);
            frag.set_pos(gp.x + angle.cos() * dist, 0.007, gp.z + angle.sin() * dist);
            g.world.add(&frag);
        }
    } else {
        let mut scorch_t = 0.0;
        g.animate(move |g, sdt, _| {
            scorch_t += sdt;
            let p = scorch_t / 8.0;
            if p >= 1.0 {
                g.world.remove(&scorch);
                return;
            }
            scorch.mat().set_opacity(0.9 * (1.0 - p));
        });
    }
}

/// A haunted candle's animator: the 0.6 s fade once it is blown out (then
/// a cold wick, for good); the LFO-synced flicker and the battery boost
/// while lit. The flame turns to the eye about its own upright (a flame
/// stands up, whatever the angle it is seen from).
fn candle_frame(g: &mut Game, ci: usize, off: f64, glow: &Obj, flame: &Obj, flame_core: &Obj, dt: f64, t: f64) {
    if c_out(g, ci) {
        // fade over 0.6 s, then a cold wick
        if cf(g, ci, "fadeT") < 0.6 {
            let fade_t = cf(g, ci, "fadeT") + dt;
            set_cf(g, ci, "fadeT", fade_t);
            let k = clamp(1.0 - fade_t / 0.6, 0.0, 1.0);
            glow.set_intensity(1.4 * k * 0.5);
            flame.mat().set_opacity(0.9 * k);
            flame_core.mat().set_opacity(0.85 * k);
            flame.scale().y = 1.0 + k * 0.5;
            flame_core.scale().y = 1.0 + k * 0.7; // core stretches slightly more
            look_at_level(flame, &g.camera);
            look_at_level(flame_core, &g.camera);
        } else {
            glow.set_intensity(0.0);
            flame.set_visible(false);
            flame_core.set_visible(false);
        }
        return;
    }
    // The safe-room LFO breath, weather-scaled amplitude
    let lfo_hz = g.audio.safe_room_lfo.as_ref().map(|o| o.frequency().value()).filter(|v| *v != 0.0 && !v.is_nan()).unwrap_or(0.35);
    let flame_amp =
        flame_breath_amp_by_weather(g.weather_key()).or(flame_breath_amp_by_weather("normal")).unwrap() * g.descent_boost_mul(0.4);
    let room_breath = 1.0 + (t * lfo_hz * PI * 2.0 * g.descent_freq_boost() + off * 0.6).sin() * flame_amp;
    let fk = (0.85 + (t * 12.0 + off).sin() * 0.12 + (t * 21.3 + off).sin() * 0.05) * room_breath;
    // Candles brighten a little as the flashlight dies
    let candle_boost = if g.player.flashlight_on { 1.0 + 0.15 * (1.0 - g.beam_battery_scale(0.0)) } else { 1.0 };
    // Descent dim
    glow.set_intensity(1.4 * fk * candle_boost * (0.65 + 0.35 * g.level_depth_mul()));
    flame.scale().y = 1.0 + (t * 10.0 + off).sin() * 0.18;
    look_at_level(flame, &g.camera);
    // Inner core: faster, different-phase wobble
    flame_core.scale().y = 1.0 + (t * 13.0 + off * 1.7).sin() * 0.24;
    flame_core.scale().x = 1.0 + (t * 17.0 + off * 2.1).sin() * 0.06;
    look_at_level(flame_core, &g.camera);
    if flame_core.visible() != flame.visible() {
        flame_core.set_visible(flame.visible());
    }
}

/// `[E] BLOW OUT`
fn candle_blow_out(g: &mut Game, ci: usize, gp: Vec3) {
    set_out(g, ci, true);
    g.candle_blowout_tally += 1.0;
    // soft breath / puff, from the candle
    if g.ac.is_some() {
        g.play_from(gp.x, 0.2, gp.z, 1.5, 1.0, 0.7, |g| g.noise_burst(0.22, 3600.0, 1.5, 0.14, "highpass", None, 0.0, 0.0));
    }
    // layered smoke planes; 4 layers when wet, 2 when still, else 3
    let smoke_layers: Vec<SmokeLayer> = match g.weather_key() {
        "wet" => {
            let mut v = SMOKE_LAYERS_BASE.to_vec();
            v.push(SMOKE_EXTRA_WET);
            v
        }
        "still" => SMOKE_LAYERS_BASE[..2].to_vec(),
        _ => SMOKE_LAYERS_BASE.to_vec(),
    };
    let smoke_start = 0.30;
    // smoke picks up a subtle tint from the weather
    let smoke_color = g.weather_pick(
        |k| {
            Some(match k {
                "wet" => 0xb0c8dcu32,
                "electric" => 0xc0b8e0,
                "haunted" => 0xd8b4b0,
                "derelict" => 0xd8bfa0,
                "still" => 0xc8c4b8,
                "normal" => 0xd8d8d8,
                _ => return None,
            })
        },
        0xd8d8d8,
    );
    for layer in smoke_layers {
        let smoke = mesh(
            plane_geo!(0.14, 0.28),
            basic_mat!({
                color: smoke_color, transparent: true, opacity: 0.35,
                blending: AdditiveBlending, depthWrite: false,
            }),
        );
        smoke.set_pos(gp.x + layer.xoff, smoke_start, gp.z);
        g.world.add(&smoke);
        let mut smoke_t = 0.0;
        g.animate(move |g, dt, _| {
            smoke_t += dt;
            let p = smoke_t / layer.life;
            if p >= 1.0 {
                g.world.remove(&smoke);
                return;
            }
            // rise, then droop back by up to `droop` metres past p = 0.6
            let droop_p = if layer.droop != 0.0 { 0f64.max((p - 0.6) / 0.4) } else { 0.0 };
            smoke.position().y = smoke_start + p * layer.rise - droop_p * layer.droop;
            // wobble: amplitude grows while drooping; weather scales amplitude and frequency
            let wk = g.weather_key();
            let wobble_weather_mul = if wk == "wet" {
                1.35
            } else if wk == "still" {
                0.75
            } else {
                1.0
            };
            let wobble_freq_mul = if wk == "wet" {
                0.75
            } else if wk == "still" {
                1.30
            } else {
                1.0
            };
            let wobble_amp = 0.03 * wobble_weather_mul * (1.0 + droop_p * 1.5);
            smoke.position().x = gp.x + layer.xoff + (smoke_t * layer.wobble * wobble_freq_mul + layer.phase).sin() * wobble_amp;
            smoke.scale().set(1.0 + p * layer.scale_xk, 1.0 + p * layer.scale_yk, 1.0);
            smoke.mat().set_opacity(0.35 * (1.0 - p) * (1.0 - p * 0.5));
            look_at_level(&smoke, &g.camera);
        });
    }
    // Tail beat ~2.8 s in (TAIL_BEAT_SPECS by weather), where the candle is
    if g.ac.is_some() {
        let specs = tail_beat_specs(g.weather_key()).or(tail_beat_specs("normal")).unwrap();
        g.schedule(2800.0, move |g| {
            g.play_from(gp.x, 0.2, gp.z, 1.5, 1.0, 0.7, |g| {
                for &(f, jit, dur, ty, amp, delay_ms) in specs {
                    let extra_sec = delay_ms / 1000.0;
                    g.blip(f + rand2(-jit, jit), dur, ty, amp, None, extra_sec, 0.0);
                }
            });
        });
        // your asides: once the two notes have rung out
        let end = specs.iter().map(|s| s.5 / 1000.0 + s.2).fold(0.0, f64::max);
        g.aside_later(2800.0 + (end + 0.3) * 1000.0, crate::audio::asides::Use::Uncanny, "candle");
    }
}

/// A deterministic lumpiness in [-1, 1] at a point (a few crossed waves; no
/// random draws), so the silk thins unevenly
fn cobweb_lumps(x: f64, y: f64, z: f64) -> f64 {
    let a = (x * 23.0 + y * 7.0).sin() * (z * 19.0 - x * 5.0).cos();
    let b = (y * 31.0 + z * 11.0 + 1.3).sin() * (x * 13.0 + 0.7).sin();
    (0.6 * a + 0.4 * b).clamp(-1.0, 1.0)
}

/// A cobweb's sheets made ready to draw (the parts' frames: game/washroom_mirror_cobwebs_and_office_plant.py): how
/// thick the silk is at each point (vertex alpha: thickest in the corner, or
/// along the wall's top, thinning unevenly outward, frayed to nothing at
/// every edge of a sheet, so no sheet shows its outline), and the silk's map laid on at
/// its real scale (a repeat every 0.3 m, from the sheet's own facing) rather
/// than stretched over the part's box
fn cobweb_sheet_geo(geo: &Geo, corner: bool) -> Geo {
    let g = geo.duplicate();
    {
        let mut gg = g.g();
        let n = gg.position.len() / 3;
        // the sheets' edges (an edge of one triangle only, the vertices
        // welded by where they are), where the silk frays out to nothing
        let key = |i: usize| -> (i64, i64, i64) {
            let q = |v: f32| (v as f64 * 1e4).round() as i64;
            (q(gg.position[i * 3]), q(gg.position[i * 3 + 1]), q(gg.position[i * 3 + 2]))
        };
        let keys: Vec<(i64, i64, i64)> = (0..n).map(key).collect();
        let mut edges: std::collections::BTreeMap<((i64, i64, i64), (i64, i64, i64)), u32> = std::collections::BTreeMap::new();
        if let Some(ix) = gg.index.as_ref() {
            for t in ix.chunks(3) {
                for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                    let (ka, kb) = (keys[a as usize], keys[b as usize]);
                    *edges.entry(if ka < kb { (ka, kb) } else { (kb, ka) }).or_insert(0) += 1;
                }
            }
        }
        // how far each point is from a sheet's edge, along the sheet (metres,
        // relaxed over its edges until nothing shortens; at most FRAY)
        const FRAY: f64 = 0.035;
        let mut dist: std::collections::BTreeMap<(i64, i64, i64), f64> = keys.iter().map(|k| (*k, FRAY)).collect();
        for ((a, b), _) in edges.iter().filter(|(_, &c)| c == 1) {
            dist.insert(*a, 0.0);
            dist.insert(*b, 0.0);
        }
        let len = |a: &(i64, i64, i64), b: &(i64, i64, i64)| {
            (((a.0 - b.0).pow(2) + (a.1 - b.1).pow(2) + (a.2 - b.2).pow(2)) as f64).sqrt() * 1e-4
        };
        loop {
            let mut changed = false;
            for (a, b) in edges.keys() {
                let l = len(a, b);
                let (da, db) = (dist[a], dist[b]);
                if da + l < db - 1e-9 {
                    dist.insert(*b, da + l);
                    changed = true;
                } else if db + l < da - 1e-9 {
                    dist.insert(*a, db + l);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let mut col = Vec::with_capacity(n * 4);
        let mut uv = Vec::with_capacity(n * 2);
        for i in 0..n {
            let (x, y, z) = (gg.position[i * 3] as f64, gg.position[i * 3 + 1] as f64, gg.position[i * 3 + 2] as f64);
            let f = dist[&keys[i]] / FRAY;
            let fray = f * f * (3.0 - 2.0 * f);
            let (nx, ny, nz) = (gg.normal[i * 3].abs(), gg.normal[i * 3 + 1].abs(), gg.normal[i * 3 + 2].abs());
            let lump = cobweb_lumps(x, y, z);
            // the dust packed into the angle itself (thick, gone within a hand's
            // breadth), and the veils that sag from it (thin, thinning outward)
            let a = if corner {
                let r = (x * x + y * y + z * z).sqrt();
                1.2 - (r / 0.52).powf(1.8) * (0.9 + 0.25 * lump)
            } else {
                let r = (y * y + z * z).sqrt();
                let ends = (x.abs() / 0.3).min(1.0);
                (1.2 - (r / 0.38).powf(1.8) * (0.9 + 0.25 * lump)) * (1.0 - 0.8 * ends.powi(3))
            };
            col.extend_from_slice(&[1.0, 1.0, 1.0, (a.clamp(0.0, 1.0) * fray) as f32]);
            let (u, v) = if ny >= nx && ny >= nz {
                (x, z)
            } else if nx >= nz {
                (z, y)
            } else {
                (x, y)
            };
            uv.extend_from_slice(&[(u / 0.3) as f32, (v / 0.3) as f32]);
        }
        gg.color = col;
        gg.color_size = 4;
        gg.uv = uv;
        gg.touch();
    }
    cobweb_lit(&g, corner, true)
}

/// Silk scatters the light through it: a web is lit as a surface turned out
/// into the room from its corner (or its wall), whichever side of a thread
/// or a sheet is seen (`both`: each sheet's triangles both ways round)
fn cobweb_lit(geo: &Geo, corner: bool, both: bool) -> Geo {
    let g = if both { geo.clone() } else { geo.duplicate() };
    {
        let mut gg = g.g();
        let n = if corner { [0.7f32, -0.1, 0.7] } else { [0.0f32, -0.1, 0.99] };
        let count = gg.position.len() / 3;
        gg.normal = (0..count).flat_map(|_| n).collect();
        if both {
            if let Some(ix) = gg.index.as_mut() {
                let back: Vec<u32> = ix.chunks(3).flat_map(|t| [t[0], t[2], t[1]]).collect();
                ix.extend(back);
            }
        }
        gg.touch();
    }
    g
}

/// The cobwebs' silk, a repeating patch 0.3 m across: a grey film of fine
/// silk, threads every way over it (most in two or three runs, as a tangle
/// is laid), dust caught in it in soft drifts, clumps and specks, the film
/// torn through in places (transparent there)
fn cobweb_silk_tex() -> Tex {
    canvas_tex(256, 256, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        // everything is drawn again across the edges it crosses, so the patch repeats
        let wraps = |x: f64, y: f64, r: f64| -> Vec<(f64, f64)> {
            let mut out = vec![];
            for dy in [-h, 0.0, h] {
                for dx in [-w, 0.0, w] {
                    let (px, py) = (x + dx, y + dy);
                    if px + r >= 0.0 && px - r <= w && py + r >= 0.0 && py - r <= h {
                        out.push((dx, dy));
                    }
                }
            }
            out
        };
        // the film: fine silk too fine to see a thread of, grey with dust
        ctx.set_fill_style("rgba(128,123,114,0.5)");
        ctx.fill_rect(0, 0, w, h);
        // dust drifts, heavier in some places than others
        for _ in 0..34 {
            let (x, y, r) = (rnd() * w, rnd() * h, 8.0 + rnd() * 34.0);
            let a = 0.3 + 0.45 * rnd();
            let (sq, rot) = (0.35 + 0.65 * rnd(), rnd() * 3.0);
            let stops = [
                (0.0, format!("rgba(112,106,97,{a:.3})")),
                (0.6, format!("rgba(112,106,97,{:.3})", a * 0.5)),
                (1.0, "rgba(112,106,97,0)".into()),
            ];
            for (dx, dy) in wraps(x, y, r) {
                soft_blob(ctx, x + dx, y + dy, r, sq, rot, &stops);
            }
        }
        // the threads, catching the light
        ctx.set_line_cap("round");
        let runs = [rnd() * 3.2, rnd() * 3.2, rnd() * 3.2];
        for i in 0..520 {
            let a = if i % 4 == 3 { rnd() * 6.3 } else { runs[i % 3] + (rnd() - 0.5) * 0.6 };
            let (x, y) = (rnd() * w, rnd() * h);
            let l = 10.0 + rnd() * rnd() * 150.0;
            let (x1, y1) = (x + a.cos() * l, y + a.sin() * l);
            let bend = (rnd() - 0.5) * l * 0.3;
            let (mx, my) = ((x + x1) / 2.0 - a.sin() * bend, (y + y1) / 2.0 + a.cos() * bend);
            let light = rnd() < 0.7;
            let (c, al) = if light { (206, 0.3 + 0.45 * rnd()) } else { (120, 0.35 + 0.4 * rnd()) };
            let lw = 0.5 + 0.9 * rnd() * rnd();
            for (dx, dy) in wraps((x + x1) / 2.0, (y + y1) / 2.0, l) {
                ctx.set_stroke_style(format!("rgba({c},{},{},{al:.3})", c - 4, c - 11));
                ctx.set_line_width(lw);
                ctx.begin_path();
                ctx.move_to(x + dx, y + dy);
                ctx.quadratic_curve_to(mx + dx, my + dy, x1 + dx, y1 + dy);
                ctx.stroke();
            }
        }
        // clumps of fluff and specks of grit
        for i in 0..340 {
            let (x, y) = (rnd() * w, rnd() * h);
            let r = if i % 9 == 0 { 2.0 + rnd() * 4.0 } else { 0.4 + rnd() * rnd() * 2.2 };
            let v = if rnd() < 0.75 { 96 } else { 170 };
            let al = 0.45 + 0.45 * rnd();
            for (dx, dy) in wraps(x, y, r) {
                ctx.set_fill_style(format!("rgba({v},{},{},{al:.2})", v - 5, v - 12));
                ctx.begin_path();
                ctx.arc(x + dx, y + dy, r, 0, 7);
                ctx.fill();
            }
        }
        // torn through
        ctx.set_global_composite_operation("destination-out");
        for _ in 0..9 {
            let (x, y, rx, ry, rot) = (rnd() * w, rnd() * h, 6.0 + rnd() * 22.0, 3.0 + rnd() * 10.0, rnd() * 3.0);
            let stops = [(0.0, "rgba(0,0,0,0.85)".to_string()), (0.7, "rgba(0,0,0,0.6)".into()), (1.0, "rgba(0,0,0,0)".into())];
            for (dx, dy) in wraps(x, y, rx) {
                soft_blob(ctx, x + dx, y + dy, rx, ry / rx, rot, &stops);
            }
        }
        ctx.set_global_composite_operation("source-over");
    })
}

/// A soft blob painted on a canvas: a radial fade of radius `r` squashed to
/// `k` of it across and turned by `turn`, the canvas squashed so the fade is
/// never cut off by its outline
fn soft_blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, stops: &[(f64, String)]) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(turn);
    ctx.scale(1.0, k);
    let g = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    for (at, c) in stops {
        g.add_color_stop(*at, c);
    }
    ctx.set_fill_style(&g);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

// ------------------------------------------------ what the weather leaves (r10)
//
// The derelict floors' fallen tile, the electric floors' cable, the haunted
// floors' candles (Blender batch `r10`, `docs/research/props-remodelling/what_a_floors_weather_leaves.md`)
// and the wet floors' puddles (painted as batch `r07` paints them).

/// A mineral-fibre lay-in tile's side and thickness (24 in, 5/8 in: `game/what_a_floors_weather_leaves.py`)
const TILE_SIDE: f64 = 0.610;
const TILE_THICK: f64 = 0.0159;
/// Where a cut cable's bared ends nearly meet, in its parts' frame (`game/what_a_floors_weather_leaves.py` `SPARK`)
const CABLE_SPARK: [f64; 3] = [0.0, 0.068, 0.0];

/// A rectangle on the floor (x, z): its middle, its own x turned by `turn`
/// (as an object's y rotation turns it) and its half sides along its x and z
#[derive(Clone, Copy, Debug)]
struct Rect2 {
    c: (f64, f64),
    u: (f64, f64),
    h: (f64, f64),
}
impl Rect2 {
    fn new(x: f64, z: f64, turn: f64, hx: f64, hz: f64) -> Rect2 {
        Rect2 { c: (x, z), u: (turn.cos(), -turn.sin()), h: (hx, hz) }
    }
    /// Whether two rectangles overlap (no axis of either separates them)
    fn overlaps(&self, o: &Rect2) -> bool {
        let axes = |r: &Rect2| [r.u, (-r.u.1, r.u.0)];
        let reach = |r: &Rect2, a: (f64, f64)| {
            let [u, v] = axes(r);
            r.h.0 * (u.0 * a.0 + u.1 * a.1).abs() + r.h.1 * (v.0 * a.0 + v.1 * a.1).abs()
        };
        let d = (o.c.0 - self.c.0, o.c.1 - self.c.1);
        axes(self).into_iter().chain(axes(o)).all(|a| (d.0 * a.0 + d.1 * a.1).abs() < reach(self, a) + reach(o, a))
    }
}

/// A copy of `geo` (a part as modelled) with its map laid flat on it from
/// above at its real scale: `metres` a repeat, starting at `off`
fn planar_uv(geo: &Geo, metres: f64, off: f64) -> Geo {
    let g = geo.duplicate();
    {
        let mut gg = g.g();
        gg.uv =
            gg.position.chunks_exact(3).flat_map(|p| [(p[0] as f64 / metres + off) as f32, (p[2] as f64 / metres + off) as f32]).collect();
        gg.touch();
    }
    g
}

/// A copy of a candle's wax (`geo`) lit as wax is: the flame's light comes
/// through it, so its sides are lit as if they faced up toward the flame
/// (each normal turned up), and go dark with it; the wax spread round its
/// foot is half in the candle's own shadow (turned partly away from the
/// flame above)
fn wax_lit(geo: &Geo) -> Geo {
    let g = geo.duplicate();
    {
        let mut gg = g.g();
        let Geometry { position, normal, .. } = &mut *gg;
        for (p, n) in position.chunks_exact(3).zip(normal.chunks_exact_mut(3)) {
            let r = (p[0] * p[0] + p[2] * p[2]).sqrt();
            let (x, y, z) = if p[1] < 0.0035 && r > 0.0385 { (p[0] / r * 0.5, 0.87, p[2] / r * 0.5) } else { (n[0], n[1] + 0.45, n[2]) };
            let l = (x * x + y * y + z * z).sqrt().max(1e-6);
            n[0] = x / l;
            n[1] = y / l;
            n[2] = z / l;
        }
        gg.touch();
    }
    g
}

/// A fixed number in [0, 1) for a spot on the floor (to the millimetre): a
/// choice made from where a thing was put, with no draw of its own
fn spot_hash(x: f64, z: f64) -> f64 {
    hash01(0x51a7_c0de, (x * 1000.0).round() as i32, (z * 1000.0).round() as i32)
}

// The face of a fine-fissured mineral-fibre tile, the whole tile on 512 px:
// its white paint yellowed unevenly over the years, the short fissures
// punched in it every way and its pinholes, and the leak that brought it
// down: a ring stain (an older one inside it), its tide line dark and sharp
// on its outside, the wash inside it
// (painted once, the first time a derelict floor needs it, from its own
// stream: the same every time, and shared by every fallen tile)
crate::lazy_static_tex!(
    tile_face_tex,
    canvas_tex_opt(512, 512, false, |ctx, w, h| {
        ctx.set_fill_style("rgb(204,197,179)");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..24 {
            let (x, y, r) = (rnd() * w, rnd() * h, 40.0 + rnd() * 130.0);
            let a = 0.05 + 0.09 * rnd();
            let tone = if rnd() < 0.6 { "170,152,112" } else { "224,219,204" };
            let (k, turn) = (0.5 + 0.5 * rnd(), rnd() * 3.0);
            soft_blob(ctx, x, y, r, k, turn, &[(0.0, format!("rgba({tone},{a:.3})")), (1.0, format!("rgba({tone},0)"))]);
        }
        // the leak
        let (sx, sy, sr) = (w * (0.2 + 0.6 * rnd()), h * (0.2 + 0.6 * rnd()), w * (0.14 + 0.12 * rnd()));
        let ph = [rnd() * 6.3, rnd() * 6.3, rnd() * 6.3];
        let ring = |r0: f64| -> Vec<(f64, f64)> {
            (0..=96)
                .map(|k| {
                    let a = k as f64 / 96.0 * PI * 2.0;
                    let r = r0 * (1.0 + 0.13 * (3.0 * a + ph[0]).sin() + 0.07 * (5.0 * a + ph[1]).sin() + 0.035 * (11.0 * a + ph[2]).sin());
                    (sx + a.cos() * r, sy + a.sin() * r)
                })
                .collect()
        };
        let trace = |ctx: &mut Canvas, pts: &[(f64, f64)]| {
            ctx.begin_path();
            ctx.move_to(pts[0].0, pts[0].1);
            for p in &pts[1..] {
                ctx.line_to(p.0, p.1);
            }
            ctx.close_path();
        };
        trace(ctx, &ring(sr));
        ctx.set_fill_style("rgba(158,124,70,0.11)");
        ctx.fill();
        let inner = 0.5 + 0.2 * rnd();
        for r0 in [sr, sr * inner] {
            // what the water carried, left at its rim: dark at the rim, fading inward
            for (lw, al) in [(9.0, 0.05), (4.0, 0.12), (1.6, 0.36)] {
                trace(ctx, &ring(r0 - lw * 0.5));
                ctx.set_stroke_style(format!("rgba(124,88,42,{al})"));
                ctx.set_line_width(lw);
                ctx.stroke();
            }
        }
        // the fissures
        ctx.set_line_cap("round");
        for _ in 0..3200 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * 6.3;
            let l = 1.5 + rnd() * rnd() * 6.5;
            let bend = (rnd() - 0.5) * l * 0.8;
            let (x1, y1) = (x + a.cos() * l, y + a.sin() * l);
            let (mx, my) = ((x + x1) / 2.0 - a.sin() * bend, (y + y1) / 2.0 + a.cos() * bend);
            ctx.set_stroke_style(format!("rgba(112,104,88,{:.2})", 0.22 + 0.3 * rnd()));
            ctx.set_line_width(0.6 + rnd() * 0.7);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.quadratic_curve_to(mx, my, x1, y1);
            ctx.stroke();
        }
        // the pinholes
        for _ in 0..2400 {
            ctx.set_fill_style(format!("rgba(112,104,88,{:.2})", 0.2 + 0.3 * rnd()));
            ctx.begin_path();
            ctx.arc(rnd() * w, rnd() * h, 0.35 + rnd() * 0.5, 0, 7);
            ctx.fill();
        }
    })
);

// The tile's core and back, a repeat 0.15 m across: mineral wool and
// perlite pressed together, grey-tan, its fibres and grit, and the dust
// that settled on its back over the years above the ceiling
// (painted once and shared, as the face is)
crate::lazy_static_tex!(
    tile_core_tex,
    canvas_tex(128, 128, |ctx, w, h| {
        ctx.set_fill_style("rgb(158,149,131)");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..10 {
            let (x, y, r) = (rnd() * w, rnd() * h, 14.0 + rnd() * 30.0);
            let a = 0.08 + 0.1 * rnd();
            soft_blob(
                ctx,
                x,
                y,
                r,
                0.6 + 0.4 * rnd(),
                rnd() * 3.0,
                &[(0.0, format!("rgba(126,122,114,{a:.3})")), (1.0, "rgba(126,122,114,0)".into())],
            );
        }
        for _ in 0..1300 {
            let v = if rnd() < 0.55 { 196 } else { 112 };
            ctx.set_fill_style(format!("rgba({v},{},{},{:.2})", v - 6, v - 18, 0.25 + 0.4 * rnd()));
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
        ctx.set_line_cap("round");
        ctx.set_line_width(0.6);
        for _ in 0..260 {
            let (x, y, a, l) = (rnd() * w, rnd() * h, rnd() * 6.3, 2.0 + rnd() * 5.0);
            ctx.set_stroke_style(format!("rgba(214,208,190,{:.2})", 0.2 + 0.3 * rnd()));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
);

// The dust a fallen tile threw out where it broke, on a plane 1.5 m
// square: the powder knocked out of it, thickest in the middle, thrown out
// along the floor in streaks, crumbs and grit fewer further out
// (painted once and shared, each fallen tile's turned its own way)
crate::lazy_static_tex!(
    debris_dust_tex,
    mark_tex(256.0, 256.0, [0.8, 0.78, 0.73], |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        for _ in 0..22 {
            let (a, d) = (rnd() * PI * 2.0, rnd() * rnd() * w * 0.2);
            let r = w * (0.06 + 0.11 * rnd());
            let al = 0.1 + 0.16 * rnd();
            let (k, turn) = (0.5 + 0.5 * rnd(), rnd() * 3.0);
            let stops = [
                (0.0, format!("rgba(206,201,189,{al:.3})")),
                (0.6, format!("rgba(206,201,189,{:.3})", al * 0.4)),
                (1.0, "rgba(206,201,189,0)".into()),
            ];
            soft_blob(ctx, cx + a.cos() * d, cy + a.sin() * d, r, k, turn, &stops);
        }
        for _ in 0..30 {
            let a = rnd() * PI * 2.0;
            let (d0, l) = (w * (0.06 + 0.1 * rnd()), w * (0.08 + 0.16 * rnd()));
            let k = 0.12 + 0.1 * rnd();
            let stops = [(0.0, "rgba(200,195,183,0.15)".to_string()), (1.0, "rgba(200,195,183,0)".into())];
            soft_blob(ctx, cx + a.cos() * (d0 + l / 2.0), cy + a.sin() * (d0 + l / 2.0), l / 2.0, k, a, &stops);
        }
        for _ in 0..700 {
            let (a, d) = (rnd() * PI * 2.0, w * 0.42 * rnd().powf(1.6));
            // a crumb: a few specks, not a disc
            let (x, y) = (cx + a.cos() * d, cy + a.sin() * d);
            let v = if rnd() < 0.8 { [214, 208, 194] } else { [150, 141, 122] };
            ctx.set_fill_style(format!("rgba({},{},{},{:.2})", v[0], v[1], v[2], 0.35 + 0.45 * rnd()));
            for _ in 0..1 + (rnd() * 3.0) as usize {
                let (ox, oy, sz) = ((rnd() - 0.5) * 1.6, (rnd() - 0.5) * 1.6, 0.5 + rnd() * rnd() * 1.2);
                ctx.fill_rect(x + ox, y + oy, sz, sz * (0.5 + rnd()));
            }
        }
    })
);

/// The wax a candle shed on the floor round its foot, on a plane 0.26 m
/// square: a thin skin of it, run further one way, soft at its edge, and the
/// drops that fell off it
fn wax_spill_mat() -> Mat {
    let tex = mark_tex(128.0, 128.0, [0.8, 0.77, 0.68], |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        let a0 = rnd() * PI * 2.0;
        for _ in 0..8 {
            let (a, d) = (a0 + (rnd() - 0.5) * 1.6, w * 0.1 * rnd());
            let r = w * (0.17 + 0.08 * rnd());
            let (k, turn) = (0.6 + 0.4 * rnd(), rnd() * 3.0);
            let stops =
                [(0.0, "rgba(210,200,174,0.3)".to_string()), (0.55, "rgba(210,200,174,0.2)".into()), (1.0, "rgba(210,200,174,0)".into())];
            soft_blob(ctx, cx + a.cos() * d, cy + a.sin() * d, r, k, turn, &stops);
        }
        for _ in 0..4 {
            let (a, d) = (a0 + (rnd() - 0.5) * 2.4, w * (0.26 + 0.14 * rnd()));
            let r = 1.2 + rnd() * 1.8;
            let stops =
                [(0.0, "rgba(212,202,176,0.45)".to_string()), (0.6, "rgba(212,202,176,0.3)".into()), (1.0, "rgba(212,202,176,0)".into())];
            soft_blob(ctx, cx + a.cos() * d, cy + a.sin() * d, r, 0.7 + 0.3 * rnd(), rnd() * 3.0, &stops);
        }
    });
    mark_mat(&tex)
}

/// A candle flame as it is seen, on a plane whose foot is the flame's: a
/// teardrop, round at its foot and drawn up to a point, yellow-white through
/// its middle, orange at its edge and its tip, blue in a cup round the dark
/// of the wick at its foot, a faint glow round it; `core`: its hot middle
/// alone
fn flame_tex(core: bool) -> Tex {
    canvas_tex_opt(32, 64, false, move |ctx, w, h| {
        paint_px(ctx, w, h, |i, j| {
            let u = (i as f64 + 0.5) / w * 2.0 - 1.0;
            let v = 1.0 - (j as f64 + 0.5) / h;
            // along it, from its foot (t = 0) to its tip (t = 1); its half width there
            let (v0, v1, half) = if core { (0.1, 0.86, 0.6) } else { (0.07, 0.84, 0.46) };
            let t = (v - v0) / (v1 - v0);
            let hw = if t <= 0.0 || t >= 1.0 {
                0.0
            } else if t < 0.3 {
                half * (t / 0.3).sqrt()
            } else {
                half * ((1.0 - t) / 0.7).powf(1.2)
            };
            let q = if hw > 1e-6 { u.abs() / hw } else { 9.0 };
            let body = 1.0 - sstep(0.55, 1.05, q);
            if core {
                return [1.0, 0.97, 0.88, body * sstep(0.0, 0.2, t) * (1.0 - sstep(0.7, 1.0, t))];
            }
            let edge = sstep(0.3, 1.0, q).max(sstep(0.55, 1.0, t));
            let (r, g, b) = (1.0, 0.9 - 0.32 * edge, 0.7 - 0.52 * edge);
            let foot = (1.0 - sstep(0.04, 0.24, t)) * (0.35 + 0.65 * sstep(0.2, 0.8, q));
            let dark = (1.0 - sstep(0.0, 0.3, t)) * (1.0 - sstep(0.0, 0.5, q)) * 0.6;
            let (r, g, b) = (r + (0.35 - r) * foot, g + (0.5 - g) * foot, b + (1.0 - b) * foot);
            let a = body * (1.0 - dark) * (1.0 - 0.45 * foot);
            // the glow round it, fading out well inside the plane's edges
            let d = (q - 1.0).max(0.0) * hw.max(0.2) + (v0 - v).max(0.0) * 1.5 + (v - v1).max(0.0) * 1.2;
            let halo = 0.14 * (-(d / 0.22).powi(2)).exp();
            [r, g, b, a + halo * (1.0 - a)]
        });
    })
}

/// What the floor of tile (x, z) is: 0 carpet (or carpet tiles), 1 concrete,
/// 2 glazed tile (as `placers_kit::wet_floor::floor_kind` reads it)
fn floor_kind(g: &Game, x: i32, z: i32) -> u8 {
    if g.region_flags_at(x, z) & crate::visuals::lighting::RF_MOSS != 0 {
        return 0;
    }
    match g.zone_ref(x, z).map(|zn| zn.floor_mat) {
        Some("concrete") => 1,
        Some("tile") => 2,
        _ => 0,
    }
}

// The puddle's painter and what it is painted with, as batch `r07` wrote
// them for `place_floor_puddles` (`placers_core`; a placer crate's, which the
// game crate cannot use)

/// A hash of (seed, i, j) in [0, 1)
fn hash01(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

/// Smooth value noise in [0, 1]
fn vnoise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (hash01(seed, i, j), hash01(seed, i + 1, j));
    let (c, d) = (hash01(seed, i, j + 1), hash01(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// Fractal value noise of `oct` octaves, in [0, 1]
fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * vnoise(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}

/// Smoothstep from `e0` to `e1` (either way round)
fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The smooth minimum of two distances, blended over `k`
fn smin(a: f64, b: f64, k: f64) -> f64 {
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

/// How a water mark shades at signed distance `d` (m, < 0 inside) from its
/// rim: the rim sharp on its outside (over `aa`, about a texel), fading
/// inward over `ew`
fn tide(d: f64, aa: f64, ew: f64) -> f64 {
    (0.5 - d / aa).clamp(0.0, 1.0) * (d.min(0.0) / ew).exp()
}

/// A pixel's straight sRGB colour and alpha, built up by laying marks over it
#[derive(Clone, Copy)]
struct Px {
    c: [f64; 3],
    a: f64,
}
impl Px {
    const CLEAR: Px = Px { c: [0.0; 3], a: 0.0 };
    /// `c` at alpha `a` laid over what is there
    fn lay(&mut self, c: [f64; 3], a: f64) {
        let a = a.clamp(0.0, 1.0);
        let out = a + self.a * (1.0 - a);
        if out > 1e-6 {
            for (k, v) in self.c.iter_mut().enumerate() {
                *v = (c[k] * a + *v * self.a * (1.0 - a)) / out;
            }
        }
        self.a = out;
    }
    fn rgba(self) -> [f64; 4] {
        [self.c[0], self.c[1], self.c[2], self.a]
    }
}

/// Paints the canvas pixel by pixel: `f(i, j)` gives pixel (i, j)'s straight
/// sRGB and alpha (0..1)
fn paint_px(ctx: &mut Canvas, w: f64, h: f64, mut f: impl FnMut(usize, usize) -> [f64; 4]) {
    let (iw, ih) = (w as usize, h as usize);
    let mut img = ctx.create_image_data(w, h);
    for j in 0..ih {
        for i in 0..iw {
            let c = f(i, j);
            let k = (j * iw + i) * 4;
            for (n, v) in c.iter().enumerate() {
                img.data[k + n] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    }
    ctx.put_image_data(&img, 0, 0);
}

/// [`paint_px`] for a painter whose pixels are a pure function of where they
/// are (its draws made before it): its rows filled on every core at once
/// (`par_rows`), the same bytes sooner
fn paint_px_rows(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(usize, usize) -> [f64; 4] + Sync) {
    let iw = w as usize;
    let mut img = ctx.create_image_data(w, h);
    crate::placers::kit::par_rows(&mut img.data, iw * 4, |j, out| {
        for i in 0..iw {
            for (n, v) in f(i, j).iter().enumerate() {
                out[i * 4 + n] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

/// A transparent mark `w × h` px painted by `paint` (on its own stream);
/// where it is clear its colour is `bleed` (sRGB), so its smaller mips keep
/// the mark's colour at its soft edges instead of darkening them
fn mark_tex<F: FnOnce(&mut Canvas, f64, f64)>(w: f64, h: f64, bleed: [f64; 3], paint: F) -> Tex {
    let tex = canvas_tex_opt(w, h, false, paint);
    {
        let img = tex.image();
        let mut im = img.borrow_mut();
        for px in im.data.chunks_exact_mut(4) {
            let k = sstep(0.0, 0.3, f64::from(px[3]) / 255.0);
            for (c, b) in px.iter_mut().zip(bleed) {
                *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * k).round().clamp(0.0, 255.0) as u8;
            }
        }
        im.version += 1;
    }
    tex
}

/// The decal material of a mark painted in `tex`, lit as the surface under it
fn mark_mat(tex: &Tex) -> Mat {
    lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

/// Water on a floor of `floor` (`floor_kind`), about `rx × rz` m, on a plane
/// `2.5 rx × 2.5 rz`: soaked into carpet, a dark patch with a soft ragged
/// edge and a darker rim where the dirt wicked out; standing on a hard floor,
/// a dark film with a sharp, irregular edge, a damp margin outside it and a
/// sheen
fn puddle_mat(rx: f64, rz: f64, floor: u8) -> Mat {
    let (pw, ph) = (rx * 2.5, rz * 2.5);
    let (tw, th) = (192.0, (192.0 * rz / rx).round().clamp(96.0, 256.0));
    let hard = floor != 0;
    let wet = if hard { [0.09, 0.09, 0.095] } else { [0.12, 0.1, 0.08] };
    let tex = mark_tex(tw, th, wet, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        let (sx, sy) = (pw / cw, ph / ch);
        let aa = sx.max(sy) * 1.2;
        let r0 = rx.min(rz);
        // on a hard floor, smaller pools beside it, joined to it or not
        let sats: Vec<(f64, f64, f64)> = if hard {
            (0..(rnd() * 3.5) as usize)
                .map(|_| {
                    let (a, rr) = (rnd() * PI * 2.0, 0.65 + 0.25 * rnd());
                    (a.cos() * rx * rr, a.sin() * rz * rr, r0 * (0.1 + 0.15 * rnd()))
                })
                .collect()
        } else {
            vec![]
        };
        paint_px_rows(ctx, cw, ch, |i, j| {
            let (x, y) = ((i as f64 + 0.5) * sx - pw / 2.0, (j as f64 + 0.5) * sy - ph / 2.0);
            let (u, v) = (x / rx, y / rz);
            let rho = u.hypot(v);
            let (cu, cv) = (u / rho.max(1e-6), v / rho.max(1e-6));
            let lobe = 1.0
                + 0.5 * (fbm(seed, cu * 1.2 + 4.0, cv * 1.2 + 4.0, 3) - 0.5)
                + 0.16 * (vnoise(seed ^ 0x9, cu * 4.0 + 2.0, cv * 4.0 + 2.0) - 0.5)
                + 0.07 * (vnoise(seed ^ 0xa, cu * 10.0 + 1.0, cv * 10.0 + 1.0) - 0.5);
            let mut d = (rho - 0.88 * lobe) * r0;
            for &(qx, qz, qr) in &sats {
                d = smin(d, (x - qx).hypot(y - qz) - qr, 0.012);
            }
            let mut px = Px::CLEAR;
            if hard {
                d += (vnoise(seed ^ 0x4d, x * 90.0, y * 90.0) - 0.5) * 0.004;
                // the damp margin outside it, the film, and the dark line of its edge
                if d > 0.0 {
                    px.lay([0.11, 0.105, 0.1], 0.2 * (-d / 0.015).exp() * (0.6 + 0.4 * vnoise(seed ^ 0x5e, x * 50.0, y * 50.0)));
                }
                px.lay(wet, (0.5 - d / aa).clamp(0.0, 1.0) * (0.4 + 0.08 * vnoise(seed ^ 0x3c, x * 25.0, y * 25.0)));
                px.lay([0.035, 0.035, 0.04], tide(d, aa, 0.0015) * 0.35);
            } else {
                // soaked in: the edge ragged and soft, a darker rim at it
                let dd = d + (fbm(seed ^ 0x77, x * 14.0, y * 14.0, 3) - 0.5) * 0.035;
                px.lay(wet, sstep(0.012, -0.025, dd) * (0.55 + 0.15 * vnoise(seed ^ 0x3c, x * 30.0, y * 30.0)));
                px.lay([0.09, 0.07, 0.05], 0.22 * (-(dd + 0.004).abs() / 0.008).exp());
            }
            px.rgba()
        });
    });
    let (sheen, shine) = if hard { (0.3, 40.0) } else { (0.03, 18.0) };
    let m = phong!({ map: &tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2, shininess: shine });
    m.m().specular.set_rgb(sheen, sheen, sheen);
    m
}
