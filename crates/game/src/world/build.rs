//! Prop themes and `build_world`.
//!
//! Every placer is sorted into a theme by name, and every zone has a purpose
//! (`world::purpose`) naming the themes it welcomes. `themed(name, dist)` runs
//! one placer with the tiles of the other zones hidden from it (value 2:
//! neither open floor nor wall). The chance it runs at all scales with how
//! much floor it may use.
use crate::base::records::*;
use crate::prelude::*;
use crate::visuals::lighting::{RF_DARK, RF_MOSS};
use crate::world::tables::{PLACER_ALWAYS, PLACER_THEME, THEMED_ORDER, THEME_ALLOW, THEME_SHARE_GAIN};
use crate::world::{PoolPlan, GRIME_LIGHT};
use std::collections::{HashMap, HashSet};

thread_local! { static MISSING: std::cell::RefCell<HashSet<String>> = std::cell::RefCell::new(HashSet::new()); }
fn log_missing(name: &str) {
    MISSING.with(|m| {
        if m.borrow_mut().insert(name.to_string()) {
            eprintln!("[world] placer {name} is not registered; skipped");
        }
    });
}

/// The theme of placer `name`, uncached: looked up in `PLACER_THEME`
/// (unlisted names are "domestic")
pub fn placer_theme_of(name: &str) -> &'static str {
    match PLACER_THEME.binary_search_by(|(n, _)| (*n).cmp(name)) {
        Ok(i) => PLACER_THEME[i].1,
        Err(_) => "domestic",
    }
}
/// How welcome `theme` is in archetype `arch` (0 when unlisted)
pub fn theme_allow(arch: &str, theme: &str) -> f64 {
    THEME_ALLOW.iter().find(|(a, _)| *a == arch).and_then(|(_, l)| l.iter().find(|(t, _)| *t == theme).map(|&(_, v)| v)).unwrap_or(0.0)
}

