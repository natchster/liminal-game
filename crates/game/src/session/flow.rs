//! The game flow: `init_level`, save / continue, next level, falling, death,
//! restart and the menu; plus the descent helpers, `beam_battery_scale` and
//! the per-level weather bias.
use crate::base::state::WEATHER_BIASES;
use crate::base::tables::*;
use crate::prelude::*;

/// (v2: the descent begins in the Building, `docs/MEGASTRUCTURE.md`; a v1
/// save's levels and anomaly plan no longer exist)
pub const SAVE_KEY: &str = "opencarole.backrooms.save.v2";

/// What `save_game()` writes (and `load_game()` reads back).
#[derive(Clone, Debug, Default)]
pub struct SaveData {
    pub level_num: i32,
    pub run_seed: u32,
    pub sanity: Option<f64>,
    pub battery: Option<f64>,
    pub bottles: f64,
    pub batteries: f64,
    pub papers: f64,
    pub coins: f64,
    pub lore: Vec<f64>,
    pub archivist: bool,
    pub survivor: bool,
    pub storyteller: bool,
    pub completionist: bool,
    pub ironwill: bool,
    pub deaths: f64,
    pub arcs: Vec<String>,
    /// Her pages taken (`story::notebook` ids)
    pub notebook: Vec<String>,
    /// Her story has ended (`story::ending`)
    pub arc_done: bool,
    /// What the run remembers of the creatures (met, your habits, what they learned)
    pub creatures: crate::entities::creature::RunMemory,
    /// What the run remembers of your asides (said, weighed, counted)
    pub asides: crate::audio::asides::AsideMemory,
    /// What you took (`secrets::keeps`), and the places emptied (key, uses)
    pub took: Vec<crate::secrets::keeps::Took>,
    pub secrets_taken: std::collections::BTreeMap<String, u32>,
}

impl Game {
    /// start-up work: the menu's flavour text and buttons
    pub fn init_flow(&mut self) {
        // The title no longer has a paragraph; its line is still drawn from
        // the native stream, so every frame after it stays where it was
        // (CLAUDE.md invariant 1)
        let _ = crate::base::rng::with_native(|| self.pick_text("overlaySubs"));
        self.load_settings();
        self.hide_overlay();
        self.open_menu(crate::ui::menu::Screen::Title);
    }

