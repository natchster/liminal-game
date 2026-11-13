//! Bit-exact check of `game::level::archgen` and `game::level::scp` against
//! recorded reference data.
//!
//! The fixtures under `tests/fixtures/` record each level as the game builds
//! it (`with_seed(seed_for_level(run_seed, n), gen_layout_level)`). Here every
//! case is rebuilt through the layout generator
//! (`with_seed(seed_for_level(..), || generate_layout(.., &mut || rnd()))`)
//! and compared field by field, float by float (bit patterns).
//!
//!     cargo test -p game --test archgen
//!
//! After a deliberate change to the generator, re-record the cases (their
//! inputs — seeds, probe points, anchors — are kept) and review the diff:
//!
//!     cargo test --release -p game --test archgen -- --ignored rerecord

use game::base::numeric::round_decimals;
use game::base::rng::{rnd, seed_for_level, seeded_draws, with_seed};
use game::level::archgen::*;
use game::level::scp::*;
use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

// ------------------------------------------------------------------ fixtures
fn fixture(name: &str) -> Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    let s = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e} (missing fixture)", p.display()));
    serde_json::from_str(&s).unwrap()
}
fn case_files() -> Vec<String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.starts_with("archgen_case_") && n.ends_with(".json"))
        .collect();
    v.sort();
    v
}
/// a fixture number: JSON number, or "-0" / "NaN" / "Infinity" / "-Infinity"
fn num(v: &Value) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap(),
        Value::String(s) => match s.as_str() {
            "-0" => -0.0,
            "NaN" => f64::NAN,
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            _ => panic!("bad number {s:?}"),
        },
        _ => panic!("bad number {v}"),
    }
}
fn int(v: &Value) -> i64 {
    v.as_i64().unwrap_or_else(|| panic!("not an int: {v}"))
}
fn st(v: &Value) -> &str {
    v.as_str().unwrap_or_else(|| panic!("not a string: {v}"))
}
fn opt_str(v: &Value) -> Option<&str> {
    v.as_str()
}
fn arr(v: &Value) -> &Vec<Value> {
    v.as_array().unwrap_or_else(|| panic!("not an array: {v}"))
}
fn b64(s: &str) -> Vec<u8> {
    fn val(c: u8) -> u32 {
        match c {
            b'A'..=b'Z' => (c - b'A') as u32,
            b'a'..=b'z' => (c - b'a') as u32 + 26,
            b'0'..=b'9' => (c - b'0') as u32 + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("bad base64 {c}"),
        }
    }
    let bytes: Vec<u8> = s.bytes().filter(|&c| c != b'=').collect();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    for ch in bytes.chunks(4) {
        let mut acc = 0u32;
        for (i, &c) in ch.iter().enumerate() {
            acc |= val(c) << (18 - 6 * i);
        }
        out.push((acc >> 16) as u8);
        if ch.len() > 2 {
            out.push((acc >> 8) as u8);
        }
        if ch.len() > 3 {
            out.push(acc as u8);
        }
    }
    out
}
fn f32s(v: &Value) -> Vec<f32> {
    b64(st(v)).chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect()
}
fn i32s(v: &Value) -> Vec<i32> {
    b64(st(v)).chunks_exact(4).map(|c| i32::from_le_bytes(c.try_into().unwrap())).collect()
}
fn same(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}

/// Collects mismatches instead of stopping at the first one.
#[derive(Default)]
struct Check {
    checks: usize,
    bad: Vec<String>,
}
impl Check {
    fn ok(&mut self, cond: bool, msg: impl FnOnce() -> String) {
        self.checks += 1;
        if !cond {
            self.bad.push(msg());
        }
    }
    fn f(&mut self, what: &str, got: f64, want: f64) {
        self.ok(same(got, want), || format!("{what}: got {got:?}, want {want:?}"));
    }
    fn eq<T: PartialEq + std::fmt::Debug>(&mut self, what: &str, got: T, want: T) {
        self.ok(got == want, || format!("{what}: got {got:?}, want {want:?}"));
    }
    fn u8s(&mut self, what: &str, got: &[u8], want: &[u8]) {
        let first = got.iter().zip(want).position(|(a, b)| a != b);
        self.ok(got.len() == want.len() && first.is_none(), || {
            format!("{what}: len {} vs {}, first diff at {first:?}", got.len(), want.len())
        });
    }
    fn i32s(&mut self, what: &str, got: &[i32], want: &[i32]) {
        let first = got.iter().zip(want).position(|(a, b)| a != b);
        self.ok(got.len() == want.len() && first.is_none(), || {
            format!("{what}: len {} vs {}, first diff at {first:?}", got.len(), want.len())
        });
    }
    fn f32s(&mut self, what: &str, got: &[f32], want: &[f32]) {
        let first = got.iter().zip(want).position(|(a, b)| a.to_bits() != b.to_bits());
        self.ok(got.len() == want.len() && first.is_none(), || {
            let i = first.unwrap_or(0);
            format!("{what}: len {} vs {}, first diff at {first:?} ({:?} vs {:?})", got.len(), want.len(), got.get(i), want.get(i))
        });
    }
    fn finish(self, name: &str) {
        eprintln!("{name}: {} checks, {} mismatches", self.checks, self.bad.len());
        if !self.bad.is_empty() {
            for m in self.bad.iter().take(40) {
                eprintln!("  {m}");
            }
            panic!("{name}: {} of {} checks mismatched", self.bad.len(), self.checks);
        }
    }
}

