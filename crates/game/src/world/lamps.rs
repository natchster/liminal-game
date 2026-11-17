//! The rooms' lamps by kind (`docs/plan/FACILITY.md` F3; the real lamps:
//! `docs/research/facility.md`, *Light*): which zones a kind lights, the
//! colour of its light, its fitting (`assets/src/blender/props/game/lamps.py`)
//! and how it fails.
//!
//! Fluorescent tubes stay the rooms' light. A tall hall hangs mercury
//! high-bays as a real plant's did, phosphor-coated and so white, never
//! clear mercury's green (the green is the exit door's and the Loop's); a
//! tunnel has a vapour-tight jar fixture on its crown, its filament a warm
//! white, never the sodium orange that is the window's (`LORE.md`, *The
//! windows*). A colour is its lamp's alone: it is the pooled spot's and the
//! glass's, and dies with them.
//!
//! Every kind fails on the tubes' own draws (`update_lights`): the frame
//! stream moves the same whatever lamps a floor has. Only what the
//! brightness does with them differs: a tube stutters; a high-bay's arc
//! drops out, and it stays dark until it restrikes and warms up again; a
//! filament dims and comes back, never blinking.

use crate::base::records::Lamp;

/// The kind of lamp that lights a zone of this archetype
/// (`level::archgen::ARCHETYPES`)
pub fn lamp_of(arch: &str) -> Lamp {
    match arch {
        // the Habitable Zone's halls, 5.4-7.6 m: hung high-bays
        "hall" => Lamp::HighBay,
        // the Utility Halls: a jar fixture on the crown
        "tunnels" => Lamp::Bulkhead,
        _ => Lamp::Tube,
    }
}

/// The colour of a kind's light (its pooled spot's, in linear RGB)
pub fn light_rgb(kind: Lamp) -> (f64, f64, f64) {
    match kind {
        Lamp::Tube => (1.0, 0.933, 0.769),
        // phosphor-coated mercury: a cool white, a little blue
        Lamp::HighBay => (0.90, 0.95, 1.0),
        // a filament: warm white
        Lamp::Bulkhead => (1.0, 0.85, 0.62),
    }
}

/// The colour of a kind's glass, lit (`Lamp::Tube` keeps its texture)
pub fn glass_hex(kind: Lamp) -> u32 {
    match kind {
        Lamp::Tube => 0xffffff,
        Lamp::HighBay => 0xeef4ff,
        Lamp::Bulkhead => 0xfff0d2,
    }
}

/// A kind's fitting and its glowing glass, as modelled (`lamps.py`); a tube
/// is the panel itself (`Game::panel_geo`)
pub fn parts(kind: Lamp) -> Option<(&'static str, &'static str)> {
    match kind {
        Lamp::Tube => None,
        Lamp::HighBay => Some(("Lamp__highBay", "Lamp__highBayBulb")),
        Lamp::Bulkhead => Some(("Lamp__bulkhead", "Lamp__bulkheadGlobe")),
    }
}

/// How far below its hook a high-bay's light comes from (its bulb's middle)
pub const HIGH_BAY_DROP: f64 = 0.48;
/// How far below the ceiling a bulkhead's light comes from (its jar's)
pub const BULKHEAD_DROP: f64 = 0.15;
/// No caged lamp hangs this near a door the layout made (m): a lamp in a
/// cage over a door is a cell's (`LORE.md`, *The Building*)
pub const CAGE_CLEAR_OF_DOORS: f64 = 1.6;

/// A high-bay's restrike, from its own seed (s): its arc out, then warming
pub fn restrike_secs(seed: f64) -> f64 {
    RESTRIKE_WARM + 14.0 + seed % 18.0
}
/// The last seconds of a restrike, in which the arc warms up from dim
pub const RESTRIKE_WARM: f64 = 12.0;

/// A kind's brightness this frame: `flicker` the tubes' stutter running
/// (its value `drawn`, drawn whatever the kind), `restrike` a high-bay's
/// seconds still to go, `prev` its brightness last frame
pub fn brightness(kind: Lamp, flicker: bool, drawn: f64, restrike: f64, prev: f64, dt: f64) -> f64 {
    match kind {
        Lamp::Tube => {
            if flicker {
                drawn
            } else {
                1.0
            }
        }
        Lamp::HighBay => {
            if restrike <= 0.0 {
                1.0
            } else if restrike > RESTRIKE_WARM {
                0.0
            } else {
                // a mercury arc warms up dim and bluish, then comes to full
                let t = 1.0 - restrike / RESTRIKE_WARM;
                0.08 + 0.92 * t * t
            }
        }
        Lamp::Bulkhead => {
            // a filament's heat holds it: it dims toward what the stutter
            // drew, softened, and comes back as smoothly
            let want = if flicker { 0.45 + 0.55 * drawn } else { 1.0 };
            prev + (want - prev) * (dt * 9.0).min(1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_high_bay_goes_out_then_warms_up() {
        let r = restrike_secs(37.0);
        assert!(r > RESTRIKE_WARM);
        assert_eq!(brightness(Lamp::HighBay, true, 1.0, r, 1.0, 0.016), 0.0, "out while its arc cools");
        let warming = brightness(Lamp::HighBay, false, 1.0, RESTRIKE_WARM / 2.0, 0.0, 0.016);
        assert!(warming > 0.08 && warming < 1.0, "dim while it warms: {warming}");
        assert_eq!(brightness(Lamp::HighBay, false, 1.0, 0.0, 0.2, 0.016), 1.0);
    }

    #[test]
    fn a_filament_never_blinks() {
        // a stutter's darkest draw dims it by little in one frame
        let b = brightness(Lamp::Bulkhead, true, 0.05, 0.0, 1.0, 0.016);
        assert!(b > 0.9, "{b}");
        // held there, it settles above half
        let mut b = 1.0;
        for _ in 0..300 {
            b = brightness(Lamp::Bulkhead, true, 0.05, 0.0, b, 0.016);
        }
        assert!(b > 0.45 && b < 0.5, "{b}");
    }

    #[test]
    fn no_lamp_takes_the_canons_colours() {
        // green (the door, the Loop), orange (the window), red (the broken
        // lift): no kind's green or red leads its other channels by much
        for k in [Lamp::Tube, Lamp::HighBay, Lamp::Bulkhead] {
            let (r, g, b) = light_rgb(k);
            assert!(g <= r.max(b) + 0.06, "{k:?} green");
            assert!(b >= 0.6 * r, "{k:?} too orange or red: {r} {g} {b}");
        }
    }
}
