//! What stays with you (`docs/SECRETS.md` 5): the run keeps a list of what
//! you took (`Took`), and what it does to you is a pure function of that
//! list (`body_of`): small steps on numbers the game already has, each
//! system at most two steps either way, never what the creatures' fairness
//! rests on (no change lowers what the dark leaves you or raises what a
//! touch takes). Nothing on the screen says any of it.
use super::{PlaceKind, Thing};

/// One thing taken (or the hole's price), and where
#[derive(Clone, Debug, PartialEq)]
pub struct Took {
    /// `Thing::id`, or "hole" (what the hole took from you)
    pub what: String,
    pub level: i32,
    /// `PlaceKind::name` of where it was
    pub place: String,
}

/// What the things you took have done to you.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    /// Your size, −2 … 2 steps (a pale mushroom one down, a brown one up)
    pub size: i32,
    /// Eye height added, m (and crouched)
    pub eye: f64,
    pub crouch_eye: f64,
    /// How far your steps are heard, ×
    pub steps: f64,
    /// How fast you walk and run, ×
    pub pace: f64,
    /// How fast nerve refills, ×
    pub nerve: f64,
    /// How fast stamina refills, ×
    pub wind: f64,
    /// The stamina ceiling
    pub stam_max: f64,
    /// What almond water gives, ×
    pub drink: f64,
    /// How fast the dark takes nerve, ×, and how much it leaves you
    pub dark: f64,
    pub dark_floor: f64,
    /// What a creature's touch takes, ×
    pub touch: f64,
    /// How far a Smiler sees your lamp, ×
    pub lamp_seen: f64,
    /// How far the door's call carries, ×
    pub door: f64,
    /// How fast your look out of a window adds up, ×
    pub look: f64,
    /// How long a Hound takes to find you standing still, ×
    pub still: f64,
    /// How often a cry is answered with the way: × then +
    pub cry_mul: f64,
    pub cry_add: f64,
    /// Your hands shake (the beam trembles)
    pub shake: bool,
    /// No machine's coin return rings for you
    pub no_coin_return: bool,
}

impl Default for Body {
    fn default() -> Self {
        Body {
            size: 0,
            eye: 0.0,
            crouch_eye: 0.0,
            steps: 1.0,
            pace: 1.0,
            nerve: 1.0,
            wind: 1.0,
            stam_max: crate::prelude::STAM_MAX,
            drink: 1.0,
            dark: 1.0,
            dark_floor: crate::player::DARK_FLOOR,
            touch: 1.0,
            lamp_seen: 1.0,
            door: 1.0,
            look: 1.0,
            still: 1.0,
            cry_mul: 1.0,
            cry_add: 0.0,
            shake: false,
            no_coin_return: false,
        }
    }
}

/// One size step: the eye this much higher (crouched a little less), m
pub const SIZE_EYE: f64 = 0.07;
pub const SIZE_CROUCH_EYE: f64 = 0.05;