// ------------------------------------------------------------------ helpers
fn check_solids(c: &mut Check, tag: &str, s: &Solids, j: &Value) {
    c.eq(&format!("{tag}.count"), s.count as i64, int(&j["count"]));
    c.eq(&format!("{tag}.W"), s.w as i64, int(&j["W"]));
    c.eq(&format!("{tag}.H"), s.h as i64, int(&j["H"]));
    c.f(&format!("{tag}.cell"), s.cell, num(&j["cell"]));
    c.f32s(&format!("{tag}.boxes"), &s.boxes, &f32s(&j["boxes"]));
    c.i32s(&format!("{tag}.start"), &s.start, &i32s(&j["start"]));
    c.i32s(&format!("{tag}.items"), &s.items, &i32s(&j["items"]));
}
fn check_stats(c: &mut Check, tag: &str, s: &LayoutStats, j: &Value) {
    c.eq(&format!("{tag}.W"), s.w as i64, int(&j["W"]));
    c.eq(&format!("{tag}.H"), s.h as i64, int(&j["H"]));
    c.eq(&format!("{tag}.open"), s.open as i64, int(&j["open"]));
    c.f(&format!("{tag}.open_share"), s.open_share, num(&j["openShare"]));
    c.eq(&format!("{tag}.zones"), s.zones as i64, int(&j["zones"]));
    let want: Vec<(String, i64)> = arr(&j["arch"]).iter().map(|e| (st(&e[0]).to_string(), int(&e[1]))).collect();
    let got: Vec<(String, i64)> = s.arch.iter().map(|&(k, a)| (k.to_string(), a as i64)).collect();
    c.eq(&format!("{tag}.arch"), got, want);
    c.eq(&format!("{tag}.boxes"), s.boxes as i64, int(&j["boxes"]));
    c.eq(&format!("{tag}.thin_boxes"), s.thin_boxes as i64, int(&j["thinBoxes"]));
    c.eq(&format!("{tag}.pillars"), s.pillars as i64, int(&j["pillars"]));
    c.eq(&format!("{tag}.jambs"), s.jambs as i64, int(&j["jambs"]));
    c.eq(&format!("{tag}.lintels"), s.lintels as i64, int(&j["lintels"]));
    c.eq(&format!("{tag}.anchors"), s.anchors as i64, int(&j["anchors"]));
}
fn check_zone(c: &mut Check, tag: &str, z: &Zone, j: &Value) {
    c.eq(&format!("{tag}.id"), z.id as i64, int(&j["id"]));
    c.eq(&format!("{tag}.arch"), z.arch, st(&j["arch"]));
    c.eq(&format!("{tag}.rect"), [z.x0, z.z0, z.x1, z.z1].map(|v| v as i64), [int(&j["x0"]), int(&j["z0"]), int(&j["x1"]), int(&j["z1"])]);
    c.eq(&format!("{tag}.title"), z.title, st(&j["title"]));
    c.eq(&format!("{tag}.lore"), z.lore, st(&j["lore"]));
    c.f(&format!("{tag}.ceil_h"), z.ceil_h, num(&j["ceilH"]));
    c.eq(&format!("{tag}.wall"), z.wall, st(&j["wall"]));
    c.f(&format!("{tag}.wall_t"), z.wall_t, num(&j["wallT"]));
    c.f(&format!("{tag}.pillar"), z.pillar, num(&j["pillar"]));
    c.f(&format!("{tag}.protrude"), z.protrude, num(&j["protrude"]));
    c.f(&format!("{tag}.jamb"), z.jamb, num(&j["jamb"]));
    c.f(&format!("{tag}.lintel_h"), z.lintel_h, num(&j["lintelH"]));
    c.f(&format!("{tag}.low_h"), z.low_h, num(&j["lowH"]));
    c.eq(&format!("{tag}.mats"), [z.wall_mat, z.floor_mat, z.ceil_mat], [st(&j["wallMat"]), st(&j["floorMat"]), st(&j["ceilMat"])]);
    c.eq(&format!("{tag}.light_every"), z.light_every as i64, int(&j["lightEvery"]));
}
fn check_probes(c: &mut Check, tag: &str, s: &Solids, j: &Value) {
    for (i, p) in arr(&j["circleHits"]).iter().enumerate() {
        let (u, v, r, y) = (num(&p[0]), num(&p[1]), num(&p[2]), num(&p[3]));
        let got = circle_hits(s, u, v, r, y);
        c.ok(got == p[4].as_bool().unwrap(), || format!("{tag}.circle_hits[{i}]({u:?}, {v:?}, {r:?}, {y:?}) = {got}"));
    }
    for (i, p) in arr(&j["pointInSolid"]).iter().enumerate() {
        let (u, v, y) = (num(&p[0]), num(&p[1]), num(&p[2]));
        let got = point_in_solid(s, u, v, y);
        c.ok(got as i64 == int(&p[3]), || format!("{tag}.point_in_solid[{i}]({u:?}, {v:?}, {y:?}) = {got}, want {}", p[3]));
    }
    for (i, p) in arr(&j["segmentClear"]).iter().enumerate() {
        let (u0, v0, u1, v1, y) = (num(&p[0]), num(&p[1]), num(&p[2]), num(&p[3]), num(&p[4]));
        let got = segment_clear(s, u0, v0, u1, v1, y);
        c.ok(got == p[5].as_bool().unwrap(), || format!("{tag}.segment_clear[{i}]({u0:?}, {v0:?}, {u1:?}, {v1:?}, {y:?}) = {got}"));
    }
    for (i, p) in arr(&j["rayAxis"]).iter().enumerate() {
        let (u, v, d, mt, y) = (num(&p[0]), num(&p[1]), int(&p[2]) as usize, num(&p[3]), num(&p[4]));
        let got = ray_axis(s, u, v, d, mt, y);
        let want = num(&p[5]);
        c.ok(same(got, want), || format!("{tag}.ray_axis[{i}]({u:?}, {v:?}, {d}, {mt:?}, {y:?}) = {got:?}, want {want:?}"));
    }
}

