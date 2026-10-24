//! Reference levels: builds levels from a New Game run and compares them with
//! recordings of the same (run seed, level) pairs in `tests/reference/`.
//!
//! A level is a pure function of its run seed and number, so every recorded
//! value has to come out the same: the seeded-draw count at each of the 687
//! build checkpoints (one per phase and per placer pass, so a difference
//! names the pass that diverged), the tiles and zones, the exit and the key
//! props, the counts, the entities, the spawn pose and every placed object.
//!
//!   cargo test --release -p backrooms --test reference_levels
use game::Game;
use serde_json::Value;

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

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or(f64::NAN)
}
fn close(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps * 1f64.max(a.abs()).max(b.abs())
}
fn vclose(a: &Value, b: &Value, eps: f64) -> bool {
    match (a.as_array(), b.as_array()) {
        (Some(x), Some(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| close(f(p), f(q), eps)),
        _ => a.is_null() && b.is_null(),
    }
}

/// Every difference between a built level (`Game::level_dump`) and its recording.
fn differences(got: &Value, want: &Value) -> Vec<String> {
    let mut d = vec![];
    // the draw trace: the first checkpoint that differs names the pass
    let (gm, wm) = (got["marks"].as_array().unwrap(), want["marks"].as_array().unwrap());
    if let Some(i) = (0..gm.len().max(wm.len())).find(|&i| gm.get(i) != wm.get(i)) {
        let prev = |m: &Vec<Value>| if i > 0 { m.get(i - 1).and_then(|x| x[1].as_u64()) } else { Some(0) };
        let drew = |m: &Vec<Value>| m.get(i).and_then(|x| x[1].as_u64()).zip(prev(m)).map(|(a, b)| a as i64 - b as i64);
        d.push(format!(
            "draw trace parts at checkpoint {i}: built {:?} vs recorded {:?} (that pass drew {:?} vs {:?})",
            gm.get(i),
            wm.get(i),
            drew(gm),
            drew(wm)
        ));
    }
    for k in ["scp", "weather"] {
        if got[k] != want[k] {
            d.push(format!("{k}: {} vs {}", got[k], want[k]));
        }
    }
    let tiles: String = got["grid"]["tiles"].as_array().unwrap().iter().map(|t| t.as_u64().unwrap().to_string()).collect();
    if (got["grid"]["W"].as_u64(), got["grid"]["H"].as_u64()) != (want["grid"]["w"].as_u64(), want["grid"]["h"].as_u64()) {
        d.push("grid size".into());
    }
    if tiles != want["grid"]["tiles"].as_str().unwrap() {
        d.push("tiles".into());
    }
    if want["zones"].is_array() {
        let zones: Vec<Value> = got["arch"]["zones"]
            .as_array()
            .unwrap()
            .iter()
            .map(|z| serde_json::json!([z["arch"], z["x0"], z["z0"], z["x1"], z["z1"]]))
            .collect();
        if zones != *want["zones"].as_array().unwrap() {
            d.push("zones".into());
        }
        if got["arch"]["dominant"] != want["dominant"] {
            d.push(format!("dominant: {} vs {}", got["arch"]["dominant"], want["dominant"]));
        }
    }
    let xz = |v: &Value| if v.is_null() { Value::Null } else { serde_json::json!([v["x"], v["z"]]) };
    if !vclose(&xz(&got["exit"]), &want["exit"], 1e-4) {
        d.push(format!("exit: {} vs {}", got["exit"], want["exit"]));
    }
    for k in ["tv", "phone", "clock", "mannequin"] {
        if !vclose(&xz(&got["props"][k]), &want["props"][k], 1e-4) {
            d.push(format!("{k}: {} vs {}", got["props"][k], want["props"][k]));
        }
    }
    for k in ["almonds", "batteries"] {
        let g: Vec<Value> = got["props"][k].as_array().unwrap().iter().map(xz).collect();
        let w = want[k].as_array().unwrap();
        if g.len() != w.len() || g.iter().zip(w).any(|(a, b)| !vclose(a, b, 1e-4)) {
            d.push(format!("{k} positions"));
        }
    }
    for k in ["walls", "panels", "almonds", "batteries", "interactables", "animators"] {
        if got["counts"][k] != want["counts"][k] {
            d.push(format!("count {k}: {} vs {}", got["counts"][k], want["counts"][k]));
        }
    }
    let ents: Vec<Value> = got["entities"].as_array().unwrap().iter().map(|e| serde_json::json!([e["type"], e["x"], e["z"]])).collect();
    let we = want["entities"].as_array().unwrap();
    if ents.len() != we.len()
        || ents.iter().zip(we).any(|(a, b)| a[0] != b[0] || !close(f(&a[1]), f(&b[1]), 1e-4) || !close(f(&a[2]), f(&b[2]), 1e-4))
    {
        d.push("entities".into());
    }
    let p = &got["player"];
    if !vclose(&serde_json::json!([p["x"], p["z"], p["yaw"]]), &want["player"], 1e-6) {
        d.push("spawn pose".into());
    }
    if got["barricades"] != want["barricades"] {
        d.push("barricades".into());
    }
    // placed objects, placer by placer, in order
    let group = |list: Vec<(String, Value)>| {
        let mut m: std::collections::BTreeMap<String, Vec<Value>> = Default::default();
        for (k, v) in list {
            m.entry(k).or_default().push(v);
        }
        m
    };
    let go = group(
        got["objects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| {
                (o["placer"].as_str().unwrap().to_string(), serde_json::json!([o["pos"], o["min"], o["max"], o["meshes"], o["interact"]]))
            })
            .collect(),
    );
    let wo = group(
        want["objects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| (o[0].as_str().unwrap().to_string(), serde_json::json!([o[1], o[2], o[3], o[4], o[5]])))
            .collect(),
    );
    for (placer, w) in &wo {
        let Some(g) = go.get(placer) else {
            d.push(format!("{placer}: none built ({} recorded)", w.len()));
            continue;
        };
        if g.len() != w.len() {
            d.push(format!("{placer}: {} objects vs {}", g.len(), w.len()));
            continue;
        }
        for (i, (a, b)) in g.iter().zip(w).enumerate() {
            let what = if !vclose(&a[0], &b[0], 1e-3) {
                "position"
            } else if !vclose(&a[1], &b[1], 2e-3) || !vclose(&a[2], &b[2], 2e-3) {
                "bounds"
            } else if a[3] != b[3] {
                "mesh count"
            } else if a[4] != b[4] {
                "interactive"
            } else {
                continue;
            };
            d.push(format!("{placer}[{i}] {what}: {a} vs {b}"));
            break;
        }
    }
    for placer in go.keys() {
        if !wo.contains_key(placer) {
            d.push(format!("{placer}: built but not recorded"));
        }
    }
    d
}

#[test]
fn levels_match_their_recordings() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/reference");
    let mut runs: std::collections::BTreeMap<u64, Vec<Value>> = Default::default();
    for e in std::fs::read_dir(&dir).expect("tests/reference") {
        let v: Value = serde_json::from_str(&std::fs::read_to_string(e.unwrap().path()).unwrap()).unwrap();
        runs.entry(v["seed"].as_u64().unwrap()).or_default().push(v);
    }
    let mut failures = vec![];
    let mut checked = 0;
    for (seed, mut levels) in runs {
        levels.sort_by_key(|v| v["level"].as_i64().unwrap());
        // one run per seed, started the way New Game starts it
        let mut g = Game::new_bare();
        register_placers(&mut g.placers);
        g.launch_flags = levels[0]["flags"].as_str().unwrap().to_string();
        g.view_width = 1280.0;
        g.view_height = 800.0;
        g.scale_factor = 1.0;
        g.boot();
        g.start_new_game();
        for want in &levels {
            let l = want["level"].as_i64().unwrap() as i32;
            g.rand_trace(true);
            g.load_level(l);
            let got = g.level_dump();
            g.rand_trace(false);
            let d = differences(&got, want);
            checked += 1;
            if !d.is_empty() {
                failures.push(format!("seed {seed} level {l}:\n    {}", d.iter().take(12).cloned().collect::<Vec<_>>().join("\n    ")));
            }
        }
    }
    assert!(checked >= 20, "only {checked} reference levels found");
    assert!(failures.is_empty(), "{} of {checked} levels differ from their recordings:\n{}", failures.len(), failures.join("\n"));
}
