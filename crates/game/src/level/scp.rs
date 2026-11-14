//! scp — which anomaly (if any) waits on which level of a run.
//!
//! The rules:
//!
//! * a level holds at most one of them
//! * within a run (one run seed) each appears at most once, never again
//! * none on the Building's floors (levels 1–3), and none on the Lobby
//!   (level 4): its emptiness is the point
//! * the plan is a pure function of the run seed, so Continue, restarts and
//!   tools all agree on it without saving anything
//!
//! Nothing here is shown to the player: names in this file are internal.
//! The plan draws from its own mulberry32 stream, never from `rnd()`.

use engine::dmath::DMath;
use std::cell::RefCell;

/// One roster entry. `from` is the first level it may appear on; `weight`
/// biases how early in its window it tends to land.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScpEntry {
    pub key: &'static str,
    pub from: i32,
    pub weight: f64,
}

pub const SCP_ROSTER: [ScpEntry; 4] = [
    ScpEntry { key: "stairwell", from: 5, weight: 1.0 }, // SCP-087  the Stairwell
    ScpEntry { key: "store", from: 5, weight: 1.0 },     // SCP-3008 the Showroom: a store that never closes
    ScpEntry { key: "house", from: 5, weight: 1.0 },     // SCP-1983 the Borrowed House: stolen rooms, shadows that hunt by light
    ScpEntry { key: "loop", from: 5, weight: 1.0 },      // SCP-970  the Loop: a line of rooms that comes back round, wronger
];
pub const SCP_KEYS: [&str; 4] = ["stairwell", "store", "house", "loop"];
/// Every anomaly is met by this level in any run
pub const SCP_HORIZON: i32 = 12;

/// The plan's own mulberry32 (int32 formulation; the same bits as
/// [`crate::base::rng::Mulberry32`]).
struct Mulberry32 {
    a: u32,
}
impl Mulberry32 {
    fn next(&mut self) -> f64 {
        self.a = self.a.wrapping_add(0x6D2B79F5);
        let a = self.a;
        let mut t = (a ^ (a >> 15)).wrapping_mul(1 | a);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }
}

/// A plan: level → anomaly key, entries in insertion order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScpPlan {
    pub entries: Vec<(i32, &'static str)>,
}
impl ScpPlan {
    /// The anomaly on level `n`, if any
    pub fn get(&self, n: i32) -> Option<&'static str> {
        self.entries.iter().find(|e| e.0 == n).map(|e| e.1)
    }
    /// Whether level `n` has an anomaly
    pub fn has(&self, n: i32) -> bool {
        self.entries.iter().any(|e| e.0 == n)
    }
    /// Sets level `n`'s anomaly (an existing level keeps its place in the order)
    pub fn set(&mut self, n: i32, key: &'static str) {
        match self.entries.iter_mut().find(|e| e.0 == n) {
            Some(e) => e.1 = key,
            None => self.entries.push((n, key)),
        }
    }
    /// Number of planned levels
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// The planned levels, in insertion order
    pub fn levels(&self) -> impl Iterator<Item = i32> + '_ {
        self.entries.iter().map(|e| e.0)
    }
}

/// Plan a run.
/// Each key gets one level in [from, horizon]; no two share a level and no
/// two are adjacent, so there is always a plain level to breathe in between.
/// Deterministic in `run_seed`.
pub fn scp_plan(run_seed: u32, roster: &[ScpEntry], horizon: i32) -> ScpPlan {
    let mut rng = Mulberry32 { a: run_seed ^ 0x5C0B1E55 };
    for _attempt in 0..200 {
        let mut order: Vec<&ScpEntry> = roster.iter().collect();
        let mut i = order.len() as i64 - 1;
        while i > 0 {
            let j = (rng.next() * (i + 1) as f64).floor() as usize;
            order.swap(i as usize, j);
            i -= 1;
        }
        let mut taken = ScpPlan::default();
        let mut ok = true;
        for s in &order {
            let mut free: Vec<i32> = vec![];
            let mut n = s.from;
            while n <= horizon {
                if !(taken.has(n) || taken.has(n - 1) || taken.has(n + 1)) {
                    free.push(n);
                }
                n += 1;
            }
            if free.is_empty() {
                ok = false;
                break;
            }
            // earlier levels are a little more likely, scaled by weight
            // (`pow`: exact for the roster's weight 1)
            let fl = free.len();
            let w: Vec<f64> = (0..fl).map(|i| (1.0 - i as f64 / (fl + 1) as f64).dpowf(s.weight)).collect();
            let total = w.iter().fold(0.0, |a, b| a + b);
            let mut r = rng.next() * total;
            let mut pick = free[fl - 1];
            for i in 0..fl {
                r -= w[i];
                if r <= 0.0 {
                    pick = free[i];
                    break;
                }
            }
            taken.set(pick, s.key);
        }
        if ok {
            return taken;
        }
    }
    // unreachable with the default roster; fall back to a fixed spread
    let mut m = ScpPlan::default();
    for (i, s) in roster.iter().enumerate() {
        m.set(s.from.max(2 + i as i32 * 2), s.key);
    }
    m
}

/// `scp_plan(run_seed)` with the default roster and horizon.
pub fn scp_plan_default(run_seed: u32) -> ScpPlan {
    scp_plan(run_seed, &SCP_ROSTER, SCP_HORIZON)
}

thread_local! {
    // the last 16 run seeds' plans, oldest first (the oldest is evicted first)
    static CACHE: RefCell<Vec<(u32, ScpPlan)>> = const { RefCell::new(Vec::new()) };
}

/// The anomaly on level `n` of the run `run_seed`, or None.
pub fn scp_for_level(run_seed: u32, n: i32) -> Option<&'static str> {
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if let Some((_, p)) = c.iter().find(|e| e.0 == run_seed) {
            return p.get(n);
        }
        let plan = scp_plan_default(run_seed);
        let k = plan.get(n);
        c.push((run_seed, plan));
        if c.len() > 16 {
            c.remove(0);
        }
        k
    })
}
