//! `cargo xtask playtest visual` (`docs/VISUAL_PLAYTEST.md`): the visual
//! playtest of an area, a level or a set of levels on one or more runs. It
//! builds the game, runs `backrooms --tour` for each seed (its renders, its
//! detectors and its contact sheets), and writes one report of what they
//! found, the flagged first: `target/xtask/playtest/visual/report.md` and
//! `findings.tsv` (every finding of every floor, with its seed and level).
//! Run by hand, never by `cargo xtask check`.
use crate::{cargo, flag, opt, root, run};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

pub const USAGE: &str = "usage: cargo xtask playtest visual [--seeds 4242,7] [--levels 1,2,3] [--budget 180]
                              [--area x0,z0,x1,z1] [--size 640x400] [--out DIR] [--no-build]";

/// A view's row of `views.tsv`
#[derive(Clone, Debug, Default)]
struct View {
    name: String,
    look: String,
    edge: String,
    shots_at: String,
    background: usize,
    gaps: usize,
    flicker: usize,
}

/// A row of `findings.tsv`
#[derive(Clone, Debug, Default)]
struct Finding {
    kind: String,
    label: String,
    mesh: String,
    views: usize,
    view: String,
    at: String,
    size: f64,
    flagged: bool,
}

fn table(path: &Path) -> Result<Vec<BTreeMap<String, String>>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut lines = text.lines();
    let head: Vec<String> = lines.next().unwrap_or_default().split('\t').map(String::from).collect();
    Ok(lines.map(|l| head.iter().cloned().zip(l.split('\t').map(String::from)).collect()).collect())
}

fn views(path: &Path) -> Result<Vec<View>, String> {
    let n = |r: &BTreeMap<String, String>, k: &str| r.get(k).and_then(|v| v.parse().ok()).unwrap_or(0);
    Ok(table(path)?
        .iter()
        .map(|r| View {
            name: r["name"].clone(),
            look: r["look"].clone(),
            edge: r["edge"].clone(),
            shots_at: r["shots_at"].clone(),
            background: n(r, "background"),
            gaps: n(r, "gaps"),
            flicker: n(r, "flicker"),
        })
        .collect())
}

fn findings(path: &Path) -> Result<Vec<Finding>, String> {
    Ok(table(path)?
        .iter()
        .map(|r| Finding {
            kind: r["kind"].clone(),
            label: r["label"].clone(),
            mesh: r["mesh"].clone(),
            views: r["views"].parse().unwrap_or(0),
            view: r["view"].clone(),
            at: format!("{},{},{}", r["x"], r["y"], r["z"]),
            size: r["size"].parse().unwrap_or(0.0),
            flagged: r["flagged"] == "true",
        })
        .collect())
}

pub fn playtest(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("visual") => visual(&args[1..]),
        _ => Err(USAGE.into()),
    }
}

fn visual(args: &[String]) -> Result<(), String> {
    let seeds: Vec<String> = opt(args, "--seeds").unwrap_or("4242").split(',').map(String::from).collect();
    let levels = opt(args, "--levels").unwrap_or("1,2,3");
    let budget = opt(args, "--budget").unwrap_or("180");
    let (w, h) = opt(args, "--size").and_then(|s| s.split_once('x')).unwrap_or(("640", "400"));
    let out = opt(args, "--out").map(PathBuf::from).unwrap_or_else(|| root().join("target/xtask/playtest/visual"));
    if !flag(args, "--no-build") {
        run(cargo().args(["build", "--release", "-p", "backrooms"]))?;
    }
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let t0 = std::time::Instant::now();
    for seed in &seeds {
        let dir = out.join(format!("s{seed}"));
        let mut c = std::process::Command::new(root().join("target/release/backrooms"));
        c.args(["--tour"]).arg(&dir).args(["--seed", seed, "--levels", levels, "--tour-budget", budget, "--width", w, "--height", h]);
        if let Some(a) = opt(args, "--area") {
            c.args(["--tour-area", a]);
        }
        let o = c.output().map_err(|e| e.to_string())?;
        print!("{}", String::from_utf8_lossy(&o.stdout));
        if !o.status.success() {
            return Err(format!("backrooms --tour on seed {seed}: {}", String::from_utf8_lossy(&o.stderr).trim()));
        }
    }
    let report = report(&out, &seeds, levels, budget, args)?;
    std::fs::write(out.join("report.md"), report).map_err(|e| e.to_string())?;
    println!("wrote {} ({:.0} s)", out.join("report.md").display(), t0.elapsed().as_secs_f64());
    Ok(())
}

