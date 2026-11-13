//! Every placer's output, fingerprinted. Each placer is run on the smoke
//! test's small levels (`Game::test_level`) until it has placed something
//! four times, and what it added is hashed: every node it made (its name, kind,
//! place, flags and user data; its geometry; its materials and the pixels of
//! their textures), its interactables, and the level's solids and tiles; then
//! again once its animators have run and what it made usable has been used.
//! No id is hashed (ids count what was made before), so a print is the
//! placer's alone.
//!
//! The prints are recorded in `tests/fixtures/fingerprints.tsv`. Moving a
//! placer's code (another module, crate or theme), sharing a helper, or
//! tidying one must leave every print as it was; a change that is meant to
//! change what a placer makes is re-recorded, and says so:
//!
//!   cargo test --release -p placers --test fingerprints
//!   REGEN=1 cargo test --release -p placers --test fingerprints
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

fn material(h: &mut Fnv, m: &Mat, seen: &mut HashMap<usize, u64>) {
    let m = m.m();
    h.str(&format!("{:?}|{}|{:?}|{:?}|{:?}", m.kind, m.name, m.side, m.blending, m.patch));
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
    h.bytes(&g.tiles);
    h.str(&format!("{:?}", g.solids));
    h.0
}

const TRIES: u32 = 256;
const PRINTS: usize = 4;

fn prints() -> String {
    let mut list: Vec<(&'static str, PlacerFn)> = vec![];
    placers::register_all(&mut list);
    list.sort_by_key(|(n, _)| *n);
    let mut out = String::from("# placer\tlevel\tplaced\tused\n");
    for (name, f) in list {
        let mut found = 0;
        for k in 0..TRIES {
            let (mut g, dist) = Game::test_level(k * 7919 + 13, 21 + (k as i32 % 3) * 8);
            game::placers::kit::restart_texture_streams();
            // what runs after the build (animators, a thing used) draws from the
            // native stream, which starts from the clock unless seeded
            game::base::rng::seed_native(u64::from(k));
            let from = (g.world.child_count(), g.interactables.len(), g.animators.len());
            let tiles = g.tiles.clone();
            game::base::rng::with_seed(k.wrapping_mul(2654435761), || g.run_placer_as(name, f, &dist));
            let placed = g.world.child_count() > from.0 || g.interactables.len() > from.1 || g.animators.len() > from.2 || g.tiles != tiles;
            if !placed {
                continue;
            }
            let a = print(&mut g, from);
            for s in 0..30 {
                g.perf_t = s as f64 * 0.37;
                g.run_animators(1.0 / 60.0, g.perf_t);
            }
            g.player.flashlight_on = true;
            g.use_all_interactables();
            for s in 0..10 {
                g.run_animators(0.25, 20.0 + s as f64);
            }
            let b = print(&mut g, from);
            let _ = writeln!(out, "{name}\t{k}\t{a:016x}\t{b:016x}");
            found += 1;
            if found == PRINTS {
                break;
            }
        }
        if found == 0 {
            let _ = writeln!(out, "{name}\t-\tplaced nothing in {TRIES} levels\t-");
        }
    }
    out
}

#[test]
fn every_placer_makes_what_it_made() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fingerprints.tsv");
    let got = prints();
    if std::env::var("REGEN").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &got).unwrap();
    }
    let want = std::fs::read_to_string(&path).unwrap_or_default();
    let differ: Vec<String> = got.lines().zip(want.lines()).filter(|(a, b)| a != b).map(|(a, b)| format!("now  {a}\n  was  {b}")).collect();
    assert!(
        differ.is_empty() && got.lines().count() == want.lines().count(),
        "{} prints differ from tests/fixtures/fingerprints.tsv ({} lines now, {} recorded): a placer makes something else. \
         If that is meant, REGEN=1 and say why in the commit:\n  {}",
        differ.len(),
        got.lines().count(),
        want.lines().count(),
        differ.iter().take(20).cloned().collect::<Vec<_>>().join("\n  ")
    );
}
