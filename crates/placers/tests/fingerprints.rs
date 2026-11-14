//! Every placer's output, fingerprinted. Each placer is run on the smoke
//! test's small levels (`Game::test_level`) until it has placed something
//! four times, and what it added is hashed: every node it made (its name, kind,
//! place, flags and user data; its geometry; its materials and the pixels of
//! their textures), its interactables, and the level's solids and tiles; then
//! again once its animators have run and what it made usable has been used.
//! No id is hashed (ids count what was made before), so a print is the
//! placer's alone: its sound anchors, its materials' shader uniforms and how
//! many draws it took from the level's stream are in it too. Each placement
//! starts as a level does (`kit::level_begins`) and is printed again on a
//! fresh thread, which must give the same (a level comes out the same
//! whatever was built before it), and no placement may draw from the
//! per-frame stream.
//!
//! The prints are recorded in `tests/fixtures/fingerprints.tsv`. Moving a
//! placer's code (another module, crate or theme), sharing a helper, or
//! tidying one must leave every print as it was; a change that is meant to
//! change what a placer makes is re-recorded, and says so:
//!
//!   cargo test --release -p placers --test fingerprints
//!   REGEN=1 cargo test --release -p placers --test fingerprints
use engine::material::Patch;
use engine::prelude::*;
use engine::scene::Kind;
use game::base::state::PlacerFn;
use game::Game;
use std::collections::HashMap;
use std::fmt::Write as _;

