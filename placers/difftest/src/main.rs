//! `placer-diff <crate|--list> [--seeds N] [--sizes 21,29,37] [--only a,b] [--detail i,j] [--no-uses] [--levels FILE]`
//! Prints the case protocol (see tools/placer-diff/README.md) on stdout.
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = placers_difftest::Opts::default();
    let mut krate = None;
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        let mut val = || { i += 1; args.get(i).cloned().unwrap_or_default() };
        match a {
            "--list" => {
                for (id, reg) in placers_difftest::crates() {
                    let mut l = vec![];
                    reg(&mut l);
                    println!("{}\t{}", id, l.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(","));
                }
                return;
            }
            "--seeds" => opts.seeds = val().parse().expect("--seeds N"),
            "--sizes" => opts.sizes = val().split(',').filter(|s| !s.is_empty()).map(|s| s.parse().expect("--sizes a,b")).collect(),
            "--only" => opts.only = Some(val().split(',').map(|s| s.to_string()).collect()),
            "--detail" => opts.detail = val().split(',').filter(|s| !s.is_empty()).map(|s| s.parse().expect("--detail i,j")).collect(),
            "--no-uses" => opts.uses = false,
            "--levels" => opts.levels_file = Some(val()),
            _ => krate = Some(a.to_string()),
        }
        i += 1;
    }
    let krate = krate.expect("usage: placer-diff <crate> [--seeds N] [--sizes 21,29,37] [--only a,b] [--detail i,j] [--no-uses]");
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::with_capacity(1 << 20, stdout.lock());
    placers_difftest::run_crate(&krate, &opts, &mut out).expect("write");
    out.flush().unwrap();
}
