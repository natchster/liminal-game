//! Zone purposes and the regions cut from them (`world::purpose`,
//! `build_regions`), on levels built with every themed placer replaced by a
//! probe that checks what floor it was given:
//!
//! * zones of one area share one purpose (each draws its own fittings), and
//!   only zones of one archetype with no wall between them form an area;
//! * a purpose's themes are ones its archetype welcomes (the second one
//!   compatible with the first); the Showroom has none, and the first room
//!   of the first level is left bare;
//! * a themed placer only ever sees floor whose purpose welcomes its theme;
//! * a region never spans two areas nor cuts through a room, an area wears
//!   one or two surfaces, and the spawn region is neither dark nor mossy;
//! * all of it is a pure function of the seed.
use game::base::rng::with_seed;
use game::level::archgen::Layout;
use game::visuals::lighting::{RF_DARK, RF_MOSS};
use game::world::build::{placer_theme_of, theme_allow};
use game::world::purpose::{walled_apart, ZonePurpose};
use game::world::tables::{PLACER_ALWAYS, THEMED_ORDER, THEME_COMPAT};
use game::Game;
use std::cell::RefCell;

/// 21 palettes × 4 grime levels, then 2 moss variants (as the textures have them)
fn set_wallpaper_meta(g: &mut Game) {
    let mut k = 0;
    for p in 0..21 {
        for gl in 0..4 {
            let v = &mut g.tex.wallpaper_variants[k];
            v.grime_lvl = gl;
            v.moss = false;
            v.palette_idx = p;
            k += 1;
        }
    }
    for (j, p) in [5usize, 17].iter().enumerate() {
        let v = &mut g.tex.wallpaper_variants[84 + j];
        v.grime_lvl = 3;
        v.moss = true;
        v.palette_idx = *p;
    }
}

thread_local! { static SEEN: RefCell<Vec<String>> = const { RefCell::new(vec![]) }; }

/// The probe placer: every open tile it is shown must be in a zone whose
/// purpose welcomes the theme of the placer it stands in for (the last
/// trace mark)
fn probe(g: &mut Game, _dist: &[i32]) {
    let name = g.rand_trace.marks.last().map(|(l, _)| l.clone()).unwrap_or_default();
    SEEN.with(|s| s.borrow_mut().push(name.clone()));
    if PLACER_ALWAYS.contains(&name.as_str()) {
        return;
    }
    let theme = placer_theme_of(&name);
    let l = g.layout.as_ref().unwrap();
    for (i, &t) in g.tiles.iter().enumerate() {
        if t == 0 {
            let zi = l.zone_of[i] as usize;
            assert!(g.zone_purposes[zi].allows(theme), "{name} ({theme}) was shown tile {i} in zone {zi} ({:?})", g.zone_purposes[zi]);
        }
    }
}

fn build(n: i32, seed: u32, scp: Option<&str>) -> Game {
    let mut g = Game::new_bare();
    g.test_stub_core_placers = true;
    set_wallpaper_meta(&mut g);
    for name in THEMED_ORDER {
        g.placers.push((name, probe));
    }
    g.init_world();
    g.level_num = n;
    g.level_scp = scp.map(|s| s.to_string());
    g.rand_trace.on = true;
    with_seed(seed, || {
        g.gen_layout_level(n, (31 + n * 4).min(53));
        g.build_world();
    });
    g
}

const CASES: [(i32, u32, Option<&str>); 8] = [
    (1, 4097, None),
    (2, 77, None),
    (3, 9001, None),
    (5, 4242, None),
    (7, 31337, None),
    (11, 2026, None),
    (4, 1234, Some("store")),
    (8, 99, Some("house")),
];

