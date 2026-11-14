//! Mesh lint for the embedded meshes (`crates/game/assets/*.bin`), and the
//! parts table that makes a commit's mesh changes readable.
//!
//! Every prop part: indices in range and a whole number of triangles; finite
//! positions inside their quantization bounds; unit normals; at most a quarter
//! of its triangles without area (`DEGENERATE_EXCEPTIONS` lists the known
//! broken ones); the part close to the frame it was modelled in (the same
//! measure `props.py` reports, failing above `MAX_FRAME_DEV`); triangles
//! within the size budget of `propkit.budget_for`, or the one its batch script
//! gives it (×1.25, as the build). Every creature part: indices in range,
//! finite positions, skin weights summing to one over bones that exist.
//!
//! `crates/game/assets/parts.tsv` lists every part (name, frame, vertices,
//! triangles, bounds); the test fails when it is out of date, and
//! `REGEN=1 cargo test --release -p game --test asset_lint` rewrites it, so
//! the diff of a mesh change is a readable table.
use game::assets::creatures::{creature_part_info, creature_part_names, decode_creature};
use game::assets::props::{decode, prop_part_info, prop_part_names};

/// Worst relative deviation from the part's frame (props.py errors above it).
const MAX_FRAME_DEV: f64 = 0.35;
/// Share of zero-area triangles a part may have. Most parts have none; 58
/// have over 1% (slivers left by booleans and bevels: wasted triangles, not
/// visible), and a part this broken is mostly slivers.
const MAX_DEGENERATE: f64 = 0.25;
/// Known broken parts, with why and how to fix them.
const DEGENERATE_EXCEPTIONS: &[(&str, &str)] = &[
    ("Props__extinguisher_carryHandle", "68% slivers: the boolean cut in game/core_placement.py ext_lever() is coplanar with the channel's faces; move the cut box off the profile's planes"),
    ("Props__extinguisher_lever", "the same ext_lever() cut"),
];

/// Parts given a budget of their own in their batch script
/// (`part('Name', …, budget=N)` in `assets/src/blender/props/<theme>/*.py`, or
/// `auto('Name', build, N, …)` in the batches whose parts are placed as
/// modelled: its third argument, before any keyword). A name made in a loop
/// (`auto(f'Real__keysKey{_i}', …)`) is a pattern: what stands for `{…}`
/// matches any letters and digits.
struct ScriptBudgets {
    exact: std::collections::HashMap<String, usize>,
    patterns: Vec<(String, String, usize)>,
}

impl ScriptBudgets {
    fn get(&self, name: &str) -> Option<usize> {
        self.exact.get(name).copied().or_else(|| {
            self.patterns.iter().find_map(|(pre, post, b)| {
                let mid = name.strip_prefix(pre.as_str())?.strip_suffix(post.as_str())?;
                (!mid.is_empty() && mid.chars().all(|c| c.is_ascii_alphanumeric())).then_some(*b)
            })
        })
    }
}

/// The top-level arguments of a call's text after its `(`
fn call_args(rest: &str) -> Vec<String> {
    let (mut out, mut cur, mut depth, mut quote) = (vec![], String::new(), 0i32, None::<char>);
    for c in rest.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => {
                quote = None;
                cur.push(c);
            }
            (Some(_), c) => cur.push(c),
            (None, '\'' | '"') => {
                quote = Some(c);
                cur.push(c);
            }
            (None, '(' | '[' | '{') => {
                depth += 1;
                cur.push(c);
            }
            (None, ')' | ']' | '}') if depth == 0 => break,
            (None, ')' | ']' | '}') => {
                depth -= 1;
                cur.push(c);
            }
            (None, ',') if depth == 0 => out.push(std::mem::take(&mut cur).trim().to_string()),
            (None, c) => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

fn script_budgets() -> ScriptBudgets {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/src/blender/props");
    let mut out = ScriptBudgets { exact: Default::default(), patterns: vec![] };
    // every family script, in its theme's folder (props/<theme>/<family>.py)
    let mut scripts = vec![];
    let mut stack = vec![dir];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "py") {
                scripts.push(p);
            }
        }
    }
    assert!(scripts.len() > 60, "only {} family scripts under assets/src/blender/props", scripts.len());
    for path in scripts {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        for line in text.lines().filter(|l| l.trim_start().starts_with("part('")) {
            let name = line.trim_start()[6..].split('\'').next().unwrap_or("");
            if let Some(b) =
                line.split("budget=").nth(1).and_then(|r| r.split(|c: char| !c.is_ascii_digit()).next()).and_then(|n| n.parse().ok())
            {
                out.exact.insert(name.to_string(), b);
            }
        }
        for line in text.lines() {
            let l = line.trim_start();
            let Some(rest) = l.strip_prefix("auto(") else { continue };
            let args = call_args(rest);
            // a keyword argument is a bare name then `=` (a lambda's default is not one)
            let keyword = |a: &str| {
                let n = a.split('=').next().unwrap_or("").trim();
                a.contains('=') && !n.is_empty() && !n.contains(' ') && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            };
            let positional: Vec<&String> = args.iter().filter(|a| !keyword(a)).collect();
            let (Some(name), Some(b)) = (positional.first(), positional.get(2).and_then(|b| b.parse::<usize>().ok())) else { continue };
            let templated = name.starts_with("f'") || name.starts_with("f\"");
            let name = name.trim_start_matches('f').trim_matches(|c| c == '\'' || c == '"');
            match (templated, name.find('{'), name.rfind('}')) {
                (true, Some(i), Some(j)) => out.patterns.push((name[..i].to_string(), name[j + 1..].to_string(), b)),
                _ => {
                    out.exact.insert(name.to_string(), b);
                }
            }
        }
    }
    out
}

