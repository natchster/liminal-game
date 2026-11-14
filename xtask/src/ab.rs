//! `cargo xtask ab`: blind A/B reviews of frames, run so that the judge
//! cannot know which side is new, and scored so that its order bias cancels.
//!
//! ```text
//! cargo xtask ab prepare --before DIR --after DIR [--names a,b,…] [--question "…"]
//! cargo xtask ab score ANSWERS.json [--reviewer NAME]
//! ```
//!
//! `prepare` pairs every PNG present in both folders (same file name) and
//! writes each pair **twice**, once in each order, as one side-by-side image
//! (`A` left, `B` right) under shuffled names in `target/xtask/ab/pairs/`, with
//! `questions.md` for the reviewer. The key (which side is the *after* image)
//! goes to `target/xtask/ab/.key.json`: give the reviewer only the pairs
//! folder and the questions, never the key or the folder names.
//!
//! `score` reads the reviewer's answers (`{"pair03": "A", "pair07": "tie", …}`)
//! and, per image, combines its two orders: both for *after* → after; both
//! for *before* → before; anything else → tie (a judge that flips with the
//! order has no preference). The verdicts are appended to
//! `crates/backrooms/tests/review/verdicts.jsonl`, the record that
//! calibrates reviewers against people (roadmap 4.5).
use crate::shots::{read_json, read_png, write_json};
use crate::{opt, root};
use serde_json::{Value, json};
use std::io::Write;
use std::path::{Path, PathBuf};

fn dir() -> PathBuf {
    root().join("target/xtask/ab")
}

const DEFAULT_QUESTION: &str = "Which side looks more like a real, worn building photographed in weak fluorescent light, and less like a game or a toy? Look at the materials, the light and the colour.";

pub fn ab(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("prepare") => prepare(&args[1..]),
        Some("score") => score(&args[1..]),
        _ => Err("usage: cargo xtask ab prepare --before DIR --after DIR [--names a,b] [--question \"…\"] | ab score ANSWERS.json [--reviewer NAME]".into()),
    }
}

fn prepare(args: &[String]) -> Result<(), String> {
    let before = PathBuf::from(opt(args, "--before").ok_or("--before DIR")?);
    let after = PathBuf::from(opt(args, "--after").ok_or("--after DIR")?);
    let question = opt(args, "--question").unwrap_or(DEFAULT_QUESTION);
    let only: Option<Vec<&str>> = opt(args, "--names").map(|s| s.split(',').collect());
    let mut names: Vec<String> = std::fs::read_dir(&before)
        .map_err(|e| format!("{}: {e}", before.display()))?
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .filter(|n| n.ends_with(".png") && after.join(n).exists())
        .filter(|n| only.as_ref().is_none_or(|o| o.contains(&n.trim_end_matches(".png"))))
        .collect();
    names.sort();
    if names.is_empty() {
        return Err("no PNG with the same name in both folders".into());
    }
    // how much each pair differs, for you and not the reviewer: a pair that
    // hardly differs can only tie, and reads as a verdict it is not (the
    // Building's metalwork worn, VP10: five ties of seven where the wear did
    // not show in its light)
    let mut near = vec![];
    for n in &names {
        let (share, mean) = difference(&before.join(n), &after.join(n))?;
        let flag = if share < NEAR_IDENTICAL { "  near-identical: it can only tie" } else { "" };
        println!("{:<32} {:>6.2}% of pixels changed, mean {:.4}{flag}", n.trim_end_matches(".png"), share * 100.0, mean);
        if share < NEAR_IDENTICAL {
            near.push(n.trim_end_matches(".png").to_string());
        }
    }
    if !near.is_empty() {
        println!(
            "warning: {} of {} images hardly differ ({}): choose cameras where the change shows, or say so with the verdict",
            near.len(),
            names.len(),
            near.join(", ")
        );
    }
    let out = dir();
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(out.join("pairs")).map_err(|e| e.to_string())?;
    // both orders of every image, shuffled by a seed from the clock (the
    // reviewer must not be able to guess the order from the names)
    let mut jobs: Vec<(String, bool)> = names.iter().flat_map(|n| [(n.clone(), true), (n.clone(), false)]).collect();
    let mut s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1) | 1;
    for i in (1..jobs.len()).rev() {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        jobs.swap(i, (s % (i as u64 + 1)) as usize);
    }
    let mut key = serde_json::Map::new();
    for (k, (name, after_left)) in jobs.iter().enumerate() {
        let pair = format!("pair{:02}", k + 1);
        let (l, r) = if *after_left { (after.join(name), before.join(name)) } else { (before.join(name), after.join(name)) };
        side_by_side(&l, &r, &out.join("pairs").join(format!("{pair}.png")))?;
        key.insert(pair, json!({ "image": name.trim_end_matches(".png"), "after": if *after_left { "A" } else { "B" } }));
    }
    write_json(&out.join(".key.json"), &json!({ "question": question, "pairs": key }))?;
    let mut q = String::from(
        "# Blind review\n\nEach image in this folder shows two frames side by side: **A** on the left, **B** on the right.\n\n",
    );
    q.push_str(&format!("**Question:** {question}\n\n"));
    q.push_str("For each pair, first write what you see on each side (one or two sentences of evidence), then answer `A`, `B` or `tie`. Answer every pair independently; some pairs repeat. Reply with JSON at the end: `{\"pair01\": \"A\", …}`.\n\nPairs: ");
    q.push_str(&(1..=jobs.len()).map(|k| format!("pair{k:02}")).collect::<Vec<_>>().join(", "));
    q.push('\n');
    std::fs::write(out.join("pairs/questions.md"), q).map_err(|e| e.to_string())?;
    println!("{} pairs ({} images × 2 orders) in target/xtask/ab/pairs/; give a fresh reviewer only that folder", jobs.len(), names.len());
    Ok(())
}

