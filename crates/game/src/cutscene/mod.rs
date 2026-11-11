//! Cutscenes (`docs/CUTSCENES.md`): a moment that borrows your eyes and
//! gives them back. A cutscene is data (`model`), written as TOML in
//! `assets/src/cutscenes/` and compiled by `cargo xtask cutscene write`
//! into the bundle embedded here (`assets/cutscenes.json`); its camera
//! moves on the curves of `curve`, and `play` plays it (one at a time, your
//! hands taken, the camera given back); `cues` holds the sounds and events
//! it may name.
//!
//! Owns: the library (`library`, `get`) and the rules a cutscene must keep
//! whatever the game around it (`Cutscene::shape_problems`). Nothing here
//! draws from a random stream or reads a clock.
pub mod cues;
pub mod curve;
pub mod model;
pub mod play;
pub mod uses;

pub use model::{Anchor, Bars, Cutscene, Event, Fade, Key, Look, Say, Skip, Sound, Then, World};

static BUNDLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/cutscenes.json"));

/// Every cutscene the game has, parsed once (a bundle that does not parse
/// is caught by `the_bundle_parses_and_keeps_its_shape` before it ships)
pub fn library() -> &'static [Cutscene] {
    static LIBRARY: std::sync::OnceLock<Vec<Cutscene>> = std::sync::OnceLock::new();
    LIBRARY.get_or_init(|| model::parse_bundle(BUNDLE).unwrap_or_else(|e| panic!("assets/cutscenes.json: {e}")))
}

/// The cutscene called `id`
pub fn get(id: &str) -> Option<&'static Cutscene> {
    library().iter().find(|c| c.id == id)
}

/// What is wrong with a bundle's text, against everything the game checks
/// before it plays one (`backrooms --cutscene-check`, run by
/// `cargo xtask cutscene check` and `write`)
pub fn bundle_problems(text: &str) -> Vec<String> {
    match model::parse_bundle(text) {
        Err(e) => vec![e],
        Ok(list) => list
            .iter()
            .flat_map(|c| c.shape_problems().into_iter().chain(cues::lint(c)).map(move |p| format!("cutscene {}: {p}", c.id)))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    static SAMPLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/cutscene/sample.json"));

    #[test]
    fn the_bundle_parses_and_keeps_its_shape() {
        assert!(bundle_problems(BUNDLE).is_empty(), "{:#?}", bundle_problems(BUNDLE));
        let _ = library();
    }

    /// The sample uses every field; `xtask`'s format compiled it
    /// (`the_sample_compiles_to_the_games_fixture`), so the two agree
    #[test]
    fn the_sample_has_every_field_and_parses() {
        let list = model::parse_bundle(SAMPLE).unwrap();
        let c = &list[0];
        assert_eq!((c.world, c.then, c.skip), (World::Live, Then::Back, Skip::Hold));
        assert!(c.camera.iter().any(|k| matches!(k.look, Look::Point(_))));
        assert!(c.camera.iter().any(|k| matches!(k.look, Look::At(Anchor::Killer))));
        assert!(c.camera.iter().any(|k| k.cut) && c.camera.iter().any(|k| k.jolt) && c.camera.iter().any(|k| k.fov != 0.0));
        assert!(!c.fade.is_empty() && !c.sound.is_empty() && !c.event.is_empty() && !c.say.is_empty() && !c.finally.is_empty());
        assert!(c.camera.iter().any(|k| k.walk > 0.0));
        assert!(c.shape_problems().is_empty(), "{:#?}", c.shape_problems());
    }

    fn sample() -> Value {
        serde_json::from_str::<Value>(SAMPLE).unwrap()["cutscenes"][0].clone()
    }

    #[test]
    fn a_field_unknown_missing_or_of_the_wrong_kind_is_refused() {
        let mut v = sample();
        v["camera"][0]["colour"] = "red".into();
        assert!(Cutscene::from_json(&v).unwrap_err().contains("unknown field `colour`"));
        let mut v = sample();
        v.as_object_mut().unwrap().remove("blend_in");
        assert!(Cutscene::from_json(&v).unwrap_err().contains("no `blend_in`"));
        let mut v = sample();
        v["length"] = "long".into();
        assert!(Cutscene::from_json(&v).unwrap_err().contains("`length` is not a number"));
        let mut v = sample();
        v["camera"][1]["anchor"] = "floor".into();
        assert!(Cutscene::from_json(&v).unwrap_err().contains("not an anchor"));
    }

    /// The rules of a cutscene's shape, one broken at a time
    #[test]
    fn every_broken_rule_is_named() {
        let base = Cutscene::from_json(&sample()).unwrap();
        let cases: Vec<(&str, Box<dyn Fn(&mut Cutscene)>)> = vec![
            ("lasts more than 0", Box::new(|c| c.length = 0.0)),
            ("must fit in its length", Box::new(|c| c.blend_out = c.length)),
            ("blend_out", Box::new(|c| c.blend_out = 0.1)),
            ("keys go forward in time", Box::new(|c| c.camera[1].t = c.camera[0].t)),
            ("no stage", Box::new(|c| c.stages.clear())),
            ("breaks_off is for a live world", Box::new(|c| c.world = World::Held)),
            (
                "the first key must be the camera as it is",
                Box::new(|c| {
                    c.blend_in = 0.0;
                    c.camera[0].pos = [0.0, 0.1, 0.0];
                }),
            ),
            ("outside its length", Box::new(|c| c.event[0].t = c.length + 1.0)),
            ("fov", Box::new(|c| c.camera[1].fov = 40.0)),
            ("only for 3 s at most", Box::new(|c| c.skip = Skip::Never)),
            ("a jolt is at most", Box::new(|c| c.camera[3].t = 3.5)),
            ("a point to look at, not a place to stand", Box::new(|c| c.camera[1].anchor = Anchor::Killer)),
            ("bars", Box::new(|c| c.bars.out_at = c.length + 1.0)),
        ];
        for (says, break_it) in cases {
            let mut c = base.clone();
            break_it(&mut c);
            let p = c.shape_problems();
            assert!(p.iter().any(|x| x.contains(says)), "{says}: {p:#?}");
        }
    }

    #[test]
    fn two_cutscenes_of_one_name_are_refused() {
        let v = sample();
        let text = serde_json::json!({ "about": "", "cutscenes": [v.clone(), v] }).to_string();
        assert!(model::parse_bundle(&text).unwrap_err().contains("two cutscenes are called"));
    }
}