/// What the list of things taken does to you
pub fn body_of(took: &[Took]) -> Body {
    let n = |id: &str| took.iter().filter(|t| t.what == id).count() as i32;
    let steps2 = |k: i32| k.clamp(0, 2) as f64;
    let mut b = Body::default();
    // size: a pale mushroom one down, a brown one up, two at most either way
    b.size = (n(Thing::BrownMushrooms.id()) - n(Thing::PaleMushrooms.id())).clamp(-2, 2);
    let s = b.size as f64;
    b.eye = s * SIZE_EYE;
    b.crouch_eye = s * SIZE_CROUCH_EYE;
    b.pace = 1.0 + 0.04 * s;
    // (each system moves two steps at most, a step being the smallest a
    // thing moves it: what several things do together is held there)
    // your steps: size, and the jelly
    b.steps = ((1.0 + 0.12 * s) * (1.0 - 0.15 * steps2(n(Thing::Jelly.id())))).clamp(0.76, 1.24);
    // nerve: the sweets, the medicine (a third sweet from one dish is its price)
    b.nerve = (1.0 + 0.1 * steps2(n(Thing::Sweets.id())) + 0.2 * steps2(n(Thing::Medicine.id()))).min(1.2);
    b.wind = 1.0 - 0.1 * steps2(n(Thing::Medicine.id()));
    // stamina: the ration's ceiling, the hole's price
    b.stam_max =
        (crate::prelude::STAM_MAX + 12.0 * steps2(n(Thing::Ration.id())).min(1.0) - 15.0 * steps2(n("hole_took"))).clamp(85.0, 112.0);
    b.drink = 1.0 - 0.2 * steps2(n(Thing::Ration.id()));
    // the dark: the bag, the seeds
    let bag = n(Thing::Bag.id()) > 0;
    let seeds = steps2(n(Thing::Seeds.id()));
    b.dark = (if bag { 0.75 } else { 1.0 } * (1.0 - 0.2 * seeds)).max(0.6);
    b.dark_floor = crate::player::DARK_FLOOR + if bag { 5.0 } else { 0.0 };
    b.touch = 1.0 - 0.2 * steps2(n(Thing::Salt.id()));
    // a Smiler and your lamp: the seeds, a third sweet's shaking hands
    b.shake = n("third_sweet") > 0;
    b.lamp_seen = ((1.0 - 0.15 * seeds) * if b.shake { 1.2 } else { 1.0 }).clamp(0.7, 1.2);
    b.door = 1.0 - 0.2 * seeds;
    b.look = 1.0 + 0.25 * steps2(n(Thing::Envelope.id()));
    // a Hound and you standing still: the cap, the yellow water
    let cap = n(Thing::Cap.id()) > 0;
    b.still = if cap { 1.25 } else { 1.0 } / (1.0 + 0.2 * steps2(n(Thing::YellowWater.id())));
    b.cry_mul = if cap { 0.5 } else { 1.0 };
    let coin = n(Thing::Coin.id()) > 0;
    b.cry_add = if coin { 0.15 } else { 0.0 };
    b.no_coin_return = coin;
    b
}

/// Where a thing was, as the notebook says it
pub fn place_words(kind: PlaceKind, t: Thing) -> &'static str {
    match (kind, t) {
        (PlaceKind::Tile, _) => "over a ceiling tile",
        (PlaceKind::Vent, _) => "behind a vent",
        (PlaceKind::Drawer, _) => "taped under a drawer",
        (PlaceKind::Radiator, _) => "behind a radiator",
        (PlaceKind::Cell, _) => "in a cell",
        (PlaceKind::Hole, _) => "in a hole in a wall",
        (PlaceKind::Corner, Thing::PaleMushrooms | Thing::BrownMushrooms) => "in a wet corner",
        (PlaceKind::Corner, Thing::Seeds) => "at the end of a dead end",
        (PlaceKind::Corner, Thing::Envelope) => "on the floor of a corridor",
        (PlaceKind::Corner, _) => "in a corner",
        (PlaceKind::Table, _) => "on a table",
        (PlaceKind::Hook, _) => "on a hook",
    }
}

/// A thing as the notebook names it
pub fn thing_words(t: Thing) -> &'static str {
    match t {
        Thing::PaleMushrooms => "Pale mushrooms",
        Thing::BrownMushrooms => "Brown mushrooms",
        Thing::Sweets => "A sweet from a dish",
        Thing::Seeds => "Red seeds on a saucer",
        Thing::Bag => "A cloth bag on a cord, sewn shut",
        Thing::Salt => "A sachet of salt",
        Thing::Jelly => "A jar of blue jelly",
        Thing::Ration => "A ration of grey concentrate",
        Thing::Medicine => "A small amber medicine bottle",
        Thing::Cap => "A tweed cap",
        Thing::Coin => "A coin with a hole, on a string",
        Thing::YellowWater => "An almond water, yellowish, its seal broken",
        Thing::Envelope => "A red envelope",
        _ => "Something",
    }
}