/// The report: totals, then each floor's gaps, flicker and things, the
/// flagged first, each with its view, its camera and its pictures
fn report(out: &Path, seeds: &[String], levels: &str, budget: &str, args: &[String]) -> Result<String, String> {
    let mut md = String::new();
    let mut all = String::from("seed\tlevel\tkind\tlabel\tmesh\tviews\tview\tat\tsize\tflagged\n");
    writeln!(md, "# Visual playtest\n").unwrap();
    writeln!(
        md,
        "`cargo xtask playtest visual {}`: seeds {}, levels {levels}, {budget} views a floor at most. What to do with it: \
         `.claude/skills/visual-playtest/SKILL.md`; what each detector means: `docs/VISUAL_PLAYTEST.md`. Detectors find; \
         a person (or the `visual-inspector`) judges: reproduce a finding with `--shots-at` before fixing it.\n",
        args.join(" "),
        seeds.join(", ")
    )
    .unwrap();
    writeln!(md, "| Seed | Level | Views | Background | Gaps | Flicker | Clip | Body | Plain (flagged) | Coplanar (flagged / alike) |")
        .unwrap();
    writeln!(md, "|---|---|---|---|---|---|---|---|---|---|").unwrap();
    let mut floors = vec![];
    for seed in seeds {
        let dir = out.join(format!("s{seed}"));
        let mut ls: Vec<(i32, PathBuf)> = std::fs::read_dir(&dir)
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().to_str().and_then(|n| n.strip_prefix('L')).and_then(|n| n.parse().ok()).map(|l| (l, e.path())))
            .collect();
        ls.sort();
        for (l, ldir) in ls {
            let vs = views(&ldir.join("views.tsv"))?;
            let fs = findings(&ldir.join("findings.tsv"))?;
            let flagged = |k: &str| fs.iter().filter(|f| f.kind == k && f.flagged).count();
            let gaps = vs.iter().filter(|v| v.gaps >= FLAG_PX).count();
            let flicker = vs.iter().filter(|v| v.flicker >= FLAG_PX).count();
            writeln!(
                md,
                "| {seed} | {l} | {} | {} | {gaps} | {flicker} | {} | {} | {} | {} / {} |",
                vs.len(),
                vs.iter().filter(|v| v.background > 0).count(),
                flagged("clip"),
                flagged("body"),
                flagged("plain") + flagged("unlit") + flagged("glow"),
                flagged("coplanar"),
                fs.iter().filter(|f| f.kind == "coplanar_alike").count()
            )
            .unwrap();
            for f in &fs {
                writeln!(
                    all,
                    "{seed}\t{l}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.4}\t{}",
                    f.kind, f.label, f.mesh, f.views, f.view, f.at, f.size, f.flagged
                )
                .unwrap();
            }
            floors.push((seed.clone(), l, ldir, vs, fs));
        }
    }
    std::fs::write(out.join("findings.tsv"), all).map_err(|e| e.to_string())?;
    let rel = |p: &Path| p.strip_prefix(out).map(|r| r.display().to_string()).unwrap_or_else(|_| p.display().to_string());
    for (seed, l, ldir, vs, fs) in &floors {
        writeln!(md, "\n## Seed {seed}, level {l}\n").unwrap();
        let cam = |name: &str| vs.iter().find(|v| v.name == name).map(|v| v.shots_at.clone()).unwrap_or_default();
        let mut sheets: Vec<String> = std::fs::read_dir(ldir)
            .map(|d| {
                d.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).filter(|n| n.starts_with("sheet_")).collect()
            })
            .unwrap_or_default();
        sheets.sort();
        if !sheets.is_empty() {
            writeln!(md, "Sheets: {}\n", sheets.iter().map(|s| format!("`{}`", rel(&ldir.join(s)))).collect::<Vec<_>>().join(", "))
                .unwrap();
        }
        // the pixel detectors, the largest first
        for (what, key) in [("Gaps", 0), ("Flicker", 1)] {
            let mut list: Vec<&View> = vs.iter().filter(|v| if key == 0 { v.gaps >= FLAG_PX } else { v.flicker >= FLAG_PX }).collect();
            list.sort_by_key(|v| std::cmp::Reverse(if key == 0 { v.gaps } else { v.flicker }));
            writeln!(md, "### {what}: {} views\n", list.len()).unwrap();
            for v in list.iter().take(12) {
                let px = if key == 0 { v.gaps } else { v.flicker };
                let pic = ldir.join(format!("{}_{}.png", v.name, if key == 0 { "gaps" } else { "flicker" }));
                writeln!(md, "- `{}` ({}, {} edge): {px} px; `{}`; `--shots-at \"{}\"`", v.name, v.look, v.edge, rel(&pic), v.shots_at)
                    .unwrap();
            }
            if list.len() > 12 {
                writeln!(md, "- … and {} more in `views.tsv`", list.len() - 12).unwrap();
            }
            writeln!(md).unwrap();
        }
        // the things, flagged, by kind
        for kind in ["clip", "body", "plain", "unlit", "glow", "coplanar"] {
            let list: Vec<&Finding> = fs.iter().filter(|f| f.kind == kind && f.flagged).collect();
            if list.is_empty() {
                continue;
            }
            writeln!(md, "### {kind}: {} things\n", list.len()).unwrap();
            for f in list.iter().take(12) {
                let size = match kind {
                    "clip" | "body" => format!("{:.2} m inside", f.size),
                    "coplanar" => format!("{:.4} m² overlap", f.size),
                    _ => format!("{:.0}% of the frame", f.size * 100.0),
                };
                let seen = if f.view.is_empty() { "no view sees it".to_string() } else { format!("`--shots-at \"{}\"`", cam(&f.view)) };
                writeln!(md, "- {} `{}` ({size}, {} views, first `{}`): {seen}; at {}", f.label, f.mesh, f.views, f.view, f.at).unwrap();
            }
            if list.len() > 12 {
                writeln!(md, "- … and {} more in `findings.tsv`", list.len() - 12).unwrap();
            }
            writeln!(md).unwrap();
        }
        let listed = fs.iter().filter(|f| !f.flagged).count();
        if listed > 0 {
            writeln!(
                md,
                "Listed and not flagged (a plain thing one view saw, coplanar faces of one material): {listed}, in `findings.tsv`."
            )
            .unwrap();
        }
    }
    Ok(md)
}

