//! The creatures of the redesign (`docs/ENTITIES.md`) at run time: what every
//! kind on the new brain shares.
//!
//! A creature is an `Entity` (its kind, position, heading, and the `state`
//! string the director, the HUD and the screen pressure read, kept in step
//! with its mode by `Mode::legacy_state`) plus a [`Creature`] in its
//! `EntExt`: its body (a `BodyRig` from `bodies.bin`), its brain, its tells
//! and its animation state. Each frame `update_creature` perceives
//! (`perceive`: sight, sound along the way it would walk, your lamp, your
//! eyes), lets its kind decide (`hound::think`, ...), moves it, animates it
//! and voices it.
//!
//! **Fairness** (`docs/ENTITIES.md` design 3) lives here, so no kind can
//! skip it: a strike lands only after two tells you could perceive in the
//! last 20 s, not in a level's first 30 s and not within 2 s of its first
//! notice of you (`strike_allowed`); the first strike of a kind in a run
//! knocks you down instead of killing; a second contact in the same peak,
//! or a contact when your nerve is below 25, kills (`contact`).
use crate::base::numeric::hypot2;
use crate::entities::body::{build_body, BodyRig};
use crate::entities::brain::*;
use crate::entities::director::Phase;
use crate::entities::EntityBuild;
use crate::prelude::*;

/// The kinds on the new brain (the rest still run `ai.rs` until they are rebuilt).
pub fn on_new_brain(kind: &str) -> bool {
    matches!(kind, "hound" | "smiler")
}

/// The one place a kind's own code is chosen: its data and materials, its
/// mind, its body, its voice and its calls.
fn kind_data(kind: &str) -> (KindData, fn(&str) -> Mat) {
    match kind {
        "hound" => (crate::entities::hound::DATA, crate::entities::hound::material),
        "smiler" => (crate::entities::smiler::DATA, crate::entities::smiler::material),
        other => panic!("{other} is not on the new brain (creature::on_new_brain)"),
    }
}
pub fn kind_think(g: &mut Game, ei: usize, dt: f64) {
    match g.entities[ei].kind.as_str() {
        "hound" => crate::entities::hound::think(g, ei, dt),
        "smiler" => crate::entities::smiler::think(g, ei, dt),
        _ => {}
    }
}
pub fn kind_animate(g: &mut Game, ei: usize, dt: f64) {
    match g.entities[ei].kind.as_str() {
        "hound" => crate::entities::hound::animate(g, ei, dt),
        "smiler" => crate::entities::smiler::animate(g, ei, dt),
        _ => {}
    }
}
pub fn kind_voice(g: &mut Game, ei: usize, dt: f64, d: f64) {
    match g.entities[ei].kind.as_str() {
        "hound" => crate::entities::hound::voice(g, ei, dt, d),
        "smiler" => crate::entities::smiler::voice(g, ei, dt, d),
        _ => {}
    }
}
pub fn kind_call(g: &mut Game, ei: usize, what: &str) {
    match g.entities[ei].kind.as_str() {
        "hound" => crate::entities::hound::call(g, ei, what),
        "smiler" => crate::entities::smiler::call(g, ei, what),
        _ => {}
    }
}

/// What a kind's senses and body are: the data that tells kinds apart.
#[derive(Clone, Copy, Debug)]
pub struct KindData {
    /// Sees you in the dark within this, m (0: blind)
    pub sight: f64,
    /// ... and when you are lit or your lamp is on
    pub sight_lit: f64,
    /// Cosine of half its field of view
    pub cone: f64,
    /// Times your audible range: how far it hears you
    pub hearing: f64,
    /// Within this it smells a still player, m (0: never)
    pub smell: f64,
    /// Its body's radius on the floor, m
    pub radius: f64,
    /// How far its strike reaches, m
    pub reach: f64,
    /// How much awareness a second of each sense gives
    pub sight_gain: f64,
    pub sound_gain: f64,
    /// Awareness lost a second when nothing feeds it
    pub forget: f64,
}