fn score(args: &[String]) -> Result<(), String> {
    let path = args.first().ok_or("ab score ANSWERS.json")?;
    let reviewer = opt(args, "--reviewer").unwrap_or("agent");
    let answers = read_json(Path::new(path));
    let key = read_json(&dir().join(".key.json"));
    let pairs = key["pairs"].as_object().ok_or("no key: run `cargo xtask ab prepare` first")?;
    let mut by_image: std::collections::BTreeMap<String, Vec<&str>> = Default::default();
    for (pair, k) in pairs {
        let a = answers[pair].as_str().unwrap_or("tie");
        let verdict = match a {
            "tie" => "tie",
            side if side == k["after"].as_str().unwrap_or("") => "after",
            "A" | "B" => "before",
            _ => "tie",
        };
        by_image.entry(k["image"].as_str().unwrap_or("?").to_string()).or_default().push(verdict);
    }
    let log = root().join("crates/backrooms/tests/review/verdicts.jsonl");
    std::fs::create_dir_all(log.parent().unwrap()).map_err(|e| e.to_string())?;
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&log).map_err(|e| e.to_string())?;
    let date = std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let (mut after, mut before, mut tie) = (0, 0, 0);
    for (image, v) in &by_image {
        let combined = if v.iter().all(|x| *x == "after") {
            after += 1;
            "after"
        } else if v.iter().all(|x| *x == "before") {
            before += 1;
            "before"
        } else {
            tie += 1;
            "tie"
        };
        println!("{image:<32} {combined:<7} ({})", v.join(", "));
        let line: Value =
            json!({ "date": date, "reviewer": reviewer, "question": key["question"], "image": image, "verdict": combined, "orders": v });
        writeln!(f, "{line}").map_err(|e| e.to_string())?;
    }
    println!("after preferred {after}, before preferred {before}, tie {tie} — logged to crates/backrooms/tests/review/verdicts.jsonl");
    Ok(())
}

/// A pixel is changed when a channel moved by more than this (0-255): above
/// the film grain, which differs between runs of `--shots-at`
const CHANGED: u32 = 12;
/// Under this share of changed pixels a pair is near-identical
const NEAR_IDENTICAL: f64 = 0.005;

/// How much two frames differ: the share of pixels changed (`CHANGED`) and
/// the mean change of the most changed channel (0-1). Frames of different
/// sizes differ wholly.
fn difference(a: &Path, b: &Path) -> Result<(f64, f64), String> {
    let (aw, ah, ap) = read_png(a)?;
    let (bw, bh, bp) = read_png(b)?;
    if (aw, ah) != (bw, bh) {
        return Ok((1.0, 1.0));
    }
    let n = (aw * ah) as usize;
    let (mut changed, mut sum) = (0usize, 0.0f64);
    for i in 0..n {
        let d = (0..3).map(|c| (ap[i * 4 + c] as i32 - bp[i * 4 + c] as i32).unsigned_abs()).max().unwrap_or(0);
        changed += usize::from(d > CHANGED);
        sum += d as f64 / 255.0;
    }
    Ok((changed as f64 / n.max(1) as f64, sum / n.max(1) as f64))
}

/// Two frames side by side with a 12 px gap and an A / B mark over each.
fn side_by_side(l: &Path, r: &Path, out: &Path) -> Result<(), String> {
    let (lw, lh, lp) = read_png(l)?;
    let (rw, rh, rp) = read_png(r)?;
    let (gap, top) = (12u32, 24u32);
    let (w, h) = (lw + gap + rw, lh.max(rh) + top);
    let mut px = vec![24u8; (w * h * 4) as usize];
    let mut put = |x: u32, y: u32, c: [u8; 4]| {
        if x < w && y < h {
            let i = ((y * w + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&c);
        }
    };
    for (ox, (iw, ih, ip)) in [(0, (lw, lh, &lp)), (lw + gap, (rw, rh, &rp))] {
        for y in 0..ih {
            for x in 0..iw {
                let i = ((y * iw + x) * 4) as usize;
                put(ox + x, top + y, [ip[i], ip[i + 1], ip[i + 2], 255]);
            }
        }
    }
    // 5×7 glyphs, scaled ×2
    const A: [u8; 7] = [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001];
    const B: [u8; 7] = [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110];
    for (ox, glyph) in [(lw / 2 - 5, A), (lw + gap + rw / 2 - 5, B)] {
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    for dy in 0..2 {
                        for dx in 0..2 {
                            put(ox + col * 2 + dx, 4 + row as u32 * 2 + dy, [255, 255, 255, 255]);
                        }
                    }
                }
            }
        }
    }
    let f = std::fs::File::create(out).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().and_then(|mut wr| wr.write_image_data(&px)).map_err(|e| e.to_string())
}