impl Game {
    /// (Re)builds the name → placer index of `g.placers`
    pub fn placer_index_refresh(&mut self) {
        if self.placer_index.0 == self.placers.len() && !self.placer_index.1.is_empty() {
            return;
        }
        let mut m: HashMap<&'static str, PlacerFn> = HashMap::new();
        for &(n, f) in &self.placers {
            m.entry(n).or_insert(f);
        }
        self.placer_index = (self.placers.len(), m);
    }
    /// A registered placer by name
    pub fn placer_fn(&mut self, name: &str) -> Option<PlacerFn> {
        self.placer_index_refresh();
        self.placer_index.1.get(name).copied()
    }
    /// The theme of placer `name` (cached)
    pub fn placer_theme(&mut self, name: &str) -> &'static str {
        if let Some(t) = self.placer_theme_cache.get(name) {
            return t;
        }
        let t = placer_theme_of(name);
        self.placer_theme_cache.insert(name.to_string(), t);
        t
    }
    /// What a pass added carries its placer's name
    pub fn tag_placed(&mut self, c0: usize, name: &str) {
        for k in c0..self.world.child_count() {
            let c = self.world.child(k).unwrap();
            let mut u = c.user();
            if u.placer.as_deref().map(|s| s.is_empty()).unwrap_or(true) {
                u.placer = Some(name.to_string());
            }
        }
    }
    /// What a core prop step added carries its kind (`kind` in its strings;
    /// read by the prop census, never by the build)
    pub fn tag_kind(&mut self, c0: usize, kind: &str) {
        for k in c0..self.world.child_count() {
            let c = self.world.child(k).unwrap();
            c.user().s.entry("kind".into()).or_insert_with(|| kind.to_string());
        }
    }
    fn run_placer(&mut self, name: &str, f: Option<PlacerFn>, dist: &[i32]) -> bool {
        let Some(f) = f else {
            log_missing(name);
            return true;
        };
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(self, dist)));
        if r.is_err() {
            eprintln!("[world] placer {name} panicked; the level goes on without it");
            return false;
        }
        true
    }
    /// Runs one placer by name, with the zones whose purpose does not
    /// welcome its theme hidden from it
    pub fn themed(&mut self, name: &str, dist: &[i32]) {
        let f = self.placer_fn(name);
        if self.placer_dist.as_deref() != Some(dist) {
            self.placer_dist = Some(dist.to_vec());
        }
        self.trace_mark(name);
        if self.layout.is_none() || PLACER_ALWAYS.contains(&name) {
            let c = self.world.child_count();
            self.run_placer(name, f, dist);
            self.tag_placed(c, name);
            self.theme_stats_ran += 1;
            return;
        }
        let theme = self.placer_theme(name);
        let allow: Vec<bool> = {
            let zones = &self.layout.as_ref().unwrap().zones;
            if self.zone_purposes.len() == zones.len() {
                self.zone_purposes.iter().map(|p| p.allows(theme)).collect()
            } else {
                // (no purposes picked: wherever the archetype welcomes the theme at all)
                zones.iter().map(|z| theme_allow(z.arch, theme) > 0.0).collect()
            }
        };
        let (mut open, mut ok) = (0usize, 0usize);
        let zo = self.layout.as_ref().unwrap().zone_of.clone();
        let n = (self.w * self.h) as usize;
        for i in 0..n {
            if self.tiles[i] == 0 {
                open += 1;
                if allow[zo[i] as usize] {
                    ok += 1;
                }
            }
        }
        let bt = self.theme_stats_by_theme.entry(theme.to_string()).or_insert((0, 0));
        if ok == 0 || rnd() > ok as f64 / open.max(1) as f64 * THEME_SHARE_GAIN {
            self.theme_stats_skipped += 1;
            bt.1 += 1;
            return;
        }
        let mut masked: Vec<usize> = vec![];
        for i in 0..n {
            if self.tiles[i] == 0 && !allow[zo[i] as usize] {
                self.tiles[i] = 2;
                masked.push(i);
            }
        }
        let (a0, i0, c0) = (self.animators.len(), self.interactables.len(), self.world.child_count());
        self.run_placer(name, f, dist);
        for &i in &masked {
            self.tiles[i] = 0;
        }
        self.tag_placed(c0, name);
        // a placer that registered no animator and no interaction made scenery
        // that never moves: frozen after `settle_props` (see `freeze_static`)
        if self.animators.len() == a0 && self.interactables.len() == i0 {
            for k in c0..self.world.child_count() {
                self.world.child(k).unwrap().user().static_prop = true;
            }
        }
        self.theme_stats_ran += 1;
        self.theme_stats_by_theme.get_mut(theme).unwrap().0 += 1;
    }

    /// Ends build phase `k`: records its time and a trace mark
    pub(crate) fn build_lap(&mut self, k: &str, bt0: f64) {
        let n = self.clock_ms();
        let last = self.build_profile.get("last").copied().filter(|&v| v != 0.0).unwrap_or(bt0);
        self.build_profile.insert(format!("t.{k}"), crate::base::numeric::round_decimals(n - last, 1));
        self.build_profile.insert("last".into(), n);
        self.trace_mark(&format!("lap:{k}"));
    }
}

