//! The Building's floors as built (`docs/MEGASTRUCTURE.md`): where
//! `level::building` planned the floors, flights, voids and masses, this
//! lays them in the world. Slabs under the floors, flights of steps,
//! a parapet on every edge you can walk (and none on what you cannot
//! reach), masses of building from the fog below to the dark above, the
//! fog's floor far down; a lamp over every cell; the door down; a few
//! pickups and notes (the dead ends pay); then her story, as on the rooms.
//!
//! The rooms' passes (purposes and their placers, settling, walls,
//! ceilings) do not run here: the Building's floors are empty.
use crate::base::records::*;
use crate::level::archgen::{Ground, DIRS, GROUND_FLOOR, GROUND_MASS, GROUND_STAIR, GROUND_VOID, SLAB};
use crate::level::building::{generate_building, BuildingPlan, Style, MASS_TOP, PLAT, VOID_DEPTH};
use crate::prelude::*;

/// A step's rise at most, metres
const STEP_RISE: f64 = 0.23;
/// A parapet's height and thickness, metres
pub const PARAPET_H: f64 = 1.05;
const PARAPET_T: f64 = 0.18;
/// A cell's lamp over its door, metres over the floor
pub const CELL_LAMP_Y: f64 = 2.55;

/// A box from `a` to `b` (its length along them, tilted with them),
/// `lift` over their line, `h` high and `t` thick
fn along(a: Vec3, b: Vec3, lift: f64, h: f64, t: f64) -> Mat4 {
    let (dx, dy, dz) = (b.x - a.x, b.y - a.y, b.z - a.z);
    let run = (dx * dx + dz * dz).sqrt().max(1e-9);
    let len = (run * run + dy * dy).sqrt();
    let q = {
        let mut q = Quat::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), (-dz).atan2(dx));
        q.multiply(&Quat::from_axis_angle(&Vec3::new(0.0, 0.0, 1.0), dy.atan2(run)));
        q
    };
    let c = Vec3::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0 + lift + h / 2.0, (a.z + b.z) / 2.0);
    Mat4::compose(&c, &q, &Vec3::new(len, h, t))
}

/// The railing of the place along an edge from `a` to `b` (on the floor)
fn rail(push: &mut impl FnMut(String, Mat4), style: Style, a: Vec3, b: Vec3, wall: &str) {
    let point = |t: f64| Vec3::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t);
    let n = |every: f64| (((b.x - a.x).hypot(b.z - a.z)) / every).round().max(1.0) as usize;
    match style {
        // the Well: a parapet poured with the gallery
        Style::Well => push(format!("{wall}:1"), along(a, b, 0.0, PARAPET_H, PARAPET_T)),
        // the Stair Halls: a stone balustrade, its plinth, balusters and rail
        Style::Halls => {
            push(format!("{wall}:1"), along(a, b, 0.0, 0.16, 0.3));
            push(format!("{wall}:1"), along(a, b, 0.93, 0.13, 0.26));
            let k = n(0.42);
            for i in 0..k {
                let p = point((i as f64 + 0.5) / k as f64);
                push(
                    format!("{wall}:1"),
                    Mat4::compose(&Vec3::new(p.x, p.y + 0.16 + 0.385, p.z), &Quat::IDENTITY, &Vec3::new(0.11, 0.77, 0.11)),
                );
            }
        }
        // the Works: posts, a mid rail, a top rail and a toe board in yellow
        Style::Works => {
            push("mat:yellow".into(), along(a, b, 1.0, 0.05, 0.05));
            push("mat:steel".into(), along(a, b, 0.5, 0.04, 0.04));
            push("mat:yellow".into(), along(a, b, 0.0, 0.12, 0.015));
            let k = n(1.35);
            for i in 0..=k {
                let p = point(i as f64 / k as f64);
                push("mat:steel".into(), Mat4::compose(&Vec3::new(p.x, p.y + 0.52, p.z), &Quat::IDENTITY, &Vec3::new(0.05, 1.05, 0.05)));
            }
        }
    }
}

impl Game {
    /// Lays out the Building's floor for level `n` (inside the level's seed):
    /// the tiles, the ground, the plan, the solids.
    pub fn gen_building_level(&mut self, n: i32) {
        let (layout, plan) = generate_building(n, &mut || rnd());
        self.w = layout.w;
        self.h = layout.h;
        self.tiles = layout.tiles.clone();
        self.ground = layout.ground.clone();
        // no ceiling over a walk: the dark above it
        self.tile_ceil = layout.ground.iter().map(|g| if g.walkable() { g.h0.max(g.h1) + 60.0 } else { MASS_TOP }).collect();
        self.layout = Some(layout);
        self.has_layout = true;
        self.building = Some(plan);
        self.solids = self.build_solids_now(None);
    }

