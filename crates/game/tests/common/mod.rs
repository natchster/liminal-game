//! Readers for the recorded reference data (binary fixtures in `tests/fixtures/`).
#![allow(dead_code, reason = "helpers shared by several test binaries; each uses a part")]
use std::collections::HashMap;

pub fn fixture(name: &str) -> Vec<u8> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    std::fs::read(&p).unwrap_or_else(|e| panic!("{}: {} (missing fixture)", p.display(), e))
}
pub fn fixture_text(name: &str) -> String {
    String::from_utf8(fixture(name)).unwrap()
}

pub struct Rd<'a> {
    pub b: &'a [u8],
    pub p: usize,
}
impl<'a> Rd<'a> {
    pub fn new(b: &'a [u8]) -> Self {
        Rd { b, p: 0 }
    }
    pub fn bytes(&mut self, n: usize) -> &'a [u8] {
        let s = &self.b[self.p..self.p + n];
        self.p += n;
        s
    }
    pub fn u8(&mut self) -> u8 {
        self.bytes(1)[0]
    }
    pub fn u16(&mut self) -> u16 {
        let s = self.bytes(2);
        u16::from_le_bytes([s[0], s[1]])
    }
    pub fn u32(&mut self) -> u32 {
        let s = self.bytes(4);
        u32::from_le_bytes(s.try_into().unwrap())
    }
    pub fn f64(&mut self) -> f64 {
        let s = self.bytes(8);
        f64::from_le_bytes(s.try_into().unwrap())
    }
    pub fn str(&mut self) -> String {
        let n = self.u16() as usize;
        String::from_utf8(self.bytes(n).to_vec()).unwrap()
    }
    pub fn done(&self) -> bool {
        self.p >= self.b.len()
    }
}

/// One sampled array of a case.
#[derive(Debug, Clone)]
pub struct Record {
    pub label: String,
    pub ty: u8,
    pub item_size: usize,
    pub stride: usize,
    pub items: usize,
    pub vals: Vec<f64>,
}
impl Record {
    pub fn read(r: &mut Rd) -> Record {
        let label = r.str();
        let ty = r.u8();
        let item_size = r.u32() as usize;
        let stride = r.u32() as usize;
        let items = r.u32() as usize;
        let sampled = if items == 0 { 0 } else { (items - 1) / stride + 1 };
        let n = sampled * item_size;
        let vals = (0..n)
            .map(|_| match ty {
                0 => f32::from_le_bytes(r.bytes(4).try_into().unwrap()) as f64,
                1 => r.u32() as f64,
                _ => r.f64(),
            })
            .collect();
        Record { label, ty, item_size, stride, items, vals }
    }
    /// Compare with a full array of ours (sampled the same way).
    pub fn check(&self, what: &str, ours: &[f64], tol: f64) {
        assert_eq!(
            ours.len(),
            self.items * self.item_size,
            "{}: {} length (got {} / {} per item, recorded {} items)",
            what,
            self.label,
            ours.len(),
            self.item_size,
            self.items
        );
        let mut k = 0;
        let mut worst = 0.0f64;
        for i in (0..self.items).step_by(self.stride.max(1)) {
            for j in 0..self.item_size {
                let (a, b) = (ours[i * self.item_size + j], self.vals[k]);
                let d = (a - b).abs();
                if !(d <= tol) {
                    panic!("{}: {}[{}][{}] got {} vs recorded {} (|d| = {:e})", what, self.label, i, j, a, b, d);
                }
                worst = worst.max(d);
                k += 1;
            }
        }
        // FIXTURE_DIFFS=1 cargo test -- --nocapture: how close every record came
        if std::env::var("FIXTURE_DIFFS").is_ok() {
            eprintln!("checked {} {:e} {} {}", k, worst, what, self.label);
        }
    }
}

pub struct Case {
    pub name: String,
    pub spec: HashMap<String, String>,
    pub result: HashMap<String, String>,
    pub recs: Vec<Record>,
}
impl Case {
    pub fn rec(&self, l: &str) -> &Record {
        self.recs.iter().find(|r| r.label == l).unwrap_or_else(|| panic!("{}: no record {}", self.name, l))
    }
    pub fn spec_f(&self, k: &str) -> Vec<f64> {
        list_f(&self.spec[k])
    }
    pub fn spec_i(&self, k: &str) -> i64 {
        self.spec[k].parse().unwrap()
    }
    pub fn res_i(&self, k: &str) -> i64 {
        self.result[k].parse().unwrap()
    }
}

pub fn kv(s: &str) -> HashMap<String, String> {
    s.split(';')
        .filter(|x| !x.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap();
            (k.to_string(), v.to_string())
        })
        .collect()
}
pub fn list_f(s: &str) -> Vec<f64> {
    if s.is_empty() { vec![] } else { s.split(',').map(|x| x.parse::<f64>().unwrap()).collect() }
}

pub fn read_cases(r: &mut Rd, magic: &[u8]) -> Vec<Case> {
    assert_eq!(r.bytes(8), magic);
    let n = r.u32() as usize;
    (0..n)
        .map(|_| {
            let name = r.str();
            let spec = kv(&r.str());
            let result = kv(&r.str());
            let nr = r.u32() as usize;
            let recs = (0..nr).map(|_| Record::read(r)).collect();
            Case { name, spec, result, recs }
        })
        .collect()
}

pub fn f32s(v: &[f32]) -> Vec<f64> {
    v.iter().map(|&x| x as f64).collect()
}
pub fn u32s(v: &[u32]) -> Vec<f64> {
    v.iter().map(|&x| x as f64).collect()
}