/// Seconds a tell counts toward a strike.
pub const TELL_WINDOW: f64 = 20.0;
/// Tells a creature must have given before it may strike.
pub const TELLS_TO_STRIKE: usize = 2;
/// No death this soon after arriving on a level, s.
pub const LEVEL_GRACE: f64 = 30.0;
/// No strike this soon after a creature first noticed you, s.
pub const NOTICE_GRACE: f64 = 2.0;
/// The fastest a creature may chase you for long, m/s (0.95 of your sprint).
pub const CHASE_CAP: f64 = 0.95 * SPRINT;
/// A burst (a lunge, a spring): at most this fast, over at most `BURST_LEN`.
pub const BURST_CAP: f64 = 7.0;
pub const BURST_LEN: f64 = 4.0;

/// Per-limb animation state (feet and hands planted in the world).
#[derive(Clone, Copy, Debug, Default)]
pub struct Limb {
    /// Where it is planted (world x, y, z), and where it swings to
    pub at: [f64; 3],
    pub from: [f64; 3],
    pub to: [f64; 3],
    /// 0 in stance, 0..1 through a swing
    pub swing: f64,
    pub planted: bool,
}

/// A creature of the redesign (in `EntExt::creature`).
#[derive(Clone, Debug)]
pub struct Creature {
    pub brain: Brain,
    pub body: std::rc::Rc<BodyRig>,
    pub data: KindData,
    /// When it gave a tell you could perceive (game time, s)
    pub tells: Vec<f64>,
    /// When it last noticed you (became sure where you are): no strike
    /// within `NOTICE_GRACE` of it
    pub noticed_at: Option<f64>,
    /// Seconds a still player has been within smelling distance
    pub smell_t: f64,
    /// The gait: its phase, and each limb
    pub phase: f64,
    pub limbs: Vec<Limb>,
    /// Smoothed values the animation eases between (by name)
    pub ease: std::collections::BTreeMap<&'static str, f64>,
    /// Seconds to its next sound
    pub voice_t: f64,
    /// The chase so far: how long, and how far in a burst
    pub chase_t: f64,
    pub burst_m: f64,
}

impl Creature {
    pub fn eased(&mut self, key: &'static str, target: f64, rate: f64, dt: f64) -> f64 {
        let v = self.ease.entry(key).or_insert(target);
        *v += (target - *v) * (dt * rate).min(1.0);
        *v
    }
    pub fn get(&self, key: &'static str) -> f64 {
        self.ease.get(key).copied().unwrap_or(0.0)
    }
}

/// What a run remembers of the creatures (`Game::creature_run`): the kinds you
/// have met (their first strike does not kill) and your habits (what they
/// learn, `docs/ENTITIES.md` design 4).
#[derive(Clone, Debug, Default)]
pub struct RunMemory {
    pub met: Vec<String>,
    pub habits: std::collections::BTreeMap<String, u32>,
}

impl Game {
    /// The creature of entity `ei`, if it is on the new brain.
    pub fn creature(&self, ei: usize) -> Option<&Creature> {
        let id = self.entities.get(ei)?.g.id();
        self.ent_ext.get(&id).and_then(|x| x.creature.as_deref())
    }
    pub fn creature_mut(&mut self, ei: usize) -> Option<&mut Creature> {
        let id = self.entities.get(ei)?.g.id();
        self.ent_ext.get_mut(&id).and_then(|x| x.creature.as_deref_mut())
    }

    /// Build a creature of the redesign at tile (x, z) → its `EntityBuild`
    /// (no seeded draw: a level's build stream is untouched).
    pub fn make_creature(&mut self, kind: &str, x: f64, z: f64) -> (EntityBuild, Creature) {
        let (data, material) = kind_data(kind);
        let body = build_body(kind, &material).unwrap_or_else(|| panic!("bodies.bin: no {kind}"));
        let g = body.root.clone();
        g.set_pos(self.wx(x), 0.0, self.wz(z));
        self.world.add(&g);
        let mut parts = std::collections::HashMap::new();
        for (i, b) in body.def.bones.iter().enumerate() {
            parts.insert(b.name.clone(), body.bones[i].clone());
        }
        if let Some(s) = body.skins.first() {
            parts.insert("skin".into(), s.clone());
        }
        let head_y = body.ix("head").map(|i| body.head(i).y).unwrap_or(1.0);
        let c = Creature {
            brain: Brain::new(Mode::Roam),
            body: std::rc::Rc::new(body),
            data,
            tells: vec![],
            noticed_at: None,
            smell_t: 0.0,
            phase: 0.0,
            limbs: vec![],
            ease: Default::default(),
            voice_t: 1.0,
            chase_t: 0.0,
            burst_m: 0.0,
        };
        (EntityBuild { g, face: None, face_y: head_y, parts }, c)
    }

