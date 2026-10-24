//! The Rust half of the diff on its own (the JS half needs node: see
//! tools/placer-diff/README.md): every placer runs without panicking, and a
//! second pass over the same cases in the same process gives the same hashes,
//! so nothing a placer caches changes what a later level gets.
use placers_difftest::{crates, run_crate, Opts};

#[test]
fn every_crate_runs_and_repeats() {
    let opts = Opts { sizes: vec![21, 29], seeds: 3, ..Opts::default() };
    for (id, _) in crates() {
        let mut a = vec![];
        run_crate(id, &opts, &mut a).unwrap();
        let mut b = vec![];
        run_crate(id, &opts, &mut b).unwrap();
        let (a, b) = (String::from_utf8(a).unwrap(), String::from_utf8(b).unwrap());
        assert!(!a.lines().any(|l| l.starts_with("P\t")), "{id}: a placer panicked:\n{}", a.lines().filter(|l| l.starts_with("P\t")).collect::<Vec<_>>().join("\n"));
        let n = a.lines().filter(|l| l.starts_with("C\t")).count();
        assert!(n > 0, "{id}: no cases");
        for (la, lb) in a.lines().zip(b.lines()) { assert_eq!(la, lb, "{id}: second run differs"); }
    }
}