// ------------------------------------------------------------------ tests
#[test]
fn archgen_tables() {
    let t = fixture("archgen_tables.json");
    let mut c = Check::default();
    let dirs: Vec<[i64; 2]> = arr(&t["DIRS"]).iter().map(|d| [int(&d[0]), int(&d[1])]).collect();
    c.eq("DIRS", DIRS.iter().map(|d| [d[0] as i64, d[1] as i64]).collect::<Vec<_>>(), dirs);
    c.eq("TF_LOW", TF_LOW as i64, int(&t["TF_LOW"]));
    let b = &t["BOX"];
    c.eq(
        "BOX_*",
        [BOX_WALL, BOX_PILLAR, BOX_JAMB, BOX_LINTEL, BOX_ANCHOR].map(|v| v as i64),
        [int(&b["WALL"]), int(&b["PILLAR"]), int(&b["JAMB"]), int(&b["LINTEL"]), int(&b["ANCHOR"])],
    );

    // ARCHETYPES: same keys in the same order, every field
    let known = [
        "key",
        "title",
        "lore",
        "ceil",
        "wall",
        "t",
        "protrude",
        "pillar",
        "jamb",
        "jambChance",
        "lintel",
        "lowH",
        "wallMat",
        "floorMat",
        "ceilMat",
        "lightEvery",
        "noSpawn",
    ];
    let ja = arr(&t["ARCHETYPES"]);
    c.eq("ARCHETYPES.len", ARCHETYPES.len(), ja.len());
    let pair = |v: &Value| [num(&v[0]), num(&v[1])];
    for (a, j) in ARCHETYPES.iter().zip(ja) {
        let tag = format!("ARCHETYPES.{}", a.key);
        for k in j.as_object().unwrap().keys() {
            c.ok(known.contains(&k.as_str()), || format!("{tag}: field {k} not checked"));
        }
        c.eq(&format!("{tag}.key"), a.key, st(&j["key"]));
        c.eq(&format!("{tag}.title"), a.title, st(&j["title"]));
        c.eq(&format!("{tag}.lore"), a.lore, st(&j["lore"]));
        c.eq(&format!("{tag}.wall"), a.wall, st(&j["wall"]));
        for (name, got, want) in [
            ("ceil", a.ceil, pair(&j["ceil"])),
            ("t", a.t, pair(&j["t"])),
            ("pillar", a.pillar, pair(&j["pillar"])),
            ("jamb", a.jamb, pair(&j["jamb"])),
            ("lintel", a.lintel, pair(&j["lintel"])),
        ] {
            c.ok(same(got[0], want[0]) && same(got[1], want[1]), || format!("{tag}.{name}: {got:?} vs {want:?}"));
        }
        let optpair = |v: &Value| if v.is_null() { None } else { Some(pair(v)) };
        c.eq(&format!("{tag}.protrude"), a.protrude, optpair(&j["protrude"]));
        c.eq(&format!("{tag}.low_h"), a.low_h, optpair(&j["lowH"]));
        c.f(&format!("{tag}.jamb_chance"), a.jamb_chance, num(&j["jambChance"]));
        c.eq(&format!("{tag}.mats"), [a.wall_mat, a.floor_mat, a.ceil_mat], [st(&j["wallMat"]), st(&j["floorMat"]), st(&j["ceilMat"])]);
        c.eq(&format!("{tag}.light_every"), a.light_every as i64, int(&j["lightEvery"]));
        c.eq(&format!("{tag}.no_spawn"), a.no_spawn, j["noSpawn"].as_bool().unwrap_or(false));
        c.ok(archetype(a.key).map(|x| x.key) == Some(a.key), || format!("{tag}: archetype() lookup"));
    }

    for e in arr(&t["levelSchedule"]) {
        let (lv, scp) = (int(&e["level"]) as i32, opt_str(&e["scp"]));
        let s = level_schedule(lv, scp);
        let tag = format!("level_schedule({lv}, {scp:?})");
        c.eq(&format!("{tag}.dominant"), s.dominant, st(&e["dominant"]));
        c.eq(&format!("{tag}.patchwork"), s.patchwork, e["patchwork"].as_bool().unwrap());
        let want: Vec<(String, u64)> = arr(&e["weights"]).iter().map(|w| (st(&w[0]).to_string(), num(&w[1]).to_bits())).collect();
        let got: Vec<(String, u64)> = s.weights.iter().map(|&(k, w)| (k.to_string(), w.to_bits())).collect();
        c.eq(&format!("{tag}.weights"), got, want);
    }

    for e in arr(&t["mulberry32"]) {
        let seed = int(&e["seed"]) as u32;
        let mut r = mulberry32(seed);
        for (i, v) in arr(&e["values"]).iter().enumerate() {
            c.f(&format!("mulberry32({seed})[{i}]"), r(), num(v));
        }
    }
    c.finish("archgen_tables");
}