/// FNV-1a, 64 bits: stable across toolchains, unlike the std hasher.
struct Fnv(u64);
impl Fnv {
    fn new() -> Fnv {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    fn bytes(&mut self, b: &[u8]) {
        for &x in b {
            self.0 ^= u64::from(x);
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
        }
    }
    fn u64(&mut self, v: u64) {
        self.bytes(&v.to_le_bytes());
    }
    fn f64(&mut self, v: f64) {
        self.u64(v.to_bits());
    }
    fn f32s(&mut self, v: &[f32]) {
        self.u64(v.len() as u64);
        for x in v {
            self.bytes(&x.to_bits().to_le_bytes());
        }
    }
    fn str(&mut self, s: &str) {
        self.u64(s.len() as u64);
        self.bytes(s.as_bytes());
    }
}

/// A texture's print (its pixels and how it is sampled), each image hashed
/// once a pass
fn texture(h: &mut Fnv, t: &Option<Tex>, seen: &mut HashMap<usize, u64>) {
    let Some(t) = t else {
        h.u64(0);
        return;
    };
    let t = t.t();
    let key = std::rc::Rc::as_ptr(&t.image) as usize;
    let img = *seen.entry(key).or_insert_with(|| {
        let i = t.image.borrow();
        let mut ih = Fnv::new();
        ih.u64(u64::from(i.width));
        ih.u64(u64::from(i.height));
        ih.str(&format!("{:?}", i.format));
        ih.bytes(&i.data);
        ih.0
    });
    h.u64(img);
    h.str(&format!("{:?}{:?}{:?}{:?}{:?}", t.wrap_s, t.wrap_t, t.color_space, t.mag_filter, t.min_filter));
    for v in [t.repeat.x, t.repeat.y, t.offset.x, t.offset.y, t.center.x, t.center.y, t.rotation] {
        h.f64(v);
    }
}

/// A material's shader patch, its textures by their pixels (a texture's id
/// counts what was made before it, so it is never hashed)
fn patch(h: &mut Fnv, p: &Patch, seen: &mut HashMap<usize, u64>) {
    match p {
        Patch::ArchUV { scale_x, scale_y, wallpaper, detail, plane_shift } => {
            h.str("arch");
            for v in [*scale_x, *scale_y, *plane_shift] {
                h.f64(v);
            }
            h.u64(u64::from(*wallpaper));
            if let Some(d) = detail {
                texture(h, &Some(d.map.clone()), seen);
                for v in [d.metres, d.tint, d.relief] {
                    h.f64(v);
                }
            }
        }
        Patch::SkinDetail { map, scale, strength, shine, rest_space } => {
            h.str("skin");
            texture(h, &Some(map.clone()), seen);
            for v in [*scale, *strength, *shine] {
                h.f64(v);
            }
            h.u64(u64::from(*rest_space));
        }
        Patch::WatcherSheet { map, scale, strength } => {
            h.str("sheet");
            texture(h, &Some(map.clone()), seen);
            h.f64(*scale);
            h.f64(*strength);
        }
        // the rest hold no texture: their Debug has no id
        other => h.str(&format!("{other:?}")),
    }
}

fn material(h: &mut Fnv, m: &Mat, seen: &mut HashMap<usize, u64>) {
    let m = m.m();
    h.str(&format!("{:?}|{}|{:?}|{:?}", m.kind, m.name, m.side, m.blending));
    patch(h, &m.patch, seen);
    for c in [&m.color, &m.emissive, &m.specular] {
        h.f64(c.r);
        h.f64(c.g);
        h.f64(c.b);
    }
    for v in [
        m.emissive_intensity,
        m.shininess,
        m.bump_scale,
        m.opacity,
        m.alpha_test,
        m.polygon_offset_factor,
        m.polygon_offset_units,
        m.size,
        m.rotation,
        m.line_width,
        m.wear,
        m.wear_use,
    ] {
        h.f64(v);
    }
    for b in [
        m.transparent,
        m.depth_write,
        m.depth_test,
        m.color_write,
        m.vertex_colors,
        m.fog,
        m.tone_mapped,
        m.flat_shading,
        m.wireframe,
        m.polygon_offset,
        m.visible,
        m.size_attenuation,
        m.look_settled,
    ] {
        h.u64(u64::from(b));
    }
    for t in [&m.map, &m.alpha_map, &m.bump_map, &m.emissive_map] {
        texture(h, t, seen);
    }
    for v in m.patch_uniforms.borrow().f {
        h.bytes(&v.to_bits().to_le_bytes());
    }
}

fn geometry(h: &mut Fnv, g: &Geo) {
    let g = g.g();
    for a in [&g.position, &g.normal, &g.uv, &g.color, &g.skin_index, &g.skin_weight] {
        h.f32s(a);
    }
    for m in g.morph_position.iter().chain(&g.morph_normal) {
        h.f32s(m);
    }
    match &g.index {
        Some(ix) => {
            h.u64(ix.len() as u64);
            for i in ix {
                h.bytes(&i.to_le_bytes());
            }
        }
        None => h.u64(u64::MAX),
    }
    for gr in &g.groups {
        h.u64((u64::from(gr.start) << 32) | u64::from(gr.count));
        h.u64(u64::from(gr.material_index));
    }
    h.u64((u64::from(g.draw_start) << 32) | u64::from(g.draw_count));
}

fn node(h: &mut Fnv, o: &Obj, seen: &mut HashMap<usize, u64>) {
    let n = o.0.borrow();
    h.str(&n.name);
    for v in n.matrix_world.e {
        h.f64(v);
    }
    h.u64(u64::from(n.visible) | (u64::from(n.cast_shadow) << 1) | (u64::from(n.receive_shadow) << 2) | (u64::from(n.frustum_culled) << 3));
    h.u64(n.render_order as u64);
    h.u64(u64::from(n.layers));
    for v in &n.morph_influences {
        h.f64(*v);
    }
    let u = &n.user;
    h.str(&format!(
        "{:?}|{}{}{}{}{}{}{}{}|{:?}",
        u.placer, u.static_prop, u.floats, u.gameplay, u.no_instance, u.instanced, u.hide, u.arch, u.prop_part, u.panel
    ));
    for v in [u.w, u.phase, u.base_x, u.base_y, u.base_z] {
        h.f64(v);
    }
    // the maps in key order (a HashMap's own order differs from run to run)
    let mut user: Vec<String> = u.f.iter().map(|(k, v)| format!("{k}={}", v.to_bits())).collect();
    user.extend(u.s.iter().map(|(k, v)| format!("{k}:{v}")));
    user.extend(u.o.iter().map(|(k, o)| format!("{k}->{}", o.0.borrow().name)));
    user.sort();
    h.str(&user.join(";"));
    match &n.kind {
        Kind::Mesh { geo, mats } | Kind::Instanced { geo, mats, .. } | Kind::Skinned { geo, mats, .. } => {
            h.str("mesh");
            geometry(h, geo);
            for m in mats {
                material(h, m, seen);
            }
            if let Kind::Instanced { inst, .. } = &n.kind {
                h.str(&format!("{inst:?}"));
            }
        }
        Kind::Points { geo, mat } | Kind::Line { geo, mat, .. } => {
            h.str("line");
            geometry(h, geo);
            material(h, mat, seen);
        }
        Kind::Sprite { mat, center } => {
            h.str("sprite");
            material(h, mat, seen);
            h.f64(center.x);
            h.f64(center.y);
        }
        other => h.str(&format!("{other:?}")),
    }
    h.u64(n.children.len() as u64);
    for c in &n.children {
        node(h, c, seen);
    }
}

/// What a placer added to `g` since `from` (the world's child count and the
/// interactables' and animators' counts before it ran)
fn print(g: &mut Game, from: (usize, usize, usize)) -> u64 {
    g.world.update_matrix_world(true);
    let mut h = Fnv::new();
    let mut seen = HashMap::new();
    let world = g.world.0.borrow();
    h.u64(world.children.len() as u64);
    for c in world.children.iter().skip(from.0) {
        node(&mut h, c, &mut seen);
    }
    drop(world);
    h.u64(g.interactables.len() as u64);
    for i in g.interactables.iter().skip(from.1) {
        h.f64(i.pos.x);
        h.f64(i.pos.y);
        h.f64(i.pos.z);
        h.f64(i.r);
        h.str(&i.prompt);
    }
    h.u64((g.animators.len() - from.2) as u64);
    // where its sounds come from (the places it adds a hum, a tick, a whirr),
    // and the gameplay it sets up (the switch and the far lamp it lights)
    h.str(&format!("{:?}", g.pp));
    if let Some(r) = &g.gameplay_r1_state {
        for v in [r.switch_x, r.switch_z, r.target_x, r.target_z] {
            h.f64(v);
        }
        h.str(&format!("{:?}{:?}{}", r.switch_tile, r.target_tile, r.on));
        // its button's two looks (the nodes are hashed with the world)
        material(&mut h, &r.btn_on, &mut seen);
        material(&mut h, &r.btn_off, &mut seen);
    }
    h.bytes(&g.tiles);
    h.str(&format!("{:?}", g.solids));
    h.0
}

const TRIES: u32 = 256;
const PRINTS: usize = 4;

/// One run of `name` on the smoke level `k`, as a level's build runs it:
/// its prints once placed and once used, or `None` if it placed nothing.
/// Also returns how many per-frame (native) draws the placement took, which
/// must be none: a level is built from its own stream alone.
fn run_once(name: &'static str, f: PlacerFn, k: u32) -> (Option<(u64, u64)>, u64) {
    let (mut g, dist) = Game::test_level(k * 7919 + 13, 21 + (k as i32 % 3) * 8);
    game::placers::kit::level_begins();
    // what runs after the build (animators, a thing used) draws from the
    // native stream, which starts from the clock unless seeded
    game::base::rng::seed_native(u64::from(k));
    let from = (g.world.child_count(), g.interactables.len(), g.animators.len());
    let tiles = g.tiles.clone();
    let native = game::base::rng::native_draws();
    let drawn = game::base::rng::with_seed(k.wrapping_mul(2654435761), || {
        let before = game::base::rng::seeded_draws();
        g.run_placer_as(name, f, &dist);
        game::base::rng::seeded_draws() - before
    });
    let native = game::base::rng::native_draws() - native;
    let placed = g.world.child_count() > from.0 || g.interactables.len() > from.1 || g.animators.len() > from.2 || g.tiles != tiles;
    if !placed {
        return (None, native);
    }
    // how many draws it took from the level's stream: one more or fewer
    // shifts every placer after it on a real level
    let a = print(&mut g, from) ^ drawn.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    for s in 0..30 {
        g.perf_t = s as f64 * 0.37;
        g.run_animators(1.0 / 60.0, g.perf_t);
    }
    g.player.flashlight_on = true;
    g.use_all_interactables();
    for s in 0..10 {
        g.run_animators(0.25, 20.0 + s as f64);
    }
    (Some((a, print(&mut g, from))), native)
}

struct Prints {
    text: String,
    /// placers whose placement drew from the native stream
    native: Vec<&'static str>,
    /// placements whose print differs when run on a fresh thread: what they
    /// make depends on what the thread made before (a cache, a counter)
    history: Vec<String>,
}

fn prints() -> Prints {
    let mut list: Vec<(&'static str, PlacerFn)> = vec![];
    placers::register_all(&mut list);
    list.sort_by_key(|(n, _)| *n);
    let mut out = Prints { text: String::from("# placer\tlevel\tplaced\tused\n"), native: vec![], history: vec![] };
    for (name, f) in list {
        let mut found = 0;
        for k in 0..TRIES {
            let (got, native) = run_once(name, f, k);
            if native > 0 && !out.native.contains(&name) {
                out.native.push(name);
            }
            let Some((a, b)) = got else { continue };
            let _ = writeln!(out.text, "{name}\t{k}\t{a:016x}\t{b:016x}");
            // the same run on a thread that has made nothing yet
            let (fresh, fresh_native) = std::thread::spawn(move || run_once(name, f, k)).join().unwrap();
            if fresh != Some((a, b)) {
                out.history.push(format!("{name} (level {k})"));
            }
            if fresh_native > 0 && !out.native.contains(&name) {
                out.native.push(name);
            }
            found += 1;
            if found == PRINTS {
                break;
            }
        }
        if found == 0 {
            let _ = writeln!(out.text, "{name}\t-\tplaced nothing in {TRIES} levels\t-");
        }
    }
    out
}

/// The prints by placer: (placer, level) -> the rest of its line
fn by_placer(text: &str) -> std::collections::BTreeMap<(String, String), String> {
    text.lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let mut c = l.splitn(3, '\t');
            Some(((c.next()?.to_string(), c.next()?.to_string()), c.next()?.to_string()))
        })
        .collect()
}