    fn ground_at(&self, x: i32, z: i32) -> Ground {
        if !self.in_grid(x, z) {
            return Ground::mass(VOID_DEPTH, MASS_TOP);
        }
        self.ground[self.t_idx(x, z)]
    }

    /// `build_world` on a floor of the Building (after the regions)
    pub(crate) fn build_building(&mut self, bt0: f64) {
        let plan = self.building.clone().expect("a floor of the Building");
        // the Building is lit by its own lamps and windows: one dim light everywhere
        for info in self.region_info.iter_mut() {
            info.flags = 0;
            info.lit = Some(0.5);
        }
        self.spawn_pos = Vec3::new(self.wx(plan.spawn.0), self.floor_y_of(plan.spawn.0, plan.spawn.1) + EYE, self.wz(plan.spawn.1));
        self.building_architecture();
        self.build_lap("floors+panels", bt0);
        self.building_lamps(&plan);
        self.cell_doors.clear();
        self.dress_building(&plan);
        self.building_fog_layers();
        let prop_start = self.world.child_count();
        let core = !self.test_stub_core_placers;
        if core {
            self.building_props(&plan);
        }
        self.tag_placed(prop_start, "place_props");
        self.build_lap("core_props", bt0);
        if core {
            let c0 = self.world.child_count();
            self.place_story();
            self.tag_placed(c0, "place_story");
        }
        self.build_lap("placement", bt0);
        if !self.test_keep_prop_looks {
            self.settle_prop_looks(prop_start);
        }
        self.freeze_static();
        if !self.has_flag("noinstance") {
            let (w, h) = (self.w as f64, self.h as f64);
            let chunk_of = |x: f64, z: f64| {
                ((x / CELL + w / 2.0).floor() / 10.0).floor() as i64 * 64 + ((z / CELL + h / 2.0).floor() / 10.0).floor() as i64
            };
            crate::world::instancing::instancify_static(&self.world, &chunk_of);
        }
        self.build_prop_cull();
        self.build_lap("architecture", bt0);
        let total = crate::base::numeric::round_decimals(self.clock_ms() - bt0, 1);
        self.build_profile.insert("t.total".into(), total);
    }