/// A view is flagged for gaps or flicker at this many pixels (as
/// `game::debug::sight::FLAG_PX`)
const FLAG_PX: usize = 12;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_ranks_the_flagged_and_keeps_every_finding() {
        let out = std::env::temp_dir().join(format!("xtask_playtest_{}", std::process::id()));
        let l = out.join("s7/L2");
        std::fs::create_dir_all(&l).unwrap();
        std::fs::write(
            l.join("views.tsv"),
            "name\tlook\tedge\tcorner\tshots_at\tbackground\tgaps\tgap_blob\tflicker\tflicker_blob\tclip\tbody\tplain\n\
             L2_000_at\tat\twall\tfalse\tL2_000_at:1,2,3,0,0\t40\t30\t30\t0\t0\t0\t0\t0\n\
             L2_001_up\tup\tvoid\tfalse\tL2_001_up:4,5,6,0,1\t900\t500\t500\t20\t20\t1\t0\t0\n\
             L2_002_at\tat\twall\tfalse\tL2_002_at:7,8,9,0,0\t0\t0\t0\t3\t3\t0\t0\t1\n",
        )
        .unwrap();
        std::fs::write(
            l.join("findings.tsv"),
            "kind\tlabel\tmesh\tviews\tview\tx\ty\tz\tsize\tflagged\n\
             clip\tlevel\tbox 4.00x0.13x0.26 #ffffff map #14\t1\tL2_001_up\t1.0\t2.0\t3.0\t0.0611\ttrue\n\
             plain\tcore_locker\tbox #2b2f36\t1\tL2_002_at\t7.0\t8.0\t9.0\t0.2000\tfalse\n",
        )
        .unwrap();
        let md = report(&out, &["7".to_string()], "2", "60", &[]).unwrap();
        assert!(md.contains("| 7 | 2 | 3 | 2 | 2 | 1 | 1 | 0 | 0 | 0 / 0 |"), "{md}");
        // the larger gap first, with its camera to reproduce it
        let (a, b) = (md.find("`L2_001_up` (up, void edge): 500 px").unwrap(), md.find("`L2_000_at` (at, wall edge): 30 px").unwrap());
        assert!(a < b);
        assert!(md.contains("--shots-at \"L2_001_up:4,5,6,0,1\""), "{md}");
        assert!(md.contains("0.06 m inside"), "{md}");
        assert!(md.contains("Listed and not flagged"), "{md}");
        let all = std::fs::read_to_string(out.join("findings.tsv")).unwrap();
        assert_eq!(all.lines().count(), 3, "{all}");
        assert!(all.contains("7\t2\tplain\tcore_locker"));
        let _ = std::fs::remove_dir_all(&out);
    }
}