#[test]
fn archgen_layouts_solids_queries() {
    let files = case_files();
    assert!(files.len() >= 20, "expected >= 20 archgen cases, found {}", files.len());
    let mut c = Check::default();
    for name in &files {
        let fx = fixture(name);
        let run_seed = int(&fx["runSeed"]) as u32;
        let level = int(&fx["level"]) as i32;
        let scp = opt_str(&fx["scp"]);
        let size = int(&fx["size"]) as i32;
        let tag = format!("{name}(rs={run_seed} L{level} {scp:?})");
        // `init_level`: the level seed is `seed_for_level(run_seed, n)`, its anomaly `scp_on_level(n)`
        let seed = seed_for_level(run_seed, level);
        c.eq(&format!("{tag}.seed"), seed as i64, int(&fx["seed"]));
        c.eq(&format!("{tag}.size"), (31 + level * 4).min(53), size);
        // (recorded when the anomalies came from level 2: the anomaly the
        // plan of that roster put on this level; the run's plan now keeps
        // them below the Building and the Lobby)
        if fx["scpOpt"].as_str() == Some("plan") {
            c.eq(&format!("{tag}.scp_for_level"), scp_plan(run_seed, &recorded_roster(), SCP_HORIZON).get(level), scp);
        }

        // `gen_layout_level(n, size)` under `with_seed(seed)`
        let opts = LayoutOpts { size, level, cell: 4.0, scp };
        let (layout, tile_ceil, solids, draws) = with_seed(seed, || {
            let d0 = seeded_draws();
            let layout = generate_layout(&opts, &mut || rnd());
            let tile_ceil = ceiling_heights(&layout);
            let solids = build_solids(&layout, &SolidsOpts { ceil: Some(&tile_ceil), ..Default::default() });
            (layout, tile_ceil, solids, seeded_draws() - d0)
        });
        c.eq(&format!("{tag}.draws"), draws as i64, int(&fx["draws"]));
        // the same through an explicit generator
        let mut n = 0u64;
        let mut m = mulberry32(seed);
        let again = generate_layout(&opts, &mut || {
            n += 1;
            m()
        });
        c.ok(again == layout, || format!("{tag}: mulberry32 rng gave a different layout"));
        c.eq(&format!("{tag}.draws(mulberry32)"), n as i64, int(&fx["draws"]));

        let jl = &fx["layout"];
        c.eq(&format!("{tag}.W"), layout.w as i64, int(&jl["W"]));
        c.eq(&format!("{tag}.H"), layout.h as i64, int(&jl["H"]));
        c.f(&format!("{tag}.cell"), layout.cell, num(&jl["cell"]));
        c.eq(&format!("{tag}.level"), layout.level as i64, int(&jl["level"]));
        c.eq(&format!("{tag}.scp"), layout.scp.as_deref(), opt_str(&jl["scp"]));
        c.eq(&format!("{tag}.dominant"), layout.dominant, st(&jl["dominant"]));
        c.u8s(&format!("{tag}.tiles"), &layout.tiles, &b64(st(&jl["tiles"])));
        c.u8s(&format!("{tag}.zone_of"), &layout.zone_of, &b64(st(&jl["zoneOf"])));
        c.u8s(&format!("{tag}.tile_flags"), &layout.tile_flags, &b64(st(&jl["tileFlags"])));
        let jz = arr(&jl["zones"]);
        c.eq(&format!("{tag}.zones.len"), layout.zones.len(), jz.len());
        for (i, (z, j)) in layout.zones.iter().zip(jz).enumerate() {
            check_zone(&mut c, &format!("{tag}.zones[{i}]"), z, j);
        }
        for e in arr(&fx["storeCells"]) {
            let zi = int(&e["zone"]) as usize;
            let want: Vec<[i64; 4]> =
                arr(&e["cells"]).iter().map(|q| [int(&q["x0"]), int(&q["x1"]), int(&q["z0"]), int(&q["z1"])]).collect();
            let got: Vec<[i64; 4]> = layout
                .zones
                .get(zi)
                .map(store_cells)
                .unwrap_or_default()
                .iter()
                .map(|q| [q.x0, q.x1, q.z0, q.z1].map(|v| v as i64))
                .collect();
            c.eq(&format!("{tag}.store_cells(zone {zi})"), got, want);
        }

        c.f32s(&format!("{tag}.ceiling_heights"), &tile_ceil, &f32s(&fx["ceil"]));
        check_solids(&mut c, &format!("{tag}.solids"), &solids, &fx["solids"]);
        check_stats(&mut c, &format!("{tag}.stats"), &layout_stats(&layout, Some(&solids)), &fx["stats"]);
        check_stats(&mut c, &format!("{tag}.stats_no_solids"), &layout_stats(&layout, None), &fx["statsNoSolids"]);
        let plain = build_solids(&layout, &SolidsOpts::default());
        let plain_same = plain.count == solids.count
            && plain.boxes.iter().map(|v| v.to_bits()).eq(solids.boxes.iter().map(|v| v.to_bits()))
            && plain.start == solids.start
            && plain.items == solids.items;
        c.eq(&format!("{tag}.plain_same"), plain_same, fx["plainSame"].as_bool().unwrap());
        check_probes(&mut c, &format!("{tag}.probes"), &solids, &fx["probes"]);

        // `settle_props`-style rebuild: pinned faces + raised ceilings
        let ja = &fx["anchored"];
        let anchors: HashSet<usize> = arr(&ja["anchors"]).iter().map(|v| int(v) as usize).collect();
        let mut ceil2 = tile_ceil.clone();
        for e in arr(&ja["raised"]) {
            ceil2[int(&e[0]) as usize] = num(&e[1]) as f32;
        }
        let anchored = build_solids(&layout, &SolidsOpts { anchors: Some(&anchors), ceil: Some(&ceil2), ..Default::default() });
        check_solids(&mut c, &format!("{tag}.anchored"), &anchored, &ja["solids"]);
        check_stats(&mut c, &format!("{tag}.anchored.stats"), &layout_stats(&layout, Some(&anchored)), &ja["stats"]);
        check_probes(&mut c, &format!("{tag}.probes_anchored"), &anchored, &fx["probesAnchored"]);
        // a view over the same arrays builds the same boxes
        let viewed = build_solids_view(&layout.view(), &SolidsOpts { anchors: Some(&anchors), ceil: Some(&ceil2), ..Default::default() });
        c.ok(viewed.boxes == anchored.boxes && viewed.items == anchored.items, || format!("{tag}: build_solids_view differs"));
    }
    c.finish("archgen_layouts_solids_queries");
}