fn budget_for(fmin: [f64; 3], fmax: [f64; 3]) -> usize {
    let d = ((fmax[0] - fmin[0]).powi(2) + (fmax[1] - fmin[1]).powi(2) + (fmax[2] - fmin[2]).powi(2)).sqrt();
    match d {
        d if d < 0.03 => 100,
        d if d < 0.12 => 280,
        d if d < 0.4 => 560,
        d if d < 1.2 => 1100,
        _ => 1800,
    }
}

/// props.py's `check_frame`: how far the part strays outside / falls short of
/// its frame, relative to the frame's size on that axis.
fn frame_dev(min: [f64; 3], max: [f64; 3], fmin: [f64; 3], fmax: [f64; 3]) -> f64 {
    let big = (0..3).map(|k| fmax[k] - fmin[k]).fold(0.0, f64::max);
    let mut worst: f64 = 0.0;
    for k in 0..3 {
        let ext = (fmax[k] - fmin[k]).max(0.25 * big);
        if ext < 1e-9 {
            continue;
        }
        worst = worst.max((min[k] - fmin[k]).abs() / ext).max((max[k] - fmax[k]).abs() / ext);
    }
    worst
}

fn table() -> String {
    let mut s = String::from("# kind\tname\tframe\tvertices\ttriangles\tmin\tmax\n");
    let f = |a: [f64; 3]| format!("{:.4},{:.4},{:.4}", a[0], a[1], a[2]);
    for name in prop_part_names() {
        let p = prop_part_info(name).unwrap();
        s.push_str(&format!("prop\t{name}\t{}\t{}\t{}\t{}\t{}\n", p.frame.as_str(), p.n, p.t, f(p.min), f(p.max)));
    }
    for name in creature_part_names() {
        let c = creature_part_info(name).unwrap();
        s.push_str(&format!(
            "creature\t{name}\t{}\t{}\t{}\t{}\t{}\n",
            if c.skinned { "skinned" } else { "rigid" },
            c.n,
            c.t,
            f(c.min),
            f(c.max)
        ));
    }
    s
}

#[test]
fn the_parts_table_is_up_to_date() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/parts.tsv");
    let now = table();
    if std::env::var_os("REGEN").is_some() {
        std::fs::write(&path, &now).unwrap();
        return;
    }
    let was = std::fs::read_to_string(&path).unwrap_or_default();
    if was != now {
        let changed: Vec<&str> = now.lines().filter(|l| !was.contains(*l)).take(10).collect();
        panic!(
            "crates/game/assets/parts.tsv is out of date ({} lines differ, e.g. {changed:?}); after an intended mesh change run `REGEN=1 cargo test --release -p game --test asset_lint` and commit the table with the meshes",
            now.lines().filter(|l| !was.contains(*l)).count()
        );
    }
}

