//! Where the Reader lives (`entities::reader::place`): on real levels of
//! several runs, at most one a level, from depth 2, near paper lying about
//! (it lives where paper lies), far from the arrival and her camp; and on
//! some levels, not all.
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
fn the_reader_lives_where_paper_lies() {
    let mut placed = 0;
    let mut with_paper = 0;
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
            let piles: Vec<(i32, i32)> = g.papers.iter().map(|p| (g.tx(p.g.pos().x), g.tz(p.g.pos().z))).collect();
            if !piles.is_empty() {
                with_paper += 1;
            }
            let readers: Vec<usize> = (0..g.entities.len()).filter(|&i| g.entities[i].kind == "reader").collect();
            assert!(readers.len() <= 1, "seed {seed} L{level}: {} of them", readers.len());
            let Some(&ei) = readers.first() else { continue };
            placed += 1;
            let p = g.entities[ei].pos();
            let (x, z) = (g.tx(p.x), g.tz(p.z));
            let i = g.t_idx(x, z);
            assert!(!g.is_wall_tile(x, z), "seed {seed} L{level}: in a wall");
            let near = piles.iter().map(|&(px, pz)| (px - x).abs().max((pz - z).abs())).min();
            assert!(near.map(|d| (1..=6).contains(&d)).unwrap_or(false), "seed {seed} L{level}: not beside paper ({near:?} tiles)");
            let from_start = g.bfs(g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
            assert!(from_start[i] >= 18, "seed {seed} L{level}: {} tiles from the arrival", from_start[i]);
            if let Some(c) = g.story.camp.as_ref() {
                let from_camp = g.bfs(c.tile.0, c.tile.1);
                assert!(from_camp[i] < 0 || from_camp[i] >= 5, "seed {seed} L{level}: {} tiles from her camp", from_camp[i]);
            }
            println!(
                "seed {seed} L{level}: the Reader at ({x}, {z}), {} tiles from the arrival, paper {} tiles off",
                from_start[i],
                near.unwrap()
            );
        }
    }
    println!("the Reader lived on {placed} of {levels} levels ({with_paper} with paper lying about)");
    assert!(placed > 0, "it was never placed");
    assert!(placed < levels, "it was on every level");
}