fn check_purposes(tag: &str, g: &Game, l: &Layout) {
    let ps: &[ZonePurpose] = &g.zone_purposes;
    assert_eq!(ps.len(), l.zones.len(), "{tag}: a purpose per zone");
    for (zi, (z, p)) in l.zones.iter().zip(ps).enumerate() {
        let root = &l.zones[p.area];
        assert!(p.area <= zi && root.arch == z.arch, "{tag}: zone {zi} in area {} of another kind", p.area);
        let first = &ps[p.area];
        assert_eq!(
            (p.label, p.primary, p.secondary),
            (first.label, first.primary, first.secondary),
            "{tag}: zone {zi} differs from its area"
        );
        if let Some(t) = p.primary {
            assert!(theme_allow(z.arch, t) > 0.0, "{tag}: zone {zi} ({}) given {t}", z.arch);
        }
        if let Some(t) = p.secondary {
            let first = p.primary.expect("a second theme comes with a first");
            let compat = THEME_COMPAT.iter().find(|(k, _)| *k == first).map(|(_, l)| l.iter().any(|(c, _)| *c == t)).unwrap_or(false);
            assert!(compat, "{tag}: zone {zi}: {t} does not go with {first}");
        }
        if z.arch == "store" {
            assert!(p.primary.is_none(), "{tag}: the Showroom furnishes itself");
        }
    }
    // zones that touch with no wall between them, of one kind, are one area
    for a in &l.zones {
        for b in &l.zones {
            if a.id < b.id && a.arch == b.arch && a.touches(b) && !walled_apart(l, &g.tiles, a, b) {
                assert_eq!(ps[a.id].area, ps[b.id].area, "{tag}: zones {} and {} open into each other", a.id, b.id);
            }
        }
    }
    if g.level_num == 1 {
        let spawn = l.zone_of[l.idx(1, 1)] as usize;
        assert!(ps[spawn].primary.is_none(), "{tag}: the first room of the first level is bare");
    }
}

fn check_regions(tag: &str, g: &Game, l: &Layout) {
    let ps = &g.zone_purposes;
    let mut area_of_region: Vec<Option<usize>> = vec![None; g.region_info.len()];
    let mut regions_of_area: std::collections::BTreeMap<usize, std::collections::BTreeSet<u8>> = Default::default();
    for (i, &r) in g.region_map.iter().enumerate() {
        let area = ps[l.zone_of[i] as usize].area;
        let seen = area_of_region[r as usize].get_or_insert(area);
        assert_eq!(*seen, area, "{tag}: region {r} spans areas {seen} and {area}");
        regions_of_area.entry(area).or_default().insert(r);
    }
    // an area wears one surface, or two when a big lone zone is split
    for (a, rs) in &regions_of_area {
        let looks: std::collections::BTreeSet<(usize, i32)> =
            rs.iter().map(|&r| &g.region_info[r as usize]).filter(|i| i.flags & RF_MOSS == 0).map(|i| (i.wall, i.grime)).collect();
        assert!(looks.len() <= 2, "{tag}: area {a} wears {} surfaces", looks.len());
    }
    // the lights never change inside a room
    for r in &l.rooms {
        let first = g.region_map[g.t_idx(r.x0, r.z0)];
        for z in r.z0..=r.z1 {
            for x in r.x0..=r.x1 {
                assert_eq!(g.region_map[g.t_idx(x, z)], first, "{tag}: room {r:?} spans regions");
            }
        }
    }
    let spawn = &g.region_info[g.region_map[g.t_idx(1, 1)] as usize];
    assert!(spawn.flags & (RF_DARK | RF_MOSS) == 0, "{tag}: spawn region {spawn:?}");
}

#[test]
fn purposes_regions_and_themed_floor() {
    for (n, seed, scp) in CASES {
        let tag = format!("level {n} seed {seed} {scp:?}");
        SEEN.with(|s| s.borrow_mut().clear());
        let g = build(n, seed, scp);
        let l = g.layout.as_ref().unwrap();
        check_purposes(&tag, &g, l);
        check_regions(&tag, &g, l);
        assert!(g.tiles.iter().all(|&t| t <= 1), "{tag}: masked tiles left behind");
        let ran = SEEN.with(|s| s.borrow().len());
        assert_eq!(ran as u32, g.theme_stats_ran, "{tag}: every themed run was checked");
        let areas = g.zone_purposes.iter().map(|p| p.area).collect::<std::collections::BTreeSet<_>>().len();
        let bare = g.zone_purposes.iter().filter(|p| p.primary.is_none()).count();
        eprintln!(
            "{tag}: {} zones, {areas} areas, {bare} bare zones, {} regions, {ran} placer runs checked",
            l.zones.len(),
            g.region_info.len()
        );
    }
}

#[test]
fn purposes_and_regions_are_a_function_of_the_seed() {
    for (n, seed, scp) in CASES {
        let a = build(n, seed, scp);
        let b = build(n, seed, scp);
        assert_eq!(a.zone_purposes, b.zone_purposes, "level {n} seed {seed}: purposes");
        assert_eq!(a.region_map, b.region_map, "level {n} seed {seed}: region map");
        assert_eq!(format!("{:?}", a.region_info), format!("{:?}", b.region_info), "level {n} seed {seed}: regions");
        let c = build(n, seed + 1, scp);
        assert!(a.zone_purposes != c.zone_purposes || a.region_map != c.region_map, "level {n}: another seed, another level");
    }
}