    /// What creature `ei` perceives of you this frame.
    pub fn perceive(&self, ei: usize) -> Percept {
        let Some(c) = self.creature(ei) else { return Percept::default() };
        let kd = c.data;
        let e = &self.entities[ei];
        let pos = e.pos();
        let pp = self.player.pos;
        let dist = hypot2(pp.x - pos.x, pp.z - pos.z);
        let line = dist < 40.0 && self.e_los(pos.x, pos.z, pp.x, pp.z);
        let (dx, dz) = if dist > 1e-6 { ((pp.x - pos.x) / dist, (pp.z - pos.z) / dist) } else { (0.0, 1.0) };
        let in_view = dist < 1.2 || e.yaw.sin() * dx + e.yaw.cos() * dz >= kd.cone;
        let you_lit = self.e_region_light_at(self.tx(pp.x), self.tz(pp.z));
        let it_lit = self.e_region_light_at(self.tx(pos.x), self.tz(pos.z));
        // your eyes: it is in the middle of your view, and you could see it
        let f = self.player.fwd();
        let toward = -(f.x * dx + f.z * dz);
        let watched = line && dist < 26.0 && toward > 0.9;
        let lamp_on_it = self.player.flashlight_on && line && dist < 18.0 && toward > 0.95;
        // sight: in its view, a line, within its range (further when you are lit)
        let lit = you_lit > 0.35 || self.player.flashlight_on;
        let range = if lit { kd.sight_lit } else { kd.sight };
        let sight = if kd.sight > 0.0 && line && in_view && dist < range {
            let k = 1.0 - dist / range;
            (k * (if self.player.crouch { 0.6 } else { 1.0 })
                * (0.5 + 0.5 * you_lit.max(if self.player.flashlight_on { 1.0 } else { 0.0 })))
            .clamp(0.0, 1.0)
        } else {
            0.0
        };
        // sound: your step's reach against the way it would walk to you
        let path_d = self.path_distance(pos.x, pos.z, dist);
        let aud = self.player.audible_range * kd.hearing;
        let sound = if aud > 0.0 && path_d < aud { ((aud - path_d) / (aud * 0.35).max(1.0)).clamp(0.0, 1.0) } else { 0.0 };
        Percept {
            dist,
            line,
            in_view,
            sight,
            sound,
            path: path_d,
            watched,
            lamp_on_it,
            lamp: self.player.flashlight_on,
            you_lit,
            it_lit,
            running: self.player.audible_range >= 12.0,
            crouched: self.player.crouch,
            hidden: false,
        }
    }

    /// The length of the way from (x, z) to you along the level (the player's
    /// flow field, a tile a step), never less than the straight line; `f64::MAX`
    /// when there is no way.
    pub fn path_distance(&self, x: f64, z: f64, straight: f64) -> f64 {
        let Some(field) = self.player_field.as_ref() else { return straight };
        let (tx, tz) = self.e_nav_tile_at(x, z);
        let steps = field.get(self.t_idx(tx, tz)).copied().unwrap_or(-1);
        if steps < 0 {
            return f64::MAX;
        }
        if steps <= 1 {
            return straight;
        }
        ((steps as f64 - 0.5) * CELL).max(straight)
    }

    /// (x, z) if a creature could stand there, else the centre of the nearest
    /// open tile (a guess at where you are never lies in a wall).
    pub fn open_point(&self, x: f64, z: f64) -> (f64, f64) {
        let (tx, tz) = (self.tx(x), self.tz(z));
        if self.in_grid(tx, tz) && !self.is_wall_tile(tx, tz) {
            return (x, z);
        }
        let mut best = (x, z);
        let mut bd = f64::MAX;
        for dz in -3..=3 {
            for dx in -3..=3 {
                let (nx, nz) = (tx + dx, tz + dz);
                if !self.in_grid(nx, nz) || self.is_wall_tile(nx, nz) {
                    continue;
                }
                let (wx, wz) = (self.wx(nx), self.wz(nz));
                let d = hypot2(wx - x, wz - z);
                if d < bd {
                    bd = d;
                    best = (wx, wz);
                }
            }
        }
        best
    }

