//! Where the game plays its cutscenes (`docs/CUTSCENES.md`): the way in
//! from the title on New descent. (The shaft and the deaths join it.)
//! Each starts a cutscene of the library the way play meets it and does
//! what must happen around it; the cutscene itself is data.
use super::play::{Anchors, Pose};
use super::Cutscene;
use crate::base::game::Game;

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
        let flags_seed = self.parse_flag_params().contains_key("seed");
        let same = !flags_seed && self.title_place.is_some_and(|(seed, level)| seed == self.run_seed && level == 1 && self.level_num == 1);
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
}
