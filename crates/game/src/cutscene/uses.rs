//! Where the game plays its cutscenes (`docs/design/CUTSCENES.md`): the way in
//! from the title on New descent, and a broken lift's shaft. (The deaths
//! are started by `on_death`, `session/flow.rs`, by their cause.)
//! Each starts a cutscene of the library the way play meets it and does
//! what must happen around it; the cutscene itself is data.
use super::Cutscene;
use super::play::{Anchors, Frame, Pose};
use crate::prelude::*;

/// The shaft's eyeshine: where it opens in the lift's frame (low in the
/// black between the doors, a hair in front of its face: the black is a box
/// 6 cm deep about z 0.10), how far apart, how big; what looking takes of
/// your nerve
const SHAFT_EYES: (f64, f64, f64) = (0.2, 0.98, 0.134);
const SHAFT_EYES_APART: f64 = 0.009;
const SHAFT_EYE_R: f64 = 0.0034;
const SHAFT_NERVE: f64 = 12.0;
/// The light the points give back: your lamp's, or the red lamp's
const SHAFT_EYES_LAMP: u32 = 0xd2dc84;
const SHAFT_EYES_RED: u32 = 0xe8563e;
/// The lift's lamp stands this far out from its foot (`tail_broken_elevator`)
const SHAFT_LAMP_OUT: f64 = 0.4;

/// Through black, when the title shows another floor than the new run's:
/// closing, then the new Well's title view
const NEW_PLACE_FADE_MS: f64 = 600.0;

impl Game {
    /// New descent, from the title: the way in (`way_in`)
    pub fn new_descent(&mut self) {
        let c = super::get("way_in").cloned();
        self.new_descent_with(c);
    }

    /// New descent with the way in `c` (a preview's or a test's own, or
    /// none: straight in, the card at once). When the title shows level 1
    /// of a run nobody has played (`title_place`), the run keeps its seed:
    /// the floor built again is the same place, under the camera as it
    /// was, and the way in starts from the title's view with no cut. When
    /// it shows a floor of another run, a new run is made behind black and
    /// the way in starts from its own title view.
    pub fn new_descent_with(&mut self, c: Option<Cutscene>) {
        // (a launch flag's seed or level makes another place than the title's)
        let flags = self.parse_flag_params();
        let flagged = flags.contains_key("seed") || flags.get("level").is_some_and(|l| l.trim() != "1");
        let same = !flagged && self.title_place.is_some_and(|(seed, level)| seed == self.run_seed && level == 1 && self.level_num == 1);
        // the title's list and its shade fade out as the bars come in
        if self.menu.screen.is_some() {
            self.menu_fade = Some((self.menu.clone(), self.now_ms));
        }
        if same {
            let title = self.camera_pose();
            let seed = self.run_seed;
            self.start_new_game_with(Some(seed), c.is_none());
            self.face_the_title();
            // (the same place: the camera stays where the title had it)
            self.set_camera_pose(&title);
            self.play_way_in(c);
            return;
        }
        self.close_menu();
        self.ui.fade.set_style("background", "#000");
        self.ui.fade.set_opacity(1.0);
        self.schedule(NEW_PLACE_FADE_MS, move |g| {
            g.start_new_game_with(None, c.is_none());
            g.face_the_title();
            let (pos, yaw) = g.title_view();
            let fov = g.camera_pose().fov;
            g.set_camera_pose(&Pose { pos, yaw, pitch: -0.03, roll: 0.0, fov });
            g.play_way_in(c);
            g.ui.fade.set_opacity(0.0);
        });
    }

    /// A run begun from the title faces the way the title looked (its
    /// start pose, set before the way in: the cutscene never moves you)
    fn face_the_title(&mut self) {
        let (_, yaw) = self.title_view();
        self.player.yaw = yaw;
        self.player.pitch = 0.0;
    }

    fn play_way_in(&mut self, c: Option<Cutscene>) {
        let played = match c {
            Some(c) => self.play_cutscene_data(c, Anchors::default()),
            None => false,
        };
        if !played {
            self.way_in_arrive();
        }
    }

    /// The way in's end (its `arrive` event, run on a skip too): the level
    /// card the run kept for it
    pub fn way_in_arrive(&mut self) {
        if let Some(sub) = self.way_in_card.take() {
            self.show_level_card(&sub);
        }
    }

    /// "Peer into the shaft" at a broken lift: `shaft`, from where the lift
    /// stands (its frame: its front out of its wall); refused in danger or
    /// while another plays, nothing happens and the lift waits
    pub fn peer_into_shaft(&mut self) {
        let Some(thing) = self.shaft_frame() else { return };
        if self.broken_elevator.as_ref().is_some_and(|b| b.peered) {
            return;
        }
        self.play_cutscene("shaft", Anchors { thing: Some(thing), killer: None, seen_as: None });
    }

    /// The broken lift's frame: its foot on the floor, its front out of
    /// its wall. Where it stands is read from its lamp, a child of its group
    /// 0.4 m out from its foot: settling moves the group onto the wall's
    /// real face (a metre out on some floors) after `world` was recorded
    pub fn shaft_frame(&self) -> Option<Frame> {
        let be = self.broken_elevator.as_ref()?;
        let (l, n) = (be.light.world_position(), be.normal);
        let (x, z) = (l.x - n.x * SHAFT_LAMP_OUT, l.z - n.z * SHAFT_LAMP_OUT);
        Some(Frame::thing([x, self.floor_y_at(x, z), z], n.x.atan2(n.z)))
    }

    /// Two small points low in the black between the doors, giving your
    /// light back (your lamp's, or the red lamp's when yours is off; seen
    /// from a hand's breadth in front of the black they read as far down);
    /// only the first lift you look into in a run (the same scare never
    /// soon again: later, only the black and the breath)
    pub fn shaft_eyes(&mut self) {
        let Some(f) = self.shaft_frame() else { return };
        if self.shaft_looked_back {
            return;
        }
        self.shaft_looked_back = true;
        self.shaft_eyes_gone();
        let colour = if self.player.flashlight_on { SHAFT_EYES_LAMP } else { SHAFT_EYES_RED };
        let eyes = group();
        for dx in [-SHAFT_EYES_APART / 2.0, SHAFT_EYES_APART / 2.0] {
            let p = f.at([SHAFT_EYES.0 + dx, SHAFT_EYES.1, SHAFT_EYES.2]);
            let e = mesh(sphere_geo!(SHAFT_EYE_R, 8, 6), basic_mat!({ color: colour }));
            e.set_pos(p[0], p[1], p[2]);
            eyes.add(&e);
        }
        self.world.add(&eyes);
        if let Some(be) = self.broken_elevator.as_mut() {
            be.objs.insert("eyes".into(), eyes);
        }
    }

    pub fn shaft_eyes_gone(&mut self) {
        if let Some(e) = self.broken_elevator.as_mut().and_then(|b| b.objs.remove("eyes")) {
            self.world.remove(&e);
        }
    }

    /// What looking did: the lift peered into (its prompt gone), your nerve
    /// shaken, the edges of your sight going red
    pub fn shaft_peered(&mut self) {
        if let Some(be) = self.broken_elevator.as_mut() {
            be.peered = true;
        }
        self.player.sanity = (self.player.sanity - SHAFT_NERVE).clamp(0.0, SAN_MAX);
        // (the grain's update eases it back to what your nerve has it)
        self.aberration_opacity_cur = 0.85;
    }

    /// However it ends: nothing left looking out of the black
    pub fn shaft_after(&mut self) {
        self.shaft_eyes_gone();
    }
}