    /// The slabs, flights, parapets and masses, and the fog's floor.
    fn building_architecture(&mut self) {
        let (w, h) = (self.w, self.h);
        let style = self.building.as_ref().map(|p| p.style).unwrap_or(Style::Well);
        let zone_mats = |g: &Game, x: i32, z: i32| -> (&'static str, &'static str) {
            g.zone_ref(x, z).map(|zn| (zn.wall_mat, zn.floor_mat)).unwrap_or(("concrete", "concrete"))
        };
        let mut slabs: Vec<(String, Vec<Mat4>)> = vec![];
        let mut push = |key: String, m: Mat4| match slabs.iter_mut().find(|(k, _)| *k == key) {
            Some((_, v)) => v.push(m),
            None => slabs.push((key, vec![m])),
        };
        for z in 0..h {
            for x in 0..w {
                let g = self.ground_at(x, z);
                let (wall, floor) = zone_mats(self, x, z);
                let (cx, cz) = (self.wx(x), self.wz(z));
                match g.kind {
                    GROUND_FLOOR => {
                        let top = g.h0 as f64;
                        push(
                            format!("floor:{floor}:1"),
                            Mat4::compose(&Vec3::new(cx, top - SLAB / 2.0, cz), &Quat::IDENTITY, &Vec3::new(CELL, SLAB, CELL)),
                        );
                    }
                    GROUND_STAIR => {
                        // steps from the low end up; each a block under its tread
                        let (h0, h1) = (g.h0 as f64, g.h1 as f64);
                        let rise = (h1 - h0).abs();
                        let n = (rise / STEP_RISE).round().max(1.0) as usize;
                        let d = g.dir as usize;
                        let (fx, fz) = (DIRS[d][0] as f64, DIRS[d][1] as f64);
                        for k in 0..n {
                            // along the run, from the tile's back edge (s = 0) to its front (s = 1)
                            let (s0, s1) = (k as f64 / n as f64, (k + 1) as f64 / n as f64);
                            let top = if h1 > h0 { h0 + rise * s1 } else { h0 - rise * s0 };
                            let bottom = top - rise / n as f64 - SLAB;
                            let sm = (s0 + s1) / 2.0 - 0.5;
                            let len = CELL / n as f64;
                            let (sx, sz) = if fx != 0.0 { (len, CELL) } else { (CELL, len) };
                            push(
                                format!("floor:{floor}:1"),
                                Mat4::compose(
                                    &Vec3::new(cx + fx * sm * CELL, (top + bottom) / 2.0, cz + fz * sm * CELL),
                                    &Quat::IDENTITY,
                                    &Vec3::new(sx, top - bottom, sz),
                                ),
                            );
                        }
                    }
                    _ => {}
                }
                // a railing on each edge of a walk that looks over the void
                // (the place's own: a poured parapet, a stone balustrade, a
                // steel rail), and nowhere else: what has none cannot be walked
                if g.walkable() {
                    for dd in DIRS.iter() {
                        if self.ground_at(x + dd[0], z + dd[1]).kind != GROUND_VOID {
                            continue;
                        }
                        let (fx, fz) = (dd[0] as f64, dd[1] as f64);
                        // the edge, from one end to the other, at the floor
                        let (ox, oz) = (cx + fx * (CELL / 2.0 - PARAPET_T / 2.0), cz + fz * (CELL / 2.0 - PARAPET_T / 2.0));
                        let (ax, az) = (-fz * CELL / 2.0, fx * CELL / 2.0);
                        let fa = g.at(x, z, self.to_u(ox - ax), self.to_v(oz - az));
                        let fb = g.at(x, z, self.to_u(ox + ax), self.to_v(oz + az));
                        let a = Vec3::new(ox - ax, fa, oz - az);
                        let b = Vec3::new(ox + ax, fb, oz + az);
                        let poured = if wall == "panel" { "concrete" } else { wall };
                        rail(&mut push, style, a, b, poured);
                    }
                }
            }
        }
        // the masses: one box per run of building along a row
        for z in 0..h {
            let mut x = 0;
            while x < w {
                if self.ground_at(x, z).kind != GROUND_MASS {
                    x += 1;
                    continue;
                }
                let x0 = x;
                let (wall, _) = zone_mats(self, x, z);
                while x < w && self.ground_at(x, z).kind == GROUND_MASS && zone_mats(self, x, z).0 == wall {
                    x += 1;
                }
                let (a, b) = (self.wx(x0) - CELL / 2.0, self.wx(x - 1) + CELL / 2.0);
                let (y0, y1) = (VOID_DEPTH as f64, MASS_TOP as f64);
                push(
                    format!("{wall}:1"),
                    Mat4::compose(
                        &Vec3::new((a + b) / 2.0, (y0 + y1) / 2.0, self.wz(z)),
                        &Quat::IDENTITY,
                        &Vec3::new(b - a, y1 - y0, CELL),
                    ),
                );
            }
        }
        let geo = self.unit_box_geo.clone();
        for (key, list) in slabs {
            let mat = match key.as_str() {
                "mat:steel" => lambert!({ color: 0x484b49 }),
                "mat:yellow" => lambert!({ color: 0x8c7a34 }),
                _ => self.arch_mat(&key),
            };
            self.add_instanced(&geo, &mat, &list, &|m: &Mat4| *m);
        }
        // the fog's floor, far down
        let bottom = mesh(plane_geo!(w as f64 * CELL * 3.0, h as f64 * CELL * 3.0), lambert!({ color: 0x2b2c29 }));
        bottom.rotation().x = -PI / 2.0;
        bottom.position().y = VOID_DEPTH as f64 + 0.5;
        self.world.add(&bottom);
    }

