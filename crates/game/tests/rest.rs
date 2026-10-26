//! The HUD at rest (`ui/rest.rs`): the building's words in sentence case,
//! each part shown while it has something to say and gone after, and the
//! help that is offered at the moment it would help.
use game::prelude::*;
use game::ui::rest::{calm, prompt_parts, SHOW_FOR, THREAD_FOR};

const DT: f64 = 1.0 / 60.0;

fn rig() -> Game {
    let (mut g, _) = Game::test_level(4242, 21);
    g.init_player();
    g.init_hud();
    g.state = "playing".into();
    g.pointer_locked = true;
    g.hide_overlay();
    g.player.pos = Vec3::new(g.wx(2), 0.0, g.wz(2));
    g
}
fn step(g: &mut Game, n: usize) {
    for _ in 0..n {
        g.perf_t += DT;
        g.now_ms += DT * 1000.0;
        g.update_hud(DT);
    }
}

#[test]
fn the_building_does_not_shout() {
    for (shout, calmly) in [
        ("ALMOND WATER — SEALED, STILL COLD", "Almond water — sealed, still cold"),
        ("LEVEL 3 — DEEPER STILL", "Level 3 — deeper still"),
        ("HER PAGE — [TAB] TO READ", "Her page — Tab to read"),
        ("YOU ARE NOT TIRED YET", "You are not tired yet"),
        ("NO SPARE BATTERIES", "No spare batteries"),
        ("I'M NOT ALONE. THE HUM STOPPED.", "I'm not alone. The hum stopped."),
        ("M. WAS HERE", "M. was here"),
        ("OPEN THE EXIT DOOR", "Open the EXIT door"),
        ("THE LOBBY — Level 0", "The lobby — Level 0"),
        ("Far below you, somebody is crying.", "Far below you, somebody is crying."),
    ] {
        assert_eq!(calm(shout), calmly);
    }
    assert_eq!(prompt_parts("[E] TAKE HER PAGE", "E"), ("E".to_string(), "Take her page".to_string()));
    assert_eq!(prompt_parts("[E] LIGHT SWITCH  (bad wiring)", "Left click").1, "Light switch (bad wiring)");
}

#[test]
fn each_part_shows_while_it_has_something_to_say() {
    let mut g = rig();
    step(&mut g, 10);
    let r = &g.rest;
    assert!(
        r.stamina == 0.0 && r.battery == 0.0 && r.pockets == 0.0 && r.thread == 0.0 && r.sanity == 0.0,
        "something shows at rest: {r:?}"
    );
    // a run: the stamina shows, and goes a moment after it is full again
    g.player.stamina = 70.0;
    step(&mut g, 1);
    assert!(g.rest.stamina > 0.0);
    g.player.stamina = STAM_MAX;
    step(&mut g, (1.3 / DT) as usize);
    assert_eq!(g.rest.stamina, 0.0, "the stamina stayed after it was full");
    // a pickup: the pockets show, then go
    g.player.bottles += 1;
    step(&mut g, 1);
    assert!(g.rest.pockets > SHOW_FOR - 0.1);
    step(&mut g, ((SHOW_FOR + 1.1) / DT) as usize);
    assert_eq!(g.rest.pockets, 0.0);
    // a new line of hers stays a while
    g.thread = "Folded twice, so I know it's mine.".into();
    step(&mut g, 1);
    assert!(g.rest.thread > THREAD_FOR - 0.1);
    // the light on: its cell shows; off, it goes
    g.player.flashlight_on = true;
    step(&mut g, 1);
    assert!(g.rest.battery > 0.0);
    g.player.flashlight_on = false;
    step(&mut g, (1.6 / DT) as usize);
    assert_eq!(g.rest.battery, 0.0);
    // sanity going shows; full and steady it does not
    g.player.sanity -= 5.0;
    step(&mut g, 1);
    assert!(g.rest.sanity > 0.0);
}

#[test]
fn help_is_offered_when_it_would_help() {
    let mut g = rig();
    assert_eq!(g.pocket_hint(), None);
    g.player.bottles = 1;
    g.player.stamina = 20.0;
    assert_eq!(g.pocket_hint(), Some(("G".to_string(), "Drink almond water")));
    // with the key as rebound
    g.settings.rebind(game::session::input::Act::Drink, "KeyZ");
    assert_eq!(g.pocket_hint().unwrap().0, "Z");
    g.player.stamina = 90.0;
    g.player.batteries = 1;
    g.player.flashlight_on = true;
    g.player.battery = 12.0;
    assert_eq!(g.pocket_hint(), Some(("R".to_string(), "Change the battery")));
    g.player.batteries = 0;
    assert_eq!(g.pocket_hint(), None, "a battery offered that is not in the pockets");
}
