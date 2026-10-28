//! Where the one in the corner stands (`entities::corner::place`): on real
//! levels of several runs, at most one a level, in an inside corner, off the
//! main way (two tiles or more from the walk from the arrival to the door),
//! far from the arrival, the door and her camp; and on some levels at all.
use game::Game;

fn register_placers(r: &mut Vec<(&'static str, game::base::state::PlacerFn)>) {
    placers_p00::register(r);
    placers_p01::register(r);
    placers_p02::register(r);
    placers_p03::register(r);
    placers_p04::register(r);
    placers_p05::register(r);
    placers_p06::register(r);
    placers_p07::register(r);
    placers_p08::register(r);
    placers_p09::register(r);
    placers_p10::register(r);
    placers_p11::register(r);
    placers_p12::register(r);
    placers_p13::register(r);
    placers_p14::register(r);
    placers_p15::register(r);
    placers_p16::register(r);
    placers_p17::register(r);
    placers_p18::register(r);
    placers_p19::register(r);
    placers_p20::register(r);
    placers_p21::register(r);
    placers_p22::register(r);
    placers_p23::register(r);
    placers_p24::register(r);
    placers_p25::register(r);
    placers_p26::register(r);
    placers_p27::register(r);
    placers_p28::register(r);
    placers_p29::register(r);
    placers_p30::register(r);
    placers_p31::register(r);
    placers_p32::register(r);
}

#[test]
fn the_one_in_the_corner_stands_off_the_way() {
    let mut placed = 0;
    let mut levels = 0;
    for seed in [4242u32, 77, 9001, 1999, 31337] {
        for level in 5..=12 {
            let mut g = Game::new_bare();
            register_placers(&mut g.placers);
            g.launch_flags = format!("seed={seed}&level={level}&nosound");
            g.boot();
            g.start_new_game();
            if g.building.is_some() || g.level_scp.is_some() {
                continue;
            }
            levels += 1;
            let corners: Vec<usize> = (0..g.entities.len()).filter(|&i| g.entities[i].kind == "corner").collect();
            assert!(corners.len() <= 1, "seed {seed} L{level}: {} of them", corners.len());
            let Some(&ei) = corners.first() else { continue };
            placed += 1;
            let p = g.entities[ei].pos();
            let (x, z) = (g.tx(p.x), g.tz(p.z));
            let i = g.t_idx(x, z);
            let e = g.exit_data.as_ref().unwrap().pos;
            let from_exit = g.bfs(g.tx(e.x), g.tz(e.z));
            let from_start = g.bfs(g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
            assert!(from_start[i] >= 8, "seed {seed} L{level}: {} tiles from the arrival", from_start[i]);
            assert!(from_exit[i] >= 5, "seed {seed} L{level}: {} tiles from the door", from_exit[i]);
            if let Some(c) = g.story.camp.as_ref() {
                let from_camp = g.bfs(c.tile.0, c.tile.1);
                assert!(from_camp[i] < 0 || from_camp[i] >= 5, "seed {seed} L{level}: {} tiles from her camp", from_camp[i]);
            }
            // off the walk from the arrival to the door
            let (mut wx, mut wz) = (g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
            loop {
                assert!((wx - x).abs().max((wz - z).abs()) > 2, "seed {seed} L{level}: on the way at ({wx}, {wz})");
                let here = from_exit[g.t_idx(wx, wz)];
                if here <= 0 {
                    break;
                }
                let next = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .into_iter()
                    .map(|(dx, dz)| (wx + dx, wz + dz))
                    .find(|&(nx, nz)| g.in_grid(nx, nz) && from_exit[g.t_idx(nx, nz)] == here - 1);
                let Some((nx, nz)) = next else { break };
                wx = nx;
                wz = nz;
            }
            // in an inside corner, its face to the angle of the walls
            let yaw = g.entities[ei].yaw;
            let (cx, cz) = (yaw.sin().signum() as i32, yaw.cos().signum() as i32);
            assert!(g.is_wall_tile(x + cx, z) && g.is_wall_tile(x, z + cz), "seed {seed} L{level}: not in a corner");
        }
    }
    println!("the one in the corner stood on {placed} of {levels} levels");
    assert!(placed > 0, "it was never placed");
    assert!(placed < levels, "it was on every level");
}