impl Game {
    pub fn build_world(&mut self) {
        if self.world.parent().is_some() {
            // (instanced meshes need no explicit dispose: GPU buffers go with the handles)
            self.scene.remove(&self.world.clone());
            let mut res = crate::world::arch::Resources::default();
            crate::world::arch::collect_resources(&self.world, &mut res);
            self.pending_dispose = Some(res); // freed once the new level is up
        }
        self.world = group();
        self.pp.reset();
        crate::assets::props::clear_prop_cache();
        self.theme_stats_ran = 0;
        self.theme_stats_skipped = 0;
        self.theme_stats_by_theme.clear();
        self.panels.clear();
        self.interactables.clear();
        self.animators.clear();
        self.almonds.clear();
        self.battery_items.clear();
        self.floor_holes.clear();
        self.barricade_tiles.clear();
        self.barricades.clear();
        self.pool_bounds = None;
        self.papers.clear();
        self.paper_markers.clear();
        self.lore_notes.clear();
        self.story = Default::default();
        self.safe_room = None;
        self.broken_elevator = None;
        self.light_switches.clear();
        self.faulty_trip_count = 0.0;
        self.escalation_toast_fired = false;
        self.haunted_candles.clear();
        // (the last level's contact shadows)
        self.prop_cull.shadows.clear();
        // Per-weather ambient timers, rare-event timers and envelopes
        crate::world::tables::reset_level_timers(self);
        self.ambient_log.clear();
        self.rain_windows.clear();
        self.drawer_props.clear();
        for m in &self.vending {
            if let Some(h) = &m.hum {
                h.stop();
            }
        }
        self.vending.clear();
        for w in &self.windows {
            if let Some(h) = &w.hum {
                h.stop();
            }
        }
        self.windows.clear();
        self.behind_glass = None;
        self.window_plans.clear();
        self.pp.vending_positions.clear();
        // a CRT's hiss is the floor's own (`place_old_crt_tvs` fills it when it
        // places one, `usable_settled` adds the lit set)
        self.pp.crt_positions.clear();
        self.vacuum_bot = None;
        // the Building's per-floor things (a floor of the rooms has none)
        self.cell_doors.clear();
        self.fog_layers.clear();
        self.building_speakers.clear();
        self.far_figure = None;
        self.figure_perches.clear();
        self.lift_at = None;
        self.knocks = Default::default();
        self.assembly = Default::default();
        self.tv_data = None;
        self.phone_data = None;
        self.clock_data = None;
        self.exit_data = None;
        self.man_data = None;

        let bt0 = self.clock_ms();
        self.build_profile.retain(|k, _| !k.starts_with("t.")); // clear the phase times
        self.build_profile.insert("last".into(), bt0);
        // what each zone is for, then the regions (areas of zones, room by room) with their own variants
        self.pick_zone_purposes();
        self.build_regions();
        // the Showroom is spotless and lit end to end until it closes
        if self.level_scp.as_deref() == Some("store") {
            for info in self.region_info.iter_mut() {
                info.flags &= !(RF_DARK | RF_MOSS);
                info.grime = 0;
                info.lit = Some(GRIME_LIGHT[0]);
                info.ceil = 0;
                info.floor = 0;
            }
        }
        // the Borrowed House needs dark to hide in: about half its rooms are dead
        if self.level_scp.as_deref() == Some("house") {
            let spawn = self.region_map.get(self.t_idx(1, 1)).copied().unwrap_or(0) as usize;
            for (i, info) in self.region_info.iter_mut().enumerate() {
                if i != spawn && rnd() < 0.5 {
                    info.flags |= RF_DARK;
                }
            }
        }
        self.build_lap("regions", bt0);
        // a floor of the Building is built by its own passes (`world::building`)
        if self.building.is_some() {
            self.build_building(bt0);
            if let Some(l) = self.layout.as_mut() {
                if l.tiles.len() == self.tiles.len() {
                    l.tiles.copy_from_slice(&self.tiles);
                }
            }
            self.scene.add(&self.world.clone());
            return;
        }

        // ---- pre-scan for the pool so the floor loop can skip its tiles ----
        let mut pool_reserved: HashSet<usize> = HashSet::new();
        'outer: for (pw, ph) in [(3, 3), (4, 3), (3, 4), (4, 4), (3, 2), (2, 3)] {
            let mut z = 2;
            while z <= self.h - 2 - ph {
                let mut x = 2;
                while x <= self.w - 2 - pw {
                    let mut all_open = true;
                    'scan: for zz in 0..ph {
                        for xx in 0..pw {
                            if self.is_wall_tile(x + xx, z + zz) {
                                all_open = false;
                                break 'scan;
                            }
                        }
                    }
                    if all_open {
                        let cxt = self.wx(x as f64 + pw as f64 / 2.0 - 0.5);
                        let czt = self.wz(z as f64 + ph as f64 / 2.0 - 0.5);
                        if crate::base::numeric::hypot2(cxt - self.wx(1), czt - self.wz(1)) >= 8.0 {
                            for zz in 0..ph {
                                for xx in 0..pw {
                                    pool_reserved.insert(self.t_idx(x + xx, z + zz));
                                }
                            }
                            self.pool_plan = Some(PoolPlan { x, z, pw, ph });
                            break 'outer;
                        }
                    }
                    x += 1;
                }
                z += 1;
            }
        }

        // ---- floor: under open tiles and under the walkable part of wall tiles ----
        self.build_floors(&pool_reserved);
        // outer skirt so the edge doesn't reveal void
        let skirt = mesh(plane_geo!(self.w as f64 * CELL * 2.0, self.h as f64 * CELL * 2.0), lambert!({ color: 0x1a140a }));
        skirt.rotation().x = -PI / 2.0;
        skirt.position().y = -0.02;
        self.world.add(&skirt);
        // walls and ceilings are built after the props (see `settle_props`)

        let dist = self.bfs(1, 1);
        self.spawn_pos = Vec3::new(self.wx(1), EYE, self.wz(1));

        // ceiling light panels over open tiles in a loose grid; dark regions get none
        let mut panel_spots: Vec<(i32, i32)> = vec![];
        for z in 1..self.h - 1 {
            for x in 1..self.w - 1 {
                if self.tiles[self.t_idx(x, z)] != 0 {
                    continue;
                }
                let rid = self.region_map[self.t_idx(x, z)] as usize;
                if self.region_info[rid].flags & RF_DARK != 0 {
                    continue;
                } // dark zones stay dark
                  // an intact room has tubes over every other tile; a rotting one thins out
                let dense = self.region_info[rid].lit.unwrap_or(0.0) >= 0.75;
                let le = self.zone_ref(x, z).map(|zn| zn.light_every).unwrap_or(0);
                let k = (if le != 0 { le } else { 2 }) * if dense { 1 } else { 2 };
                // the fittings are laid out per room (per zone where it has no
                // rooms) and centred in it: they line up with the room, not
                // with wherever a level-wide grid happens to fall
                let (ax0, az0, ax1, az1) = self
                    .layout
                    .as_ref()
                    .and_then(|l| l.room_at(x, z).map(|r| (r.x0, r.z0, r.x1, r.z1)))
                    .or_else(|| self.zone_ref(x, z).map(|zn| (zn.x0, zn.z0, zn.x1, zn.z1)))
                    .unwrap_or((0, 0, self.w - 1, self.h - 1));
                let (ox, oz) = (ax0 + (ax1 - ax0) % k / 2, az0 + (az1 - az0) % k / 2);
                if (x - ox).rem_euclid(k) != 0 || (z - oz).rem_euclid(k) != 0 {
                    continue;
                }
                panel_spots.push((x, z));
            }
        }
        let pm = instanced_mesh(&self.panel_geo, basic_mat!({ map: &self.tex.light_tex }), panel_spots.len());
        let c_white = Color::white();
        for (i, &(x, z)) in panel_spots.iter().enumerate() {
            let mut m4 = Mat4::IDENTITY;
            m4.make_translation(self.wx(x), self.tile_ceil[self.t_idx(x, z)] as f64 - 0.04, self.wz(z));
            pm.set_matrix_at(i, &m4);
            pm.set_color_at(i, &c_white);
            let lit = self.region_info[self.region_map[self.t_idx(x, z)] as usize].lit;
            let seed = rand1(100.0);
            let rot = 1.0 - lit.unwrap_or(0.5);
            let lfo_freq = 0.15 + rnd() * 0.25;
            let lfo_phase = rnd() * PI * 2.0;
            self.panels.push(Panel {
                wx: self.wx(x),
                wz: self.wz(z),
                bright: 1.0,
                flicker: 0.0,
                seed,
                rot,
                lfo_freq,
                lfo_phase,
                ..Default::default()
            });
        }
        self.world.add(&pm);
        self.panel_mesh = Some(pm);
        // 0-2 broken ceiling panels — a dead spotlight and a hanging tile piece
        let broken_count = if rnd() < 0.6 {
            if rnd() < 0.4 {
                2
            } else {
                1
            }
        } else {
            0
        };
        if broken_count > 0 && self.panels.len() > 3 {
            let mut index_pool: Vec<usize> = (0..self.panels.len()).collect();
            let mut i = index_pool.len();
            while i > 1 {
                i -= 1;
                let j = (rnd() * (i + 1) as f64).floor() as usize;
                index_pool.swap(i, j);
            }
            // one of the ceiling's own tiles, grimy (near black, seen from
            // below against a lit panel, it had read as a hole in the ceiling)
            let broken_mat = lambert!({ color: 0x6d675a });
            let cell = CELL / crate::visuals::textures::CEIL_CELLS as f64;
            for k in 0..broken_count {
                let pi = index_pool[k];
                self.panels[pi].bright = 0.05; // near-dead spotlight
                let tile = mesh(box_geo!(cell, 0.02, cell), &broken_mat);
                let tilt_a = (PI / 6.0) + rnd() * (PI / 6.0); // 30-60°
                let tilt_x = rnd() < 0.5;
                let v = tilt_a * if rnd() < 0.5 { 1.0 } else { -1.0 };
                if tilt_x {
                    tile.rotation().x = v;
                } else {
                    tile.rotation().z = v;
                }
                tile.rotation().y = rnd() * PI * 2.0;
                let offset = 0.35 + rnd() * 0.20;
                let (pwx, pwz) = (self.panels[pi].wx, self.panels[pi].wz);
                tile.position().set(pwx, self.ceil_at_world(pwx, pwz) - offset, pwz);
                self.world.add(&tile);
            }
        }
        // (no light cones under the panels: a fixture shows as a bright panel)

        self.build_lap("floors+panels", bt0);
        let prop_start = self.world.child_count();
        let core = !self.test_stub_core_placers;
        if core {
            self.place_props(&dist);
        }
        self.tag_placed(prop_start, "place_props");
        if !core {
        } else if self.level_scp.as_deref() == Some("store") {
            let c0 = self.world.child_count();
            self.place_store(&dist);
            self.tag_placed(c0, "place_store");
            for k in c0..self.world.child_count() {
                self.world.child(k).unwrap().user().static_prop = true;
            }
        } else if self.level_scp.as_deref() == Some("house") {
            let c0 = self.world.child_count();
            self.place_house(&dist);
            self.tag_placed(c0, "place_house");
        }
        self.build_lap("core_props", bt0);
        for name in THEMED_ORDER {
            self.themed(name, &dist);
        }
        self.build_lap("themed_props", bt0);
        // reconcile the props with the real walls, then build the walls themselves
        self.settle_props(prop_start);
        self.build_lap("settle", bt0);
        // the barricades, in the real openings of the final walls, on a
        // stream of their own (`core_props::barricade`): before her trail,
        // which goes round them, and before placement, which clears them
        if core {
            self.barricades_settled();
        }
        // her camp and her trail (`story`): once the walls are final (settling
        // rebuilds them), before placement so the props round them make room;
        // on a stream of its own, so no checkpoint moves
        if core {
            let c0 = self.world.child_count();
            self.place_story();
            self.tag_placed(c0, "place_story");
            // and what is put away off her way (`secrets`), on a stream of its own
            let c0 = self.world.child_count();
            self.place_secrets();
            self.tag_placed(c0, "place_secrets");
        }
        self.settle_placement(prop_start);
        self.story_marks_settled();
        self.free_buried_pickups();
        self.vending_settled();
        // the windows, on the final walls (their openings are cut as the walls are drawn)
        self.windows_settled();
        // the verbs of what a hand can use, now that nothing will move
        // (`core_props::usable`; an interactable pins the walls near it)
        if core {
            self.usable_settled();
        }
        self.build_lap("placement", bt0);
        // each prop measured once, for its look and its contact shadow
        let props = self.prop_boxes(prop_start);
        if !self.test_keep_prop_looks {
            self.settle_prop_looks(&props);
        }
        // the small things' contact shadows, from where they now lie (laid
        // after the instancing, which takes the repeated ones out)
        let contact = self.contact_blobs(&props);
        drop(props);
        self.build_walls();
        self.build_ceilings();
        self.lay_panels();
        self.freeze_static();
        // repeated static parts become one instanced mesh per 10×10-tile chunk; #noinstance keeps them apart
        if !self.has_flag("noinstance") {
            let st = crate::world::instancing::instancify_static(&self.world, &|x, z| self.chunk_of(x, z));
            for (k, v) in
                [("meshes", st.meshes), ("instanced", st.instanced), ("buckets", st.buckets), ("removedGroups", st.removed_groups)]
            {
                self.build_profile.insert(format!("instancing.{k}"), v as f64);
            }
        }
        self.add_contact_shadows(&contact);
        self.build_prop_cull();
        self.build_lap("architecture", bt0);
        let total = crate::base::numeric::round_decimals(self.clock_ms() - bt0, 1);
        self.build_profile.insert("t.total".into(), total);
        // keep the layout's copy of the tiles in step with `g.tiles`
        if let Some(l) = self.layout.as_mut() {
            if l.tiles.len() == self.tiles.len() {
                l.tiles.copy_from_slice(&self.tiles);
            }
        }
        self.scene.add(&self.world.clone());
    }
}