    /// The lamps, as panels the pooled lights and the light map know: one
    /// over every cell's door; one where three or more ways meet; one at the
    /// foot of every flight (what can be walked is lit where it starts). The
    /// Well's are bulkheads on the walls, the Stair Halls' iron lanterns, the
    /// Works' floodlights on posts.
    fn building_lamps(&mut self, plan: &BuildingPlan) {
        let mut spots: Vec<(f64, f64, f64, bool)> = vec![];
        for c in &plan.cells {
            let (dx, dz) = (DIRS[c.door][0] as f64, DIRS[c.door][1] as f64);
            let (x, z) = (self.wx(c.tile.0) + dx * (CELL / 2.0 + 0.15), self.wz(c.tile.1) + dz * (CELL / 2.0 + 0.15));
            spots.push((x, z, self.floor_y_of(c.tile.0, c.tile.1) + CELL_LAMP_Y, false));
        }
        for nd in &plan.nodes {
            if nd.degree >= 3 {
                let (x, z) = (self.wx(nd.x) + CELL / 2.0, self.wz(nd.z) + CELL / 2.0);
                spots.push((x, z, nd.storey as f64 * crate::level::building::STOREY + 3.4, true));
            }
        }
        // the foot of every flight: the floor tile its low end meets
        for z in 0..self.h {
            for x in 0..self.w {
                let g = self.ground_at(x, z);
                if g.kind != GROUND_STAIR {
                    continue;
                }
                // the low end's side
                let d = if g.h1 < g.h0 { g.dir as usize } else { g.dir as usize ^ 1 };
                let (fx, fz) = (x + DIRS[d][0], z + DIRS[d][1]);
                let foot = self.ground_at(fx, fz);
                if foot.kind != GROUND_FLOOR {
                    continue;
                }
                // to one side of the way, so it lights the first steps
                let (sx, sz) = (-(DIRS[d][1] as f64), DIRS[d][0] as f64);
                let (px, pz) = (self.wx(fx) + sx * (CELL / 2.0 - 0.35), self.wz(fz) + sz * (CELL / 2.0 - 0.35));
                spots.push((px, pz, foot.h0 as f64 + 2.6, true));
            }
        }
        let style = plan.style;
        let geo = match style {
            Style::Well => box_geo!(0.22, 0.14, 0.22),
            Style::Halls => box_geo!(0.2, 0.34, 0.2),
            Style::Works => box_geo!(0.5, 0.16, 0.32),
        };
        let pm = instanced_mesh(&geo, basic_mat!({ color: if style == Style::Halls { 0xf3d6a0 } else { 0xfff1d0 } }), spots.len());
        let white = Color::white();
        let mut posts: Vec<Mat4> = vec![];
        for (i, &(x, z, y, post)) in spots.iter().enumerate() {
            let mut m4 = Mat4::IDENTITY;
            m4.make_translation(x, y, z);
            pm.set_matrix_at(i, &m4);
            pm.set_color_at(i, &white);
            // a post under it, where it does not hang from a wall
            if post && style != Style::Well {
                let f = self.floor_y_at(x, z);
                posts.push(Mat4::compose(&Vec3::new(x, (f + y) / 2.0, z), &Quat::IDENTITY, &Vec3::new(0.07, (y - f).max(0.1), 0.07)));
            }
            let seed = rand1(100.0);
            let lfo_freq = 0.15 + rnd() * 0.25;
            let lfo_phase = rnd() * PI * 2.0;
            self.panels.push(Panel {
                wx: x,
                wz: z,
                y,
                bright: 1.0,
                seed,
                rot: 0.4,
                lfo_freq,
                lfo_phase,
                h_boost: 1.0,
                ..Default::default()
            });
        }
        self.world.add(&pm);
        self.panel_mesh = Some(pm);
        if !posts.is_empty() {
            let geo = self.unit_box_geo.clone();
            let mat = lambert!({ color: 0x3a3936 });
            self.add_instanced(&geo, &mat, &posts, &|m: &Mat4| *m);
        }
    }

    /// The self-assembly's fog, as a surface a few layers deep over the
    /// whole floor (hidden until it comes: `world::assembly`)
    fn building_fog_layers(&mut self) {
        self.fog_layers.clear();
        // (its noise from a stream of its own: a draw from the native stream
        // at a floor's start would move every frame after it)
        let tex = crate::placers::kit::canvas_tex(256, 256, |ctx, w, h| {
            let mut r = SplitMix64::new(0xF06_5EED, 0x5A_3B1E);
            ctx.set_fill_style("#6e5a7a");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..40 {
                let (x, y, rr) = (r.next() * w, r.next() * h, 20.0 + r.next() * 70.0);
                let c = if r.next() < 0.5 { "rgba(190,160,200,0.25)" } else { "rgba(30,20,36,0.3)" };
                crate::visuals::surfaces::blotch(ctx, w, h, x, y, rr, c);
            }
        });
        {
            let mut t = tex.t();
            t.repeat = Vec2::new(6.0, 6.0);
        }
        let size = self.w.max(self.h) as f64 * CELL * 1.6;
        for (k, op) in [0.7, 0.45, 0.3].iter().enumerate() {
            let m = mesh(
                plane_geo!(size, size),
                basic_mat!({ map: &tex, color: 0xb8a4c4, transparent: true, opacity: *op, depthWrite: false, side: DoubleSide }),
            );
            m.rotation().x = -PI / 2.0;
            m.rotation().z = k as f64 * 0.7;
            m.set_visible(false);
            m.user().gameplay = true;
            self.world.add(&m);
            self.fog_layers.push(m);
        }
    }