#[test]
fn archgen_rooms_and_ceilings() {
    for (run_seed, level, scp) in [(4242u32, 1, None), (4242, 5, None), (77, 9, None), (9, 6, Some("store")), (31, 7, Some("house"))] {
        let seed = seed_for_level(run_seed, level);
        let size = (31 + level * 4).min(53);
        let l = with_seed(seed, || generate_layout(&LayoutOpts { size, level, cell: 4.0, scp }, &mut || rnd()));
        let tag = format!("rs={run_seed} L{level}");
        // every zone has rooms, and every room lies inside its zone
        for z in &l.zones {
            let rooms: Vec<&Room> = l.rooms.iter().filter(|r| r.zone == z.id).collect();
            assert!(!rooms.is_empty(), "{tag}: zone {} has no room", z.id);
            for r in rooms {
                assert!(
                    r.x0 >= z.x0 && r.x1 <= z.x1 && r.z0 >= z.z0 && r.z1 <= z.z1 && r.x0 <= r.x1 && r.z0 <= r.z1,
                    "{tag}: room {r:?} outside zone {}",
                    z.id
                );
            }
        }
        // neighbours of one archetype share a ceiling
        for a in &l.zones {
            for b in &l.zones {
                if a.arch == b.arch && a.touches(b) {
                    assert_eq!(a.ceil_h, b.ceil_h, "{tag}: zones {} and {} ({}) step their ceiling", a.id, b.id, a.arch);
                }
            }
            assert!(a.lintel_h <= a.ceil_h - 0.25, "{tag}: zone {} lintel above its ceiling", a.id);
        }
    }
}

