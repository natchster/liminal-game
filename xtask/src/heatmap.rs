//! `cargo xtask heatmap REC.ocreplay…`: where players went, from recorded
//! sessions (`backrooms --record`; `docs/engineering/REPLAYS.md`), read
//! through `backrooms --dump-replay` so the format is read in one place.
//!
//! For every `(run seed, level)` the recordings visited, the level's tile grid
//! comes from `backrooms --dump-level` (the same run seed builds the same
//! levels) and each telemetry row (one per second of play) adds to its tile:
//! yellow for time spent, red where the director was at a peak. Several
//! recordings (a playtest round) add up into one map per level, written to
//! `target/xtask/heatmap/<seed>_L<level>.png`, with a summary of deaths and
//! peaks per level from the recorded events.
use crate::shots::read_json;
use crate::{cargo, root, run};
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::Command;

pub fn heatmap(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("usage: cargo xtask heatmap RECORDING.ocreplay …".into());
    }
    run(cargo().args(["build", "--release", "-q", "-p", "backrooms"]))?;
    let game = root().join("target/release/backrooms");
    // (seed, level) → tile → (seconds, seconds at a peak)
    let mut visits: BTreeMap<(u64, i64), BTreeMap<(i64, i64), (u32, u32)>> = BTreeMap::new();
    let mut tally: BTreeMap<(u64, i64), BTreeMap<String, u32>> = BTreeMap::new();
    let mut rows_by_level: BTreeMap<(u64, i64), Vec<(f64, f64, bool)>> = BTreeMap::new();
    for f in args {
        let o = Command::new(&game).args(["--dump-replay", f]).output().map_err(|e| e.to_string())?;
        if !o.status.success() {
            return Err(format!("{f}: {}", String::from_utf8_lossy(&o.stderr).trim()));
        }
        let dumped: Value = serde_json::from_slice(&o.stdout).map_err(|e| format!("{f}: {e}"))?;
        let v = &dumped["telemetry"];
        for r in v["rows"].as_array().into_iter().flatten() {
            if r["state"].as_str() != Some("playing") {
                continue;
            }
            let key = (r["seed"].as_u64().unwrap_or(0), r["level"].as_i64().unwrap_or(0));
            rows_by_level.entry(key).or_default().push((
                r["x"].as_f64().unwrap_or(0.0),
                r["z"].as_f64().unwrap_or(0.0),
                r["phase"].as_str() == Some("Peak"),
            ));
        }
        for e in v["events"].as_array().into_iter().flatten() {
            let key = (e["seed"].as_u64().unwrap_or(0), e["level"].as_i64().unwrap_or(0));
            *tally.entry(key).or_default().entry(e["what"].as_str().unwrap_or("?").to_string()).or_default() += 1;
        }
    }
    let out = root().join("target/xtask/heatmap");
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    for ((seed, level), rows) in &rows_by_level {
        let dump_dir = out.join("levels");
        let st = Command::new(&game)
            .args(["--fresh", "--dump-level"])
            .arg(&dump_dir)
            .args(["--seed", &seed.to_string(), "--levels", &level.to_string()])
            .stdout(std::process::Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;
        if !st.success() {
            return Err(format!("could not rebuild seed {seed} level {level}"));
        }
        let dump = read_json(&dump_dir.join(format!("rs_{seed}_L{level}.json")));
        let (w, h) = (dump["grid"]["W"].as_i64().unwrap_or(0), dump["grid"]["H"].as_i64().unwrap_or(0));
        let tiles: Vec<i64> = dump["grid"]["tiles"].as_array().into_iter().flatten().filter_map(|t| t.as_i64()).collect();
        let cell = dump["grid"]["cell"].as_f64().unwrap_or(4.0);
        let tile = |x: f64, z: f64| (((x / cell) + w as f64 / 2.0).floor() as i64, ((z / cell) + h as f64 / 2.0).floor() as i64);
        let v = visits.entry((*seed, *level)).or_default();
        for &(x, z, peak) in rows {
            let e = v.entry(tile(x, z)).or_default();
            e.0 += 1;
            e.1 += u32::from(peak);
        }
        let most = v.values().map(|c| c.0).max().unwrap_or(1).max(1) as f64;
        let s = 6i64;
        let mut px = vec![0u8; (w * s * h * s * 4) as usize];
        for z in 0..h {
            for x in 0..w {
                let wall = tiles.get((z * w + x) as usize).copied().unwrap_or(1) != 0;
                let c = match v.get(&(x, z)) {
                    _ if wall => [22, 22, 22, 255],
                    Some(&(n, peak)) => {
                        let k = (n as f64 / most).sqrt();
                        if peak > 0 {
                            [220, (60.0 + 80.0 * (1.0 - k)) as u8, 40, 255]
                        } else {
                            [(120.0 + 120.0 * k) as u8, (110.0 + 100.0 * k) as u8, 50, 255]
                        }
                    }
                    None => [80, 80, 80, 255],
                };
                for dy in 0..s {
                    for dx in 0..s {
                        let i = (((z * s + dy) * w * s + x * s + dx) * 4) as usize;
                        px[i..i + 4].copy_from_slice(&c);
                    }
                }
            }
        }
        let path = out.join(format!("{seed}_L{level}.png"));
        let f = std::fs::File::create(&path).map_err(|e| e.to_string())?;
        let mut enc = png::Encoder::new(std::io::BufWriter::new(f), (w * s) as u32, (h * s) as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().and_then(|mut wr| wr.write_image_data(&px)).map_err(|e| e.to_string())?;
        let t: Value = serde_json::to_value(tally.get(&(*seed, *level)).cloned().unwrap_or_default()).unwrap_or(Value::Null);
        println!("seed {seed} L{level}: {} s of play on {} tiles; events {t}; {}", rows.len(), v.len(), path.display());
    }
    Ok(())
}