    /// Creature `ei` gives a tell (a pose, a sound) that carries `reach` m:
    /// it counts toward a strike if you could perceive it.
    pub fn creature_tell(&mut self, ei: usize, reach: f64) {
        let p = self.perceive(ei);
        let pos = self.entities[ei].pos();
        let heard = self.path_distance(pos.x, pos.z, p.dist) < reach;
        let f = self.player.fwd();
        let seen = p.line && p.dist < 30.0 && {
            let (dx, dz) = ((pos.x - self.player.pos.x) / p.dist.max(1e-6), (pos.z - self.player.pos.z) / p.dist.max(1e-6));
            f.x * dx + f.z * dz > 0.5
        };
        let now = self.perf_t;
        if let Some(c) = self.creature_mut(ei) {
            if heard || seen {
                c.tells.retain(|t| now - t < TELL_WINDOW);
                c.tells.push(now);
            }
        }
    }

    /// Whether creature `ei` may strike now (design 3, invariants 1 and 5).
    pub fn strike_allowed(&self, ei: usize) -> bool {
        let now = self.perf_t;
        let Some(c) = self.creature(ei) else { return false };
        let tells = c.tells.iter().filter(|t| now - **t < TELL_WINDOW).count();
        let noticed = c.noticed_at.map(|t| now - t >= NOTICE_GRACE).unwrap_or(false);
        tells >= TELLS_TO_STRIKE && noticed && now - self.level_start_t >= LEVEL_GRACE
    }

    /// Creature `ei`'s strike reached you (design 3, invariants 2 and 3): a
    /// first meeting knocks you down, a second contact in a peak or a contact
    /// on a spent nerve kills. → true when you died.
    pub fn creature_contact(&mut self, ei: usize) -> bool {
        let kind = self.entities[ei].kind.clone();
        self.director_note("contact");
        // it lands where it struck
        let e = &mut self.entities[ei];
        e.vx = 0.0;
        e.vz = 0.0;
        let first = !self.creature_run.met.contains(&kind);
        let lethal = !first && (self.director.contacts > 0 || self.player.sanity < 25.0);
        if lethal {
            self.director.killer = Some(ei);
            self.on_death("entity");
            if self.state != "playing" {
                return true;
            }
            // god mode counted it (`god_deaths`); it backs off as after any strike
        }
        if first {
            self.creature_run.met.push(kind);
        }
        self.director.contacts += 1;
        if !lethal {
            self.knock_down(if first { 30.0 } else { 22.0 });
        }
        // that was its moment: it backs off, and the place settles
        let now = self.perf_t;
        if let Some(c) = self.creature_mut(ei) {
            c.brain.set(Mode::Withdraw, if first { "first meeting: it lets you go" } else { "it struck and backs off" }, now);
            c.brain.order = Order::Away(if first { 45.0 } else { 20.0 });
            // somewhere far from you, chosen afresh (not where it was going)
            c.brain.goal = None;
            c.tells.clear();
        }
        if self.director.phase == Phase::Peak || self.director.stalker == Some(ei) {
            self.director_withdraw_all();
        }
        false
    }

    /// You are thrown down: your nerve goes, your breath goes, the screen
    /// flashes and the view shakes.
    pub fn knock_down(&mut self, nerve: f64) {
        self.player.sanity = (self.player.sanity - nerve).clamp(0.0, SAN_MAX);
        self.player.stamina = 0.0;
        self.ui.flash.set_opacity(1.0);
        self.schedule(350.0, |g| {
            g.ui.flash.set_opacity(0.0);
        });
        self.shake_t = 0.7;
        self.sfx_thump(1.0);
        if self.player.sanity <= 0.0 && !self.god_mode {
            self.on_death("sanity");
        }
    }

