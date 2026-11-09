//! What still burns (`docs/LORE.md`): the place's own flames, the three
//! candles on the bench (the safe room) and the candles a haunted floor
//! leaves, are never on the Lobby nor on a floor with rules of its own (the
//! Showroom, the Borrowed House, the floors whose door opens on the
//! Stairwell or the Loop), as the things a hand can use; and a candle a hand
//! blows out stays out.
//!
//!   cargo test --release -p backrooms --test flames -- --nocapture
mod common;
use game::Game;

/// Level 4 to 12 of a few runs, each floor as `f` wants it
fn floors(mut f: impl FnMut(&mut Game, i32, Option<&str>) -> bool) {
    let mut g = common::new_game();
    for k in 0..40u32 {
        g.run_seed = k.wrapping_mul(2654435761) ^ 0x4C1D;
        for level in 4..=12 {
            let scp = game::level::scp::scp_for_level(g.run_seed, level);
            if f(&mut g, level, scp) {
                return;
            }
        }
    }
    panic!("not every kind of floor met");
}

#[test]
fn the_place_burns_nothing_on_the_lobby_or_a_floor_of_its_own() {
    // none on the Lobby (its emptiness is the point), nor on a floor with
    // rules of its own (the Showroom's and the Borrowed House's light is
    // their own); below the Lobby elsewhere the bench's candles burn, and
    // on a haunted floor its own
    let (mut lobby, mut store, mut house, mut door, mut haunted_lobby) = (0, 0, 0, 0, 0);
    let (mut elsewhere, mut benches, mut haunted, mut lit) = (0, 0, 0, 0);
    floors(|g, level, scp| {
        let count = match (level, scp) {
            (4, _) if lobby < 4 => &mut lobby,
            (_, Some("store")) if store < 2 => &mut store,
            (_, Some("house")) if house < 2 => &mut house,
            (_, Some("stairwell" | "loop")) if door < 3 => &mut door,
            (5..=12, None) if elsewhere < 6 || haunted < 2 => &mut elsewhere,
            _ => return false,
        };
        g.init_level(level);
        let is_haunted = g.weather_key() == "haunted";
        if level == 4 || scp.is_some() {
            let at = format!("seed {} level {level} ({scp:?}, {})", g.run_seed, g.weather_key());
            assert!(g.safe_room.is_none(), "{at}: candles on a bench");
            assert!(g.haunted_candles.is_empty(), "{at}: {} candles burn", g.haunted_candles.len());
            haunted_lobby += usize::from(is_haunted);
        } else {
            benches += usize::from(g.safe_room.is_some());
            if is_haunted {
                haunted += 1;
                lit += usize::from(!g.haunted_candles.is_empty() && g.haunted_candles.iter().all(|c| c.lit));
            }
        }
        *count += 1;
        lobby >= 4 && store >= 2 && house >= 2 && door >= 3 && elsewhere >= 6 && haunted >= 2
    });
    eprintln!(
        "lobby {lobby}, showroom {store}, house {house}, the Stairwell's or the Loop's {door} (haunted {haunted_lobby}); \
         elsewhere {elsewhere}: benches {benches}, haunted {haunted}, lit {lit}"
    );
    assert!(benches * 4 >= elsewhere * 3, "the bench's candles burn below the Lobby: {benches} of {elsewhere}");
    assert_eq!(lit, haunted, "a haunted floor's candles burn");
}

#[test]
fn a_candle_blown_out_stays_out() {
    floors(|g, level, scp| {
        if level < 5 || scp.is_some() {
            return false;
        }
        g.init_level(level);
        if g.weather_key() != "haunted" || g.haunted_candles.is_empty() {
            return false;
        }
        g.state = "playing".into();
        let blow: Vec<usize> = (0..g.interactables.len()).filter(|&i| g.interactables[i].prompt == "[E] BLOW OUT").collect();
        assert_eq!(blow.len(), g.haunted_candles.len(), "a verb a candle");
        let i = blow[0];
        let at = g.interactables[i].pos;
        // the one verb at a candle is to blow it out
        let here: Vec<&str> = g.interactables.iter().filter(|it| it.pos.distance_to(&at) < 0.01).map(|it| it.prompt.as_str()).collect();
        assert_eq!(here, ["[E] BLOW OUT"], "what a hand can do at a candle");
        assert!((g.interactables[i].enabled)(g));
        let mut f = std::mem::replace(&mut g.interactables[i].on_use, Box::new(|_| {}));
        f(g);
        g.interactables[i].on_use = f;
        g.player.battery = 100.0;
        for k in 0..600 {
            g.run_animators(1.0 / 60.0, k as f64 / 60.0);
        }
        assert!(!g.haunted_candles[0].lit, "blown out, it stays out");
        assert!(!(g.interactables[i].enabled)(g), "nothing blows it out twice");
        let again = g.interactables.iter().filter(|it| it.pos.distance_to(&at) < 0.01 && (it.enabled)(g)).count();
        assert_eq!(again, 0, "nothing you carry lights it again");
        true
    });
}