    pub fn init_level(&mut self, n: i32) {
        // (a cutscene ends with its floor)
        self.cutscene_end_now("a new floor");
        self.cut_hold = None;
        self.leave_stairwell_mode();
        self.leave_loop_mode();
        self.level_num = n;
        if self.run_seed == 0 {
            self.run_seed = ((native_rnd() * 4294967295.0).floor() as u64 + 1) as u32;
        }
        self.level_scp = self.scp_on_level(n);
        self.store_reset();
        self.house_reset();
        self.level_start_t = self.perf_t;
        self.cries_on_floor = 0;
        if self.run_seed == 0 {
            self.run_seed = ((native_rnd() * 4294967295.0).floor() as u64 + 1) as u32;
        }
        self.current_seed = seed_for_level(self.run_seed, n);
        // the weather first: many placers read it
        self.pick_level_weather();
        let seed = self.current_seed;
        // a level is a function of (run seed, level) alone: it is built as a
        // new run would build it, with nothing read, and its notes are fitted
        // to the save afterwards (`fit_lore_notes`), so a retry, a Continue or
        // a rebuild is the same place
        let found = std::mem::take(&mut self.lore_found);
        let depth = crate::level::building::rooms_depth(n);
        crate::placers::kit::restart_texture_streams();
        with_seed(seed, || {
            // levels 1–3 are the Building's floors; the rooms count their depth from the Lobby
            if crate::level::building::Style::of_level(n).is_some() {
                self.gen_building_level(n);
            } else {
                self.gen_layout_level(depth, (31 + depth * 4).min(53));
            }
            self.trace_mark("layout");
            self.build_world();
            self.trace_mark("world");
            let scp = self.level_scp.clone();
            match scp.as_deref() {
                Some("store") => {
                    let d = self.bfs(1, 1);
                    self.spawn_entities(0, &d);
                    let d = self.bfs(1, 1);
                    self.spawn_staff(4 + 2.min(depth / 5), &d);
                }
                Some("house") => {
                    let d = self.bfs(1, 1);
                    self.spawn_entities(0, &d);
                    let d = self.bfs(1, 1);
                    self.spawn_shadows(3 + if depth >= 5 { 1 } else { 0 }, &d);
                }
                // nothing walks the Building's floors: what comes there comes from far off
                _ if self.building.is_some() => {}
                _ => {
                    let d = self.bfs(1, 1);
                    self.spawn_entities(1 + ((depth - 1) / 2).min(2), &d);
                }
            }
        });
        self.lore_found = found;
        self.fit_lore_notes();
        self.fit_story();
        self.fit_secrets();
        self.fit_knocks();
        // the one in the corner, after her camp (never near it): by the place
        crate::entities::corner::place(self);
        // the Reader, where paper lies
        crate::entities::reader::place(self);
        // the one behind the glass, behind a window off the way
        crate::entities::glass::place(self);
        // a kind held back from learning on the last level learns now
        self.creature_learn_pending();
        self.build_light_map();
        self.update_ambience();
        self.release_pooled_lights();
        self.dispose_unused();
        let t0 = self.clock_ms();
        self.warm_up_gpu();
        let dt = self.clock_ms() - t0;
        self.build_profile.insert("compileMs".into(), (dt * 10.0).round() / 10.0);
        self.player.pos = self.spawn_pos;
        self.player.yaw = -PI / 4.0;
        self.player.pitch = 0.0;
        self.player.stamina = self.body.stam_max;
        self.size_eye = if self.player.crouch { self.body.crouch_eye } else { self.body.eye };
        self.player.sanity = SAN_MAX.min(self.player.sanity + 30.0);
        self.player.battery = BAT_MAX.min(self.player.battery + 20.0);
        self.heart_t = 0.0;
        self.reset_director();
        self.pick_level_weather();
        // per-level safe-room hum detune (stable per (run seed, level))
        if let (Some(lfo), Some(car)) = (self.audio.safe_room_lfo.clone(), self.audio.safe_room_trem_carrier.clone()) {
            let cs = self.current_seed;
            let lfo_jit = ((cs & 0xff) as f64 / 255.0 - 0.5) * 0.20;
            let car_jit = (((cs >> 8) & 0xff) as f64 / 255.0 - 0.5) * 40.0;
            let wk = self.weather_key();
            let lfo_weather_mul = lfo_weather_mul(wk).unwrap_or(1.0);
            let car_weather_off = carrier_weather_detune(wk).unwrap_or(0.0);
            let level_depth = self.level_depth_mul();
            lfo.frequency().set_value((0.35 + lfo_jit) * lfo_weather_mul * level_depth);
            car.frequency().set_value((330.0 + car_jit + car_weather_off) * level_depth);
        }
        // staggered ambient cadences and the level's signature sounds (native
        // draws: the seeded build is over)
        self.arm_level_ambient();
        self.ui.level.set_text(&format!("LEVEL {}", n));
        // level entry chime, 1.2 s later
        self.schedule(1200.0, |g| {
            if g.state != "playing" || g.ac.is_none() {
                return;
            }
            let mel = recovery_melody_by_weather(g.weather_key()).or(recovery_melody_by_weather("normal")).unwrap();
            let depth_shift = g.chime_depth_mul();
            g.blip(mel.root * depth_shift, 0.60, "sine", 0.030, None, 0.0, 0.0);
            g.blip(mel.up * depth_shift, 0.75, "sine", 0.022, None, 0.35, 0.0);
        });
        if n >= 10 && !self.survivor_shown {
            self.schedule(2400.0, |g| g.fire_survivor_achievement());
        }
        if n >= 5 && self.death_count == 0.0 && !self.ironwill_shown {
            self.schedule(2400.0, |g| g.fire_ironwill_achievement());
        }
        let seed_tag = format!("SEED 0x{:08x}  ·  {}", self.current_seed, self.level_weather.key.to_uppercase());
        self.ui.seed_tag.set_text(&seed_tag);
        // nothing is told (PILLARS 4): the only line up there is her last
        // one you read (`show_thread`, by `fit_story` above)
        for (dx, dz, yaw) in [(1, 0, -PI / 2.0), (0, 1, PI), (-1, 0, PI / 2.0), (0, -1, 0.0)] {
            if !self.is_wall_tile(1 + dx, 1 + dz) {
                self.player.yaw = yaw;
                break;
            }
        }
        // your asides: the floor's arrival waits for your first steps
        let place = self.arrival_subject();
        self.asides_arrive(place);
    }