    /// Steer creature `ei` toward (x, z) along the level at `speed` (m/s,
    /// capped for fairness), or straight at you when `at_you`.
    pub fn creature_go(&mut self, ei: usize, goal: Option<(f64, f64)>, at_you: bool, speed: f64, dt: f64) {
        let speed = speed.min(CHASE_CAP);
        if at_you {
            let pf = self.player_field.clone();
            let pp = self.player.pos;
            self.entity_step_toward(ei, pf, dt, speed, Some(pp));
            return;
        }
        let Some((gx, gz)) = goal else {
            self.entity_drive(ei, dt, 0.0);
            return;
        };
        let (tx, tz) = self.e_nav_tile_at(gx, gz);
        let same = self.entities[ei].w_target == Some((tx, tz)) && self.entities[ei].w_field.is_some();
        if !same {
            let f = std::rc::Rc::new(self.e_bfs(tx, tz));
            let e = &mut self.entities[ei];
            e.w_target = Some((tx, tz));
            e.w_field = Some(f);
        }
        let wf = self.entities[ei].w_field.clone();
        self.entity_step_toward(ei, wf, dt, speed, Some(v3(gx, 0.0, gz)));
    }

    /// A burst at you (a lunge, a spring): fast, but short (design 3, invariant 4).
    pub fn creature_burst(&mut self, ei: usize, speed: f64, dt: f64) {
        let speed = speed.min(BURST_CAP);
        let used = self.creature(ei).map(|c| c.burst_m).unwrap_or(BURST_LEN);
        let sp = if used < BURST_LEN { speed } else { CHASE_CAP.min(speed) };
        let before = self.entities[ei].pos();
        let pf = self.player_field.clone();
        let pp = self.player.pos;
        self.entity_step_toward(ei, pf, dt, sp, Some(pp));
        let after = self.entities[ei].pos();
        if let Some(c) = self.creature_mut(ei) {
            c.burst_m += hypot2(after.x - before.x, after.z - before.z);
        }
    }

    /// One frame of creature `ei` (its kind decides; this moves the body,
    /// keeps the legacy state in step and syncs the skins).
    pub fn update_creature(&mut self, ei: usize, dt: f64) {
        let p = self.perceive(ei);
        if let Some(c) = self.creature_mut(ei) {
            c.brain.last = p;
            c.brain.mode_t += dt;
            c.brain.t += dt;
        }
        // the director's orders, from the fields it already sets
        self.creature_orders(ei, dt);
        kind_think(self, ei, dt);
        let mode = self.creature(ei).map(|c| c.brain.mode).unwrap_or_default();
        self.entities[ei].state = mode.legacy_state().into();
        let e = &self.entities[ei];
        e.g.rotation().y = e.yaw;
        kind_animate(self, ei, dt);
        if let Some(c) = self.creature(ei) {
            c.body.sync();
        }
    }

    /// Read the director's wishes for creature `ei` into its orders: the
    /// stalker comes toward you unseen; a retreat keeps it away.
    fn creature_orders(&mut self, ei: usize, dt: f64) {
        let stalker = self.director.stalker == Some(ei);
        let retreat = self.entities[ei].n("retreatT");
        if retreat > 0.0 {
            self.entities[ei].set_n("retreatT", retreat - dt);
        }
        let push = self.director.final_push && stalker;
        let pp = self.player.pos;
        if let Some(c) = self.creature_mut(ei) {
            // an away runs its time out (the longer of its own, after a
            // contact, and the director's retreat), whatever else is wished
            let away = match c.brain.order {
                Order::Away(t) if t > 0.0 => (t - dt).max(retreat),
                _ => retreat,
            };
            c.brain.order = if away > 0.0 {
                Order::Away(away)
            } else if push {
                Order::Seek(pp.x, pp.z)
            } else if stalker {
                Order::Stalk
            } else {
                Order::None
            };
        }
    }

    /// Something tells creature `ei` you are at (x, z) (a cry, the bench, an
    /// old caller): it is sure, and comes.
    pub fn creature_alert(&mut self, ei: usize, x: f64, z: f64) {
        let now = self.perf_t;
        if let Some(c) = self.creature_mut(ei) {
            c.brain.mem.awareness = 1.0;
            c.brain.mem.heard = Some((x, z, now));
            if matches!(c.brain.mode, Mode::Withdraw) {
                c.brain.set(Mode::Roam, "called back", now);
            }
        }
    }

    /// Everything after you backs off (the director's relief after a strike).
    pub fn director_withdraw_all(&mut self) {
        if self.director.phase == Phase::Peak {
            self.director_note("withdraw");
        }
        self.director_relax_now();
    }
}