#[test]
fn prop_parts_are_sound() {
    let mut bad = vec![];
    let mut over_budget = vec![];
    let budgets = script_budgets();
    let mut worst_degenerate: Vec<(f64, &str)> = vec![];
    for name in prop_part_names() {
        let p = prop_part_info(name).unwrap();
        let d = decode(p);
        let n = d.pos.len() / 3;
        if n != p.n || d.idx.len() != p.t * 3 {
            bad.push(format!("{name}: {n} vertices / {} indices, header says {} / {}", d.idx.len(), p.n, p.t * 3));
            continue;
        }
        if let Some(i) = d.idx.iter().find(|&&i| i as usize >= n) {
            bad.push(format!("{name}: index {i} past {n} vertices"));
        }
        let eps = 1e-3 * (0..3).map(|k| p.max[k] - p.min[k]).fold(1e-6, f64::max);
        for (j, c) in d.pos.chunks(3).enumerate() {
            let out = (0..3).any(|k| !c[k].is_finite() || (c[k] as f64) < p.min[k] - eps || (c[k] as f64) > p.max[k] + eps);
            if out {
                bad.push(format!("{name}: vertex {j} {c:?} outside its bounds"));
                break;
            }
        }
        if let Some(j) = d.nrm.chunks(3).position(|c| ((c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt() - 1.0).abs() > 0.03) {
            bad.push(format!("{name}: normal {j} is not unit length"));
        }
        let degenerate = d
            .idx
            .chunks(3)
            .filter(|t| {
                let v = |i: u16| [d.pos[i as usize * 3] as f64, d.pos[i as usize * 3 + 1] as f64, d.pos[i as usize * 3 + 2] as f64];
                let (a, b, c) = (v(t[0]), v(t[1]), v(t[2]));
                let (u, w) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
                let x = [u[1] * w[2] - u[2] * w[1], u[2] * w[0] - u[0] * w[2], u[0] * w[1] - u[1] * w[0]];
                (x[0] * x[0] + x[1] * x[1] + x[2] * x[2]).sqrt() < 1e-12
            })
            .count();
        worst_degenerate.push((degenerate as f64 / p.t.max(1) as f64, name));
        let known = DEGENERATE_EXCEPTIONS.iter().any(|(n, _)| n == name);
        if degenerate as f64 > MAX_DEGENERATE * p.t as f64 && degenerate > 2 && !known {
            bad.push(format!("{name}: {degenerate} of {} triangles have no area", p.t));
        }
        let dev = frame_dev(p.min, p.max, p.fmin, p.fmax);
        if dev > MAX_FRAME_DEV {
            bad.push(format!("{name}: {:.0}% away from its frame (wrong frame or orientation?)", dev * 100.0));
        }
        let budget = budgets.get(name).unwrap_or_else(|| budget_for(p.fmin, p.fmax)) * 5 / 4;
        if p.t > budget {
            over_budget.push(format!("(\"{name}\", {}),", p.t));
        }
    }
    worst_degenerate.sort_by(|a, b| b.0.total_cmp(&a.0));
    let share = |f: f64| worst_degenerate.iter().filter(|d| d.0 > f).count();
    eprintln!(
        "{} prop parts; {} over budget; zero-area triangles: {} parts over 1%, {} over 5%, {} over 10%, worst {:?}",
        prop_part_names().len(),
        over_budget.len(),
        share(0.01),
        share(0.05),
        share(0.10),
        &worst_degenerate[..5.min(worst_degenerate.len())]
    );
    for l in &over_budget {
        eprintln!("    {l}");
    }
    assert!(bad.is_empty(), "prop parts with problems: {bad:#?}");
    assert!(
        over_budget.is_empty(),
        "prop parts over their triangle budget (lighten them, or add a deliberate exception to BUDGET_EXCEPTIONS): {over_budget:#?}"
    );
}

#[test]
fn creature_parts_are_sound() {
    let mut bad = vec![];
    for name in creature_part_names() {
        let c = creature_part_info(name).unwrap();
        let g = decode_creature(c);
        let n = g.position.len() / 3;
        if g.position.iter().any(|v| !v.is_finite()) {
            bad.push(format!("{name}: a position is not finite"));
        }
        if let Some(i) = g.index.as_ref().and_then(|ix| ix.iter().find(|&&i| i as usize >= n)) {
            bad.push(format!("{name}: index {i} past {n} vertices"));
        }
        if c.skinned {
            for (v, w) in g.skin_weight.chunks(4).enumerate() {
                let s: f32 = w.iter().sum();
                if (s - 1.0).abs() > 0.02 {
                    bad.push(format!("{name}: vertex {v} skin weights sum to {s:.3}"));
                    break;
                }
            }
            if let Some(j) = g.skin_index.iter().find(|&&j| j as usize >= c.bones.len().max(1)) {
                bad.push(format!("{name}: bone slot {j} past its {} bones", c.bones.len()));
            }
        }
    }
    assert!(bad.is_empty(), "creature parts with problems: {bad:#?}");
}