    pub fn trace_mark(&mut self, label: &str) {
        if self.rand_trace.on {
            let n = crate::base::rng::seeded_draws() - self.rand_trace.n;
            self.rand_trace.marks.push((label.to_string(), n));
        }
    }

    // ------------------------------------------------------------ save / continue
    pub fn save_game(&mut self) {
        let lore: Vec<serde_json::Value> = self.lore_found.iter().map(|&i| serde_json::json!(i)).collect();
        let arcs: Vec<serde_json::Value> = self.lore_arcs_completed.iter().map(|a| serde_json::json!(a)).collect();
        let data = serde_json::json!({
            "levelNum": self.level_num,
            "runSeed": self.run_seed,
            "sanity": self.player.sanity,
            "battery": self.player.battery,
            "bottles": self.player.bottles,
            "batteries": self.player.batteries,
            "papers": self.player.papers,
            "coins": self.player.coins,
            "lore": lore,
            "archivist": self.lore_archivist_shown,
            "survivor": self.survivor_shown,
            "storyteller": self.storyteller_shown,
            "completionist": self.completionist_shown,
            "ironwill": self.ironwill_shown,
            "deaths": self.death_count,
            "arcs": arcs,
            "notebook": self.notebook.iter().collect::<Vec<_>>(),
            "arcDone": self.arc_done,
            "creatures": self.creature_run.to_json(),
            "asides": self.asides.memory.to_json(),
            "took": self.took.iter().map(|t| serde_json::json!([t.what, t.level, t.place])).collect::<Vec<_>>(),
            "secrets": self.secrets_taken,
            "savedAt": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as f64).unwrap_or(0.0),
        });
        self.storage_set(SAVE_KEY, &data.to_string());
    }
    pub fn load_game(&self) -> Option<SaveData> {
        let raw = self.storage_get(SAVE_KEY)?;
        let d: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let level_num = d.get("levelNum")?.as_f64()?;
        if level_num < 1.0 {
            return None;
        }
        let num = |k: &str| d.get(k).and_then(|v| v.as_f64());
        let flag = |k: &str| {
            d.get(k).map(|v| v.as_bool().unwrap_or(!v.is_null() && v.as_f64().map(|x| x != 0.0).unwrap_or(false))).unwrap_or(false)
        };
        // integer fields: truncated and wrapped to 32 bits (`to_i32`)
        let int = |k: &str| num(k).map(to_i32).unwrap_or(0) as f64;
        Some(SaveData {
            level_num: level_num as i32,
            run_seed: num("runSeed").map(to_u32).unwrap_or(0),
            sanity: num("sanity"),
            battery: num("battery"),
            bottles: int("bottles"),
            batteries: int("batteries"),
            papers: int("papers"),
            coins: int("coins"),
            lore: d.get("lore").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_f64()).collect()).unwrap_or_default(),
            archivist: flag("archivist"),
            survivor: flag("survivor"),
            storyteller: flag("storyteller"),
            completionist: flag("completionist"),
            ironwill: flag("ironwill"),
            deaths: int("deaths"),
            arcs: d
                .get("arcs")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            arc_done: flag("arcDone"),
            creatures: crate::entities::creature::RunMemory::from_json(d.get("creatures")),
            asides: crate::audio::asides::AsideMemory::from_json(d.get("asides")),
            took: d
                .get("took")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|t| {
                            let t = t.as_array()?;
                            Some(crate::secrets::keeps::Took {
                                what: t.first()?.as_str()?.to_string(),
                                level: t.get(1)?.as_f64()? as i32,
                                place: t.get(2)?.as_str()?.to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
            secrets_taken: d
                .get("secrets")
                .and_then(|v| v.as_object())
                .map(|o| o.iter().filter_map(|(k, v)| Some((k.clone(), v.as_f64()? as u32))).collect())
                .unwrap_or_default(),
            notebook: d
                .get("notebook")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().filter(|s| crate::story::notebook::page(s).is_some()).map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
    pub fn clear_save(&mut self) {
        self.storage_remove(SAVE_KEY);
    }
    pub fn apply_saved(&mut self, d: &SaveData) {
        self.player.sanity = clamp(d.sanity.unwrap_or(SAN_MAX), 0.0, SAN_MAX);
        self.player.battery = clamp(d.battery.unwrap_or(BAT_MAX), 0.0, BAT_MAX);
        self.player.bottles = (d.bottles as i32).max(0);
        self.player.batteries = (d.batteries as i32).max(0);
        self.player.papers = (d.papers as i32).max(0);
        self.player.coins = (d.coins as i32).max(0);
        let n = self.lore_entry_count();
        self.lore_found = d.lore.iter().filter(|&&i| i.fract() == 0.0 && i >= 0.0 && (i as usize) < n).map(|&i| i as usize).collect();
        self.lore_archivist_shown = d.archivist;
        self.survivor_shown = d.survivor;
        self.storyteller_shown = d.storyteller;
        self.completionist_shown = d.completionist;
        self.ironwill_shown = d.ironwill;
        self.death_count = d.deaths.max(0.0);
        self.lore_arcs_completed = d.arcs.iter().cloned().collect();
        self.notebook = d.notebook.iter().cloned().collect();
        self.arc_done = d.arc_done;
        self.creature_run = d.creatures.clone();
        self.asides.memory = d.asides.clone();
        self.took = d.took.clone();
        self.secrets_taken = d.secrets_taken.clone();
        self.refresh_body();
        // the thread: the last line of the latest page of hers taken
        self.thread = crate::story::notebook::notebook(self.run_seed)
            .into_iter()
            .rev()
            .find(|(_, p)| self.notebook.contains(p.id))
            .map(|(_, p)| p.thread.to_string())
            .unwrap_or_default();
        self.refresh_lore_medal();
        self.refresh_survivor_medal();
        self.refresh_storyteller_medal();
        self.refresh_completionist_medal();
        self.refresh_ironwill_medal();
        self.schedule(400.0, |g| g.replay_medal_highlight());
    }

    /// The next level's fog tint, lifted
    pub fn next_weather_fade_color(&self, _fallback: &str) -> String {
        let next_seed = seed_for_level(self.run_seed, self.level_num + 1);
        let bias = &WEATHER_BIASES[(next_seed as usize) % WEATHER_BIASES.len()];
        let hex = bias.fog_hex;
        let lift = 90;
        let rr = 255.min(((hex >> 16) & 0xff) + lift);
        let gg = 255.min(((hex >> 8) & 0xff) + lift);
        let bb = 255.min((hex & 0xff) + lift);
        format!("rgb({},{},{})", rr, gg, bb)
    }

    pub fn next_level(&mut self) {
        if self.state != "playing" {
            return;
        }
        if self.arc_ends_here() {
            return self.begin_ending();
        }
        self.state = "transition".into();
        self.sfx_door();
        if self.ac.is_some() {
            let mel = recovery_melody_by_weather(self.weather_key()).or(recovery_melody_by_weather("normal")).unwrap();
            let ds = self.chime_depth_mul();
            self.blip(mel.up * ds, 0.60, "sine", 0.030, None, 0.0, 0.0);
            self.blip(mel.root * ds, 0.75, "sine", 0.022, None, 0.30, 0.0);
        }
        let c = self.next_weather_fade_color("#f5edd2");
        self.ui.fade.set_style("background", &c);
        self.ui.fade.set_opacity(1.0);
        self.schedule(650.0, |g| {
            g.init_level(g.level_num + 1);
            g.save_game();
            g.state = "playing".into();
            let title = g.level_title();
            g.show_level_card(&title);
            g.scp_arrival();
            g.ui.fade.set_opacity(0.0);
        });
    }
    pub fn next_level_via_elevator(&mut self) {
        if self.state != "playing" {
            return;
        }
        if self.arc_ends_here() {
            return self.begin_ending();
        }
        self.state = "transition".into();
        self.sfx_door_slam();
        self.sfx_clank();
        self.schedule(400.0, |g| g.sfx_clank());
        self.schedule(900.0, |g| g.sfx_dial_tone());
        if self.ac.is_some() {
            let brk = breaking_melody_by_weather(self.weather_key()).or(breaking_melody_by_weather("normal")).unwrap();
            let ds = self.chime_depth_mul();
            self.blip(brk.a * 2.0 * ds, 0.35, "sine", 0.028, None, 0.15, 0.0);
            self.blip(brk.b * 2.0 * ds, 0.40, "sine", 0.024, None, 0.45, 0.0);
            self.blip(brk.c * 2.0 * ds, 0.55, "sine", 0.020, None, 0.75, 0.0);
        }
        let c = self.next_weather_fade_color("#0a0808");
        self.ui.fade.set_style("background", &c);
        self.ui.fade.set_opacity(1.0);
        self.schedule(1500.0, |g| {
            g.init_level(g.level_num + 1);
            g.save_game();
            g.state = "playing".into();
            let sub = g.pick_text("levelSubtitle");
            g.show_level_card(&sub);
            g.ui.fade.set_opacity(0.0);
        });
    }
    /// A fast fall through a hole, then the next level
    pub fn begin_fall_down(&mut self) {
        if self.state != "playing" {
            return;
        }
        if self.arc_ends_here() {
            return self.begin_ending();
        }
        self.state = "transition".into();
        self.falling_t = 1.2;
        self.sfx_distant_scream();
        self.noise_burst(1.2, 500.0, 0.5, 0.25, "lowpass", Some(90.0), 0.0, 0.0);
        self.ui.fade.set_style("background", "#000");
        self.ui.fade.set_opacity(0.0);
        let start = self.now_ms;
        fn tick(g: &mut Game, start: f64) {
            let t = (g.now_ms - start) / 1200.0;
            g.camera.position().y = g.floor_y_at(g.player.pos.x, g.player.pos.z) + EYE - t * 4.5;
            g.camera.rotation().z += 0.02;
            g.ui.fade.set_opacity(1f64.min(t * 1.4));
            if t >= 1.0 {
                g.init_level(g.level_num + 1);
                g.save_game();
                g.camera.rotation().z = 0.0;
                g.state = "playing".into();
                g.show_level_card("You fell");
                g.schedule(400.0, |g| g.ui.fade.set_opacity(0.0));
            } else {
                g.schedule(30.0, move |g| tick(g, start));
            }
        }
        self.schedule(30.0, move |g| tick(g, start));
    }

    pub fn on_death(&mut self, cause: &str) {
        if self.state != "playing" {
            return;
        }
        self.director_note("death");
        if self.god_mode {
            self.god_deaths += 1.0;
            *self.god_causes.entry(cause.to_string()).or_insert(0.0) += 1.0;
            self.player.sanity = self.player.sanity.max(40.0);
            return;
        }
        // a death ends any cutscene where it is
        self.cutscene_end_now("a death");
        self.state = "dead".into();
        self.death_count += 1.0;
        self.last_death_cause = Some(cause.to_string());
        self.last_death_t = self.ac_time();
        let wk = self.weather_key().to_string();
        *self.death_tally.entry(cause.to_string()).or_default().entry(wk.clone()).or_insert(0.0) += 1.0;
        self.save_game();
        self.sfx_scream();
        if let (Some(_), Some(gain), Some(lfo), Some(lp)) = (
            self.ac.as_ref(),
            self.audio.stamina_breath_gain.clone(),
            self.audio.stamina_breath_lfo.clone(),
            self.audio.stamina_breath_lp.clone(),
        ) {
            let br = stamina_breath_by_weather(&wk).or(stamina_breath_by_weather("normal")).unwrap();
            let is_sanity = cause == "sanity";
            let finale_gain = br.gain * if is_sanity { 3.0 } else { 4.4 };
            let finale_rate = if is_sanity { 0.60 } else { 1.00 };
            let finale_dur = if is_sanity { 2.6 } else { 1.5 };
            let finale_lp = if is_sanity { 380.0 } else { 500.0 };
            let t0 = self.ac_time();
            gain.gain().cancel_scheduled_values(t0);
            gain.gain().set_value_at_time(finale_gain, t0);
            gain.gain().exponential_ramp_to_value_at_time(0.0001, t0 + finale_dur);
            lfo.frequency().cancel_scheduled_values(t0);
            lfo.frequency().set_value_at_time(finale_rate, t0);
            lfo.frequency().linear_ramp_to_value_at_time(br.rate_base * 0.9, t0 + finale_dur);
            lp.frequency().cancel_scheduled_values(t0);
            lp.frequency().set_value_at_time(finale_lp, t0);
            lp.frequency().linear_ramp_to_value_at_time(220.0, t0 + finale_dur);
            if let Some(sub) = self.audio.stamina_breath_sub_gain.clone() {
                let finale_sub = br.sub_gain * if is_sanity { 2.5 } else { 3.5 };
                sub.gain().cancel_scheduled_values(t0);
                sub.gain().set_value_at_time(finale_sub, t0);
                sub.gain().exponential_ramp_to_value_at_time(0.0001, t0 + finale_dur);
            }
        }
        if self.ac.is_some() {
            let brk = breaking_melody_by_weather(&wk).or(breaking_melody_by_weather("normal")).unwrap();
            let dd = self.chime_depth_mul();
            self.blip(brk.a * 0.5 * dd, 1.6, "sine", 0.08, None, 0.20, 0.0);
            self.blip(brk.b * 0.5 * dd, 1.8, "sine", 0.06, None, 0.35, 0.0);
            self.blip(brk.c * 0.5 * dd, 2.2, "sine", 0.05, None, 0.55, 0.0);
            self.blip(brk.c * 0.25 * dd, 2.5, "sine", 0.045, None, 0.70, 0.0);
        }
        self.shake_t = 1.0;
        let flash = match wk.as_str() {
            "wet" => "radial-gradient(ellipse at center, rgba(140,0,20,0.85) 0%, rgba(20,0,10,0.95) 100%)",
            "electric" => "radial-gradient(ellipse at center, rgba(200,180,40,0.85) 0%, rgba(80,20,0,0.95) 100%)",
            "haunted" => "radial-gradient(ellipse at center, rgba(130,20,80,0.85) 0%, rgba(20,0,30,0.95) 100%)",
            "derelict" => "radial-gradient(ellipse at center, rgba(160,80,20,0.85) 0%, rgba(60,20,0,0.95) 100%)",
            _ => "radial-gradient(ellipse at center, rgba(120,0,0,0.85) 0%, rgba(20,0,0,0.95) 100%)",
        };
        self.ui.flash.set_style("background", flash);
        // the flash comes a beat late, so you see it first
        self.schedule(260.0, |g| g.ui.flash.set_opacity(0.85));
        self.schedule(450.0, |g| g.ui.flash.set_opacity(0.0));
        // the thing that got you, in your face; the screen cuts to black on it
        let killer = self.director.killer.take().filter(|&i| i < self.entities.len()).or_else(|| self.nearest_entity());
        self.director.killer = killer;
        if let Some(ki) = killer {
            let f = self.player.fwd();
            let (px, pz) = (self.player.pos.x, self.player.pos.z);
            let e = &mut self.entities[ki];
            e.yaw = (-f.x).atan2(-f.z);
            if e.kind == "hound" {
                // mid-leap, reared up into your face
                e.g.position().set(px + f.x * 0.7, 0.75, pz + f.z * 0.7);
                e.g.set_rotation_order(EulerOrder::YXZ);
                e.g.rotation().x = -0.75;
            } else {
                e.g.position().set(px + f.x * 0.8, 0, pz + f.z * 0.8);
            }
            e.g.rotation().y = e.yaw;
            e.state = "hunt".into();
            e.set_n("noticeT", 1.0);
            e.set_n("lungeT", 1.4);
            self.entity_call(ki, "shriek");
        }
        self.ui.fade.set_style("background", "#000");
        // the death's own cutscene, by what killed you (`docs/CUTSCENES.md`):
        // its fall, its black, and the card at its end (1.4 s, as always)
        let id = match cause {
            "entity" => "death_taken",
            "assembly" => "death_fog",
            _ => "death_mind",
        };
        let anchors = crate::cutscene::play::Anchors { thing: None, killer };
        if !self.play_cutscene(id, anchors) {
            // none to play (a creature's death with no creature): black on
            // the clock, then the card. Black, the hum, one line in the
            // world's voice, and Wake up (no title, no seconds survived).
            // The card's three draws are kept, so nothing after a death
            // moves (CLAUDE.md invariant 1)
            self.schedule(450.0, |g| g.ui.fade.set_opacity(1.0));
            self.schedule(1400.0, |g| g.open_death_card());
        }
    }
    /// The death card: black, one line in the world's voice, Wake up (a
    /// death's cutscene opens it at its end, `then = "card"`)
    pub fn open_death_card(&mut self) {
        if self.state != "dead" || self.live_menu().is_some() {
            return;
        }
        self.want_pointer_lock = Some(false);
        let _ = self.pick_text("deathTitle");
        let line = self.pick_text("deathBodyBits");
        let _ = self.pick_text("deathHint");
        self.open_menu(crate::ui::menu::Screen::Dead);
        self.menu.heading = line;
    }
    pub fn restart(&mut self) {
        self.ui.fade.set_opacity(0.0);
        let n = self.level_num;
        self.close_menu();
        self.reset_hands();
        self.init_level(n);
        self.hide_overlay();
        self.state = "playing".into();
        self.want_pointer_lock = Some(true);
        self.show_level_card("");
    }

    // ------------------------------------------------------------ the menu
    /// The launch flags (`--flags 'seed=4242&level=3'`) as key → value.
    pub fn parse_flag_params(&self) -> std::collections::BTreeMap<String, String> {
        let mut out = std::collections::BTreeMap::new();
        for kv in self.launch_flags.split('&') {
            let mut it = kv.splitn(2, '=');
            let k = it.next().unwrap_or("");
            if !k.is_empty() {
                out.insert(k.to_string(), it.next().unwrap_or("").to_string());
            }
        }
        out
    }
    pub fn start_new_game(&mut self) {
        self.start_new_game_with(None, true)
    }
    /// A new run: on the run seed `seed` if given (New descent keeps the
    /// title's place), else the launch flags' or a new one; the level card
    /// now, or (`card_now` false) kept for the way in to show at its end
    /// (`way_in_card`; the line is drawn here either way, so nothing after
    /// it moves)
    pub fn start_new_game_with(&mut self, seed: Option<u32>, card_now: bool) {
        self.close_menu();
        self.reset_hands();
        self.audio_init();
        self.clear_save();
        self.title_place = None;
        let hp = self.parse_flag_params();
        let hash_seed = hp.get("seed").and_then(|s| parse_int_prefix(s)).map(to_u32).unwrap_or(0);
        self.run_seed = match seed {
            Some(s) => s,
            None if hash_seed != 0 => hash_seed,
            None => ((native_rnd() * 4294967295.0).floor() as u64 + 1) as u32,
        };
        self.lore_found.clear();
        self.lore_arcs_completed.clear();
        self.creature_run = Default::default();
        self.asides = Default::default();
        self.cut_seen.clear();
        self.notebook.clear();
        self.thread.clear();
        self.secrets_taken.clear();
        self.took.clear();
        self.refresh_body();
        self.last_used = None;
        self.journal_newest = None;
        self.arc_done = false;
        self.lore_archivist_shown = false;
        self.survivor_shown = false;
        self.storyteller_shown = false;
        self.completionist_shown = false;
        self.ironwill_shown = false;
        self.death_count = 0.0;
        self.refresh_lore_medal();
        self.refresh_survivor_medal();
        self.refresh_storyteller_medal();
        self.refresh_completionist_medal();
        self.refresh_ironwill_medal();
        self.player.bottles = 0;
        self.player.batteries = 0;
        self.player.papers = 0;
        self.player.coins = 0;
        self.hinted_bottle = false;
        self.hinted_battery = false;
        let start_level = hp.get("level").and_then(|s| parse_int_prefix(s)).map(|v| 1.max(to_i32(v))).unwrap_or(1);
        self.init_level(start_level);
        self.state = "playing".into();
        self.hide_overlay();
        let sub = self.pick_text("levelSubtitle");
        if card_now {
            self.show_level_card(&sub);
        } else {
            self.way_in_card = Some(sub);
        }
        self.scp_arrival();
        self.want_pointer_lock = Some(true);
    }
    pub fn continue_game(&mut self) {
        self.close_menu();
        self.title_place = None;
        self.reset_hands();
        self.audio_init();
        let Some(d) = self.load_game() else {
            return self.start_new_game();
        };
        if d.run_seed > 0 {
            self.run_seed = d.run_seed;
        }
        // build with nothing read, then fit the notes (and her page) to the save's
        self.lore_found.clear();
        self.notebook.clear();
        self.thread.clear();
        self.init_level(d.level_num);
        self.apply_saved(&d);
        self.fit_lore_notes();
        self.fit_story();
        self.fit_secrets();
        self.fit_knocks();
        self.state = "playing".into();
        self.hide_overlay();
        self.show_level_card("");
        self.scp_arrival();
        self.want_pointer_lock = Some(true);
    }

    // ------------------------------------------------------------ descent helpers
    /// How far into the rooms this level is (`level::building::rooms_depth`)
    pub fn depth(&self) -> i32 {
        crate::level::building::rooms_depth(self.level_num)
    }
    pub fn level_depth_mul(&self) -> f64 {
        let slope = level_depth_slope_by_weather(self.weather_key()).or(level_depth_slope_by_weather("normal")).unwrap();
        0.85f64.max(1.0 - (self.depth() - 1) as f64 * slope)
    }
    pub fn chime_depth_mul(&self) -> f64 {
        let slope = level_depth_slope_by_weather(self.weather_key()).or(level_depth_slope_by_weather("normal")).unwrap();
        0.80f64.max(1.0 - (self.depth() - 1) as f64 * slope * 4.0)
    }
    pub fn descent_boost_mul(&self, f: f64) -> f64 {
        1.0 + (1.0 - self.level_depth_mul()) * f
    }
    pub fn descent_freq_boost(&self) -> f64 {
        1.0 + (1.0 - self.level_depth_mul()) * 0.2
    }
    pub fn descent_amp_boost(&self) -> f64 {
        1.0 + (1.0 - self.level_depth_mul()) * 0.3
    }
    pub fn beam_battery_scale(&self, min: f64) -> f64 {
        if !self.player.flashlight_on {
            return 1.0;
        }
        let t = clamp(self.player.battery / 40.0, 0.0, 1.0);
        min + (1.0 - min) * t
    }
    /// `map` looked up with this level's weather key, or `fallback` when it has no entry
    pub fn weather_pick<T>(&self, map: impl Fn(&str) -> Option<T>, fallback: T) -> T {
        map(self.level_weather.key).unwrap_or(fallback)
    }

    // ------------------------------------------------------------ weather
    pub fn apply_weather_bias(&mut self, bias: &'static crate::base::state::Weather) {
        self.level_weather = bias;
        // reweight the ambient events per sfx (dampens win over boosts)
        self.biased_ambient = AMBIENT_EVENTS
            .iter()
            .map(|&(w, name)| {
                let mut scale = 1.0;
                if !name.is_empty() {
                    if let Some((_, b)) = bias.boosts.iter().find(|(k, _)| *k == name) {
                        scale = *b;
                    }
                    if let Some((_, d)) = bias.dampens.iter().find(|(k, _)| *k == name) {
                        scale = *d;
                    }
                }
                (1f64.max(round_half_up(w * scale)), name)
            })
            .collect();
        let cd = self.chime_depth_mul();
        self.fog_color.set_hex(bias.fog_hex);
        self.fog_color.r *= cd;
        self.fog_color.g *= cd;
        self.fog_color.b *= cd;
        let fog_descent = 1.0 + (1.0 - self.chime_depth_mul()) * 1.05;
        self.fog_density = 0.052 * bias.fog_mul * fog_descent;
        self.background.set_hex(bias.bg_hex);
        let cd = self.chime_depth_mul();
        self.background.r *= cd;
        self.background.g *= cd;
        self.background.b *= cd;
        // full-screen weather tint (CSS class swap)
        let classes: Vec<String> = self.ui.weather_tint.classes.iter().cloned().collect();
        for c in classes {
            self.ui.weather_tint.remove_class(&c);
        }
        if bias.key != "normal" {
            self.ui.weather_tint.add_class(bias.key);
        }
    }
    /// Deterministic per (run seed, level)
    pub fn pick_level_weather(&mut self) {
        // the Building's floors have none of the rooms' weathers: their air
        // is the Building's own (`docs/MEGASTRUCTURE.md`)
        let idx = if crate::level::building::Style::of_level(self.level_num).is_some() {
            WEATHER_BIASES.iter().position(|w| w.key == "normal").unwrap_or(0)
        } else {
            (self.current_seed as usize) % WEATHER_BIASES.len()
        };
        self.apply_weather_bias(&WEATHER_BIASES[idx]);
    }
}

/// The leading decimal integer of `s` (after whitespace and an optional sign);
/// None when there are no digits.
pub fn parse_int_prefix(s: &str) -> Option<f64> {
    let t = s.trim();
    let (neg, rest) = if let Some(r) = t.strip_prefix('-') { (true, r) } else { (false, t.strip_prefix('+').unwrap_or(t)) };
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let v: f64 = digits.parse().ok()?;
    Some(if neg { -v } else { v })
}