/// The placers whose prints changed, appeared or went between two recordings
fn changed_placers(got: &str, want: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let (g, w) = (by_placer(got), by_placer(want));
    let names = |m: &std::collections::BTreeMap<(String, String), String>| {
        m.keys().map(|(n, _)| n.clone()).collect::<std::collections::BTreeSet<_>>()
    };
    let (gn, wn) = (names(&g), names(&w));
    let changed = gn
        .intersection(&wn)
        .filter(|n| {
            let mine = |m: &std::collections::BTreeMap<(String, String), String>| {
                m.iter().filter(|((p, _), _)| p == *n).map(|(k, v)| (k.1.clone(), v.clone())).collect::<Vec<_>>()
            };
            mine(&g) != mine(&w)
        })
        .cloned()
        .collect();
    (changed, gn.difference(&wn).cloned().collect(), wn.difference(&gn).cloned().collect())
}

#[test]
fn every_placer_makes_what_it_made() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fingerprints.tsv");
    let got = prints();
    assert!(
        got.native.is_empty(),
        "these placers drew from the per-frame (native) stream while a level was built: a level is built from its own stream \
         alone (`rnd()`), or it is not the same twice: {:#?}",
        got.native
    );
    assert!(
        got.history.is_empty(),
        "these placers make something else on a thread that has made other things first (a cache filled, a counter moved): \
         a level must come out the same whatever was built before it: {:#?}",
        got.history
    );
    let want = std::fs::read_to_string(&path).unwrap_or_default();
    let (changed, added, gone) = changed_placers(&got.text, &want);
    let summary = format!("changed {changed:?}\nadded {added:?}\ngone {gone:?}");
    if let Ok(allowed) = std::env::var("REGEN") {
        // REGEN=1 records whatever changed (say why in the commit);
        // REGEN=place_a,place_b refuses a change to any placer not named
        let named: Vec<&str> = allowed.split(',').map(str::trim).collect();
        if allowed != "1" {
            let stray: Vec<&String> = changed.iter().chain(&added).chain(&gone).filter(|n| !named.contains(&n.as_str())).collect();
            assert!(stray.is_empty(), "REGEN={allowed}, but these placers changed too, and are not recorded: {stray:?}");
        }
        println!("recording tests/fixtures/fingerprints.tsv:\n{summary}");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &got.text).unwrap();
        return;
    }
    assert!(
        changed.is_empty() && added.is_empty() && gone.is_empty(),
        "a placer makes something else than tests/fixtures/fingerprints.tsv records:\n{summary}\n\
         If that is meant: REGEN=<those placers, comma-separated> (REGEN=1 for any), and say why in the commit"
    );
}

#[test]
fn the_comparison_names_the_placers_that_changed() {
    let want = "# h\nplace_a\t0\t1\t1\nplace_a\t3\t2\t2\nplace_c\t1\t5\t5\n";
    let got = "# h\nplace_a\t0\t1\t1\nplace_a\t3\t2\t9\nplace_b\t0\t7\t7\nplace_c\t1\t5\t5\n";
    let (changed, added, gone) = changed_placers(got, want);
    assert_eq!((changed, added, gone), (vec!["place_a".to_string()], vec!["place_b".to_string()], vec![]));
}