// ------------------------------------------------------------------ re-recording
fn b64_encode(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for ch in bytes.chunks(3) {
        let acc = ((ch[0] as u32) << 16) | ((*ch.get(1).unwrap_or(&0) as u32) << 8) | *ch.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= ch.len() {
                s.push(T[((acc >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                s.push('=');
            }
        }
    }
    s
}
fn f32s_b64(v: &[f32]) -> Value {
    Value::String(b64_encode(&v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>()))
}
fn i32s_b64(v: &[i32]) -> Value {
    Value::String(b64_encode(&v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>()))
}
/// a fixture number (the inverse of `num`)
fn jnum(v: f64) -> Value {
    if v.is_nan() {
        "NaN".into()
    } else if v == f64::INFINITY {
        "Infinity".into()
    } else if v == f64::NEG_INFINITY {
        "-Infinity".into()
    } else if v == 0.0 && v.is_sign_negative() {
        "-0".into()
    } else if v.fract() == 0.0 && v.abs() < 1e15 {
        (v as i64).into()
    } else {
        v.into()
    }
}
fn solids_json(s: &Solids) -> Value {
    serde_json::json!({ "count": s.count, "W": s.w, "H": s.h, "cell": jnum(s.cell), "boxes": f32s_b64(&s.boxes), "start": i32s_b64(&s.start), "items": i32s_b64(&s.items) })
}
fn stats_json(s: &LayoutStats) -> Value {
    serde_json::json!({
        "W": s.w, "H": s.h, "open": s.open, "openShare": jnum(s.open_share), "zones": s.zones,
        "arch": s.arch.iter().map(|&(k, a)| serde_json::json!([k, a])).collect::<Vec<_>>(),
        "boxes": s.boxes, "thinBoxes": s.thin_boxes, "pillars": s.pillars, "jambs": s.jambs, "lintels": s.lintels, "anchors": s.anchors,
    })
}
fn zone_json(z: &Zone) -> Value {
    serde_json::json!({
        "id": z.id, "arch": z.arch, "x0": z.x0, "z0": z.z0, "x1": z.x1, "z1": z.z1, "title": z.title, "lore": z.lore,
        "ceilH": jnum(z.ceil_h), "wall": z.wall, "wallT": jnum(z.wall_t), "pillar": jnum(z.pillar), "protrude": jnum(z.protrude),
        "jamb": jnum(z.jamb), "lintelH": jnum(z.lintel_h), "lowH": jnum(z.low_h), "wallMat": z.wall_mat, "floorMat": z.floor_mat,
        "ceilMat": z.ceil_mat, "lightEvery": z.light_every,
    })
}
/// The probe lists with their answers recomputed (the points are kept)
fn reprobe(s: &Solids, j: &Value) -> Value {
    let redo = |key: &str, at: usize, f: &dyn Fn(&[Value]) -> Value| -> Value {
        Value::Array(
            arr(&j[key])
                .iter()
                .map(|p| {
                    let mut p = arr(p).clone();
                    p[at] = f(&p);
                    Value::Array(p)
                })
                .collect(),
        )
    };
    serde_json::json!({
        "circleHits": redo("circleHits", 4, &|p| circle_hits(s, num(&p[0]), num(&p[1]), num(&p[2]), num(&p[3])).into()),
        "pointInSolid": redo("pointInSolid", 3, &|p| point_in_solid(s, num(&p[0]), num(&p[1]), num(&p[2])).into()),
        "segmentClear": redo("segmentClear", 5, &|p| segment_clear(s, num(&p[0]), num(&p[1]), num(&p[2]), num(&p[3]), num(&p[4])).into()),
        "rayAxis": redo("rayAxis", 5, &|p| jnum(ray_axis(s, num(&p[0]), num(&p[1]), int(&p[2]) as usize, num(&p[3]), num(&p[4])))),
    })
}

/// Rewrites every `archgen_case_*.json` from the current generator, keeping
/// each case's inputs (run seed, level, size, probe points, anchors, raised
/// ceilings).
#[test]
#[ignore = "rewrites the fixtures"]
fn rerecord_archgen_cases() {
    for name in case_files() {
        let mut fx = fixture(&name);
        let (run_seed, level, size) = (int(&fx["runSeed"]) as u32, int(&fx["level"]) as i32, int(&fx["size"]) as i32);
        let scp = opt_str(&fx["scp"]).map(|s| s.to_string());
        let seed = seed_for_level(run_seed, level);
        let opts = LayoutOpts { size, level, cell: 4.0, scp: scp.as_deref() };
        let (layout, tile_ceil, solids, draws) = with_seed(seed, || {
            let d0 = seeded_draws();
            let layout = generate_layout(&opts, &mut || rnd());
            let tile_ceil = ceiling_heights(&layout);
            let solids = build_solids(&layout, &SolidsOpts { ceil: Some(&tile_ceil), ..Default::default() });
            (layout, tile_ceil, solids, seeded_draws() - d0)
        });
        fx["draws"] = draws.into();
        fx["layout"] = serde_json::json!({
            "W": layout.w, "H": layout.h, "cell": jnum(layout.cell), "level": layout.level, "scp": layout.scp, "dominant": layout.dominant,
            "tiles": b64_encode(&layout.tiles), "zoneOf": b64_encode(&layout.zone_of), "tileFlags": b64_encode(&layout.tile_flags),
            "zones": layout.zones.iter().map(zone_json).collect::<Vec<_>>(),
        });
        let cells: Vec<Value> = arr(&fx["storeCells"])
            .iter()
            .map(|e| {
                let zi = int(&e["zone"]) as usize;
                let cells = layout.zones.get(zi).map(store_cells).unwrap_or_default();
                serde_json::json!({ "zone": zi, "cells": cells.iter().map(|q| serde_json::json!({ "x0": q.x0, "x1": q.x1, "z0": q.z0, "z1": q.z1 })).collect::<Vec<_>>() })
            })
            .collect();
        fx["storeCells"] = cells.into();
        fx["ceil"] = f32s_b64(&tile_ceil);
        fx["solids"] = solids_json(&solids);
        fx["stats"] = stats_json(&layout_stats(&layout, Some(&solids)));
        fx["statsNoSolids"] = stats_json(&layout_stats(&layout, None));
        let plain = build_solids(&layout, &SolidsOpts::default());
        fx["plainSame"] = (plain.count == solids.count
            && plain.boxes.iter().map(|v| v.to_bits()).eq(solids.boxes.iter().map(|v| v.to_bits()))
            && plain.start == solids.start
            && plain.items == solids.items)
            .into();
        fx["probes"] = reprobe(&solids, &fx["probes"]);
        let anchors: HashSet<usize> = arr(&fx["anchored"]["anchors"]).iter().map(|v| int(v) as usize).collect();
        let mut ceil2 = tile_ceil.clone();
        for e in arr(&fx["anchored"]["raised"]) {
            ceil2[int(&e[0]) as usize] = num(&e[1]) as f32;
        }
        let anchored = build_solids(&layout, &SolidsOpts { anchors: Some(&anchors), ceil: Some(&ceil2), ..Default::default() });
        fx["anchored"]["solids"] = solids_json(&anchored);
        fx["anchored"]["stats"] = stats_json(&layout_stats(&layout, Some(&anchored)));
        fx["probesAnchored"] = reprobe(&anchored, &fx["probesAnchored"]);
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(&name);
        std::fs::write(&p, serde_json::to_string(&fx).unwrap()).unwrap();
        eprintln!("rewrote {}", p.display());
    }
}

#[test]
fn archgen_to_fixed() {
    // rounding to d decimals on the exact binary value, ties away from zero
    let cases: [(f64, usize, f64); 15] = [
        (1.005, 2, 1.0),
        (0.125, 2, 0.13),
        (0.375, 2, 0.38),
        (2.675, 2, 2.67),
        (1.45, 1, 1.4),
        (0.5, 0, 1.0),
        (2.5, 0, 3.0),
        (8.345, 2, 8.35),
        (1.0005, 3, 1.0),
        (4.35, 1, 4.3),
        (-0.001, 2, -0.0),
        (9.995, 2, 9.99),
        (1.5, 0, 2.0),
        (0.045, 2, 0.04),
        (3.7250000000000005, 3, 3.725),
    ];
    let mut c = Check::default();
    for (x, d, want) in cases {
        c.f(&format!("round_decimals({x}, {d})"), round_decimals(x, d), want);
    }
    c.finish("archgen_to_fixed");
}

/// The roster `scp_plan.json` was recorded with
fn recorded_roster() -> Vec<ScpEntry> {
    let key = |k: &str| *SCP_KEYS.iter().find(|x| **x == k).unwrap();
    arr(&fixture("scp_plan.json")["SCP_ROSTER"])
        .iter()
        .map(|e| ScpEntry { key: key(st(&e["key"])), from: int(&e["from"]) as i32, weight: num(&e["weight"]) })
        .collect()
}
fn plan_json(p: &ScpPlan) -> Vec<(i64, String)> {
    p.entries.iter().map(|&(n, k)| (n as i64, k.to_string())).collect()
}
fn want_plan(v: &Value) -> Vec<(i64, String)> {
    arr(v).iter().map(|e| (int(&e[0]), st(&e[1]).to_string())).collect()
}

#[test]
fn scp_plans() {
    let f = fixture("scp_plan.json");
    let mut c = Check::default();
    // The plans were recorded with the roster of their day (anomalies from
    // level 2 or 3): the planner is checked against them with that roster.
    // The run's roster now starts every anomaly on level 5, below the
    // Building's floors and the Lobby (`docs/design/MEGASTRUCTURE.md`); its plans
    // are checked by the rules.
    let old = recorded_roster();
    c.eq("SCP_ROSTER keys", SCP_ROSTER.iter().map(|e| e.key).collect::<Vec<_>>(), old.iter().map(|e| e.key).collect::<Vec<_>>());
    assert!(SCP_ROSTER.iter().all(|e| e.from == 5 && e.weight == 1.0), "every anomaly from level 5");
    c.eq(
        "SCP_KEYS",
        SCP_KEYS.iter().map(|k| k.to_string()).collect::<Vec<_>>(),
        arr(&f["SCP_KEYS"]).iter().map(|k| st(k).to_string()).collect::<Vec<_>>(),
    );
    c.eq("SCP_HORIZON", SCP_HORIZON as i64, int(&f["SCP_HORIZON"]));
    let plans = arr(&f["plans"]);
    assert!(plans.len() >= 500, "expected >= 500 plans");
    for e in plans {
        let seed = int(&e["seed"]) as u32;
        c.eq(&format!("scp_plan({seed})"), plan_json(&scp_plan(seed, &old, SCP_HORIZON)), want_plan(&e["plan"]));
        // the run's plan: each anomaly once, on levels 5–12, never two in a row
        let p = scp_plan_default(seed);
        c.eq(&format!("scp_plan({seed}) default"), plan_json(&p), plan_json(&scp_plan(seed, &SCP_ROSTER, SCP_HORIZON)));
        let mut levels: Vec<i32> = p.levels().collect();
        levels.sort();
        c.ok(levels.len() == 4, || format!("scp_plan({seed}): {levels:?}"));
        c.ok(levels.iter().all(|&n| (5..=SCP_HORIZON).contains(&n)), || format!("scp_plan({seed}): {levels:?} outside 5–12"));
        c.ok(levels.windows(2).all(|w| w[1] - w[0] >= 2), || format!("scp_plan({seed}): {levels:?} two in a row"));
        for n in 0..=SCP_HORIZON + 2 {
            c.eq(&format!("scp_for_level({seed}, {n})"), scp_for_level(seed, n), p.get(n));
        }
    }
    for cs in arr(&f["custom"]) {
        let name = st(&cs["name"]);
        // keys must be 'static: map them onto the roster's names
        let key = |k: &str| *SCP_KEYS.iter().find(|x| **x == k).unwrap();
        let roster: Vec<ScpEntry> = arr(&cs["roster"])
            .iter()
            .map(|e| ScpEntry { key: key(st(&e["key"])), from: int(&e["from"]) as i32, weight: num(&e["weight"]) })
            .collect();
        let horizon = int(&cs["horizon"]) as i32;
        for e in arr(&cs["plans"]) {
            let seed = int(&e["seed"]) as u32;
            c.eq(&format!("scp_plan({seed}, {name})"), plan_json(&scp_plan(seed, &roster, horizon)), want_plan(&e["plan"]));
        }
    }
    c.finish("scp_plans");
}