    /// The door down, and what the floor leaves lying about: water and a
    /// battery at the ends of walks that go nowhere and in cells, a torn
    /// note or two.
    fn building_props(&mut self, plan: &BuildingPlan) {
        // the door, set in its wall
        let (ex, ez) = plan.exit;
        let (fx, fz) = (DIRS[plan.exit_side][0] as f64, DIRS[plan.exit_side][1] as f64);
        let spot = WallSpot {
            pos: Vec3::new(self.wx(ex) + fx * (CELL / 2.0 - 0.06), self.floor_y_of(ex, ez), self.wz(ez) + fz * (CELL / 2.0 - 0.06)),
            normal: Vec3::new(-fx, 0.0, -fz),
            yaw: (-fx).atan2(-fz),
        };
        self.build_exit_door(&spot);
        // where things lie: the dead ends first, then the cells
        let mut spots: Vec<(i32, i32)> = vec![];
        for nd in &plan.nodes {
            if nd.degree == 1 && (nd.x, nd.z) != (1, 1) {
                spots.push((nd.x + irand(0, (PLAT - 1) as i64) as i32, nd.z + irand(0, (PLAT - 1) as i64) as i32));
            }
        }
        let mut cells: Vec<(i32, i32)> = plan.cells.iter().map(|c| c.tile).collect();
        let mut k = cells.len();
        while k > 1 {
            k -= 1;
            let j = irand(0, k as i64) as usize;
            cells.swap(k, j);
        }
        spots.extend(cells);
        spots.retain(|&t| t != plan.exit && t != plan.spawn);
        let bottle_geo = pg("Props__almond_bottle", cyl_geo!(0.09, 0.09, 0.3, 10));
        let bottle_mat = lambert!({ color: 0x9fc4d8, emissive: 0x16323e });
        let cap_geo = pg("Props__almond_cap", cyl_geo!(0.05, 0.05, 0.06, 8));
        let bat_body_geo = pg("Props__battery_body", cyl_geo!(0.06, 0.06, 0.18, 10));
        let bat_tip_geo = pg("Props__battery_tip", cyl_geo!(0.025, 0.025, 0.03, 8));
        let bat_body_mat = lambert!({ color: 0xc79b3d, emissive: 0x2a1f08 });
        let bat_tip_mat = lambert!({ color: 0xdad3c2 });
        let (waters, batteries) = (irand(2, 3) as usize, irand(1, 2) as usize);
        let mut used = 0usize;
        for i in 0..waters + batteries {
            let Some(&(tx, tz)) = spots.get(used) else { break };
            used += 1;
            let (px, pz) = (self.wx(tx) + rand2(-0.8, 0.8), self.wz(tz) + rand2(-0.8, 0.8));
            let water = i < waters;
            let base = self.floor_y_at(px, pz) + if water { 0.15 } else { 0.11 };
            let gp = group();
            gp.position().set(px, base, pz);
            if water {
                gp.add(&mesh(&bottle_geo, &bottle_mat));
                let cap = mesh(&cap_geo, &self.mats.metal);
                cap.position().y = 0.18;
                gp.add(&cap);
            } else {
                gp.add(&mesh(&bat_body_geo, &bat_body_mat));
                let tip = mesh(&bat_tip_geo, &bat_tip_mat);
                tip.position().y = 0.105;
                gp.add(&tip);
            }
            self.world.add(&gp);
            let p = Pickup { g: gp, taken: false, base, phase: rand1(7.0) };
            if water {
                self.almonds.push(p);
            } else {
                self.battery_items.push(p);
            }
        }
        // a torn note or two, from the pool (not an anomaly's)
        let n = self.lore_entry_count();
        let mut pool: Vec<usize> = (0..n).filter(|&i| self.lore_entry_scp(i).is_none()).collect();
        let mut i = pool.len();
        while i > 1 {
            i -= 1;
            let j = irand(0, i as i64) as usize;
            pool.swap(i, j);
        }
        let notes = irand(1, 2) as usize;
        for &id in pool.iter().take(notes) {
            let Some(&(tx, tz)) = spots.get(used) else { break };
            used += 1;
            let m = self.build_lore_note_mesh(false, None);
            let (px, pz) = (self.wx(tx) + rand2(-0.6, 0.6), self.wz(tz) + rand2(-0.6, 0.6));
            m.position().set(px, self.floor_y_at(px, pz), pz);
            m.rotation().y = rand1(PI * 2.0);
            self.world.add(&m);
            self.lore_notes.push(LoreNote { g: m, taken: false, id, clue: None });
        }
    }
}
