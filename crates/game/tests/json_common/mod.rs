//! Shared test helpers: a small JSON reader used by these tests (for the
//! recorded fixtures in `tests/fixtures/`), float comparison, and the
//! deterministic stub deps.
#![allow(dead_code, reason = "helpers shared by several test binaries; each uses a part")]

use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn parse(s: &str) -> Json {
        let b = s.as_bytes();
        let mut i = 0;
        let v = parse_value(b, &mut i);
        skip_ws(b, &mut i);
        assert_eq!(i, b.len(), "trailing data in JSON");
        v
    }
    pub fn load(name: &str) -> Json {
        let p = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
        Json::parse(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e} (missing fixture)")))
    }
    pub fn get(&self, k: &str) -> &Json {
        match self {
            Json::Obj(v) => v.iter().find(|(kk, _)| kk == k).map_or(&Json::Null, |(_, v)| v),
            _ => &Json::Null,
        }
    }
    pub fn has(&self, k: &str) -> bool {
        matches!(self, Json::Obj(v) if v.iter().any(|(kk, _)| kk == k))
    }
    pub fn at(&self, i: usize) -> &Json {
        match self {
            Json::Arr(v) => &v[i],
            _ => panic!("not an array: {self:?}"),
        }
    }
    pub fn arr(&self) -> &[Json] {
        match self {
            Json::Arr(v) => v,
            _ => panic!("not an array: {self:?}"),
        }
    }
    pub fn len(&self) -> usize {
        self.arr().len()
    }
    pub fn f(&self) -> f64 {
        match self {
            Json::Num(n) => *n,
            Json::Null => f64::NAN,
            Json::Bool(b) => *b as i32 as f64,
            _ => panic!("not a number: {self:?}"),
        }
    }
    pub fn i(&self) -> i64 {
        let f = self.f();
        assert!(f == f.trunc(), "not an integer: {f}");
        f as i64
    }
    pub fn b(&self) -> bool {
        match self {
            Json::Bool(b) => *b,
            Json::Num(n) => *n != 0.0,
            _ => panic!("not a bool: {self:?}"),
        }
    }
    pub fn s(&self) -> &str {
        match self {
            Json::Str(s) => s,
            _ => panic!("not a string: {self:?}"),
        }
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Json::Null)
    }
    pub fn fs(&self) -> Vec<f64> {
        self.arr().iter().map(|v| v.f()).collect()
    }
    pub fn entries(&self) -> &[(String, Json)] {
        match self {
            Json::Obj(v) => v,
            _ => panic!("not an object"),
        }
    }
}
fn skip_ws(b: &[u8], i: &mut usize) {
    while *i < b.len() && (b[*i] as char).is_ascii_whitespace() {
        *i += 1;
    }
}
fn parse_value(b: &[u8], i: &mut usize) -> Json {
    skip_ws(b, i);
    match b[*i] {
        b'{' => {
            *i += 1;
            let mut v = vec![];
            loop {
                skip_ws(b, i);
                if b[*i] == b'}' {
                    *i += 1;
                    break;
                }
                let k = match parse_value(b, i) {
                    Json::Str(s) => s,
                    o => panic!("bad key {o:?}"),
                };
                skip_ws(b, i);
                assert_eq!(b[*i], b':');
                *i += 1;
                let val = parse_value(b, i);
                v.push((k, val));
                skip_ws(b, i);
                if b[*i] == b',' {
                    *i += 1;
                }
            }
            Json::Obj(v)
        }
        b'[' => {
            *i += 1;
            let mut v = vec![];
            loop {
                skip_ws(b, i);
                if b[*i] == b']' {
                    *i += 1;
                    break;
                }
                v.push(parse_value(b, i));
                skip_ws(b, i);
                if b[*i] == b',' {
                    *i += 1;
                }
            }
            Json::Arr(v)
        }
        b'"' => {
            *i += 1;
            let mut out: Vec<u8> = vec![];
            while b[*i] != b'"' {
                if b[*i] == b'\\' {
                    *i += 1;
                    match b[*i] {
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'u' => {
                            let h = std::str::from_utf8(&b[*i + 1..*i + 5]).unwrap();
                            let c = u32::from_str_radix(h, 16).unwrap();
                            *i += 4;
                            let ch = char::from_u32(c).unwrap_or('?');
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        c => out.push(c),
                    }
                    *i += 1;
                } else {
                    out.push(b[*i]);
                    *i += 1;
                }
            }
            *i += 1;
            Json::Str(String::from_utf8(out).unwrap())
        }
        b't' => {
            *i += 4;
            Json::Bool(true)
        }
        b'f' => {
            *i += 5;
            Json::Bool(false)
        }
        b'n' => {
            *i += 4;
            Json::Null
        }
        _ => {
            let st = *i;
            while *i < b.len() && matches!(b[*i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
                *i += 1;
            }
            Json::Num(std::str::from_utf8(&b[st..*i]).unwrap().parse::<f64>().unwrap())
        }
    }
}

/// |a − b| ≤ tol·max(1, |a|, |b|) (NaN only matches NaN)
pub fn close(a: f64, b: f64, tol: f64) -> bool {
    if a.is_nan() || b.is_nan() {
        return a.is_nan() && b.is_nan();
    }
    (a - b).abs() <= tol * 1f64.max(a.abs()).max(b.abs())
}
#[track_caller]
pub fn assert_close(a: f64, b: f64, tol: f64, what: &str) {
    assert!(close(a, b, tol), "{}: got {} vs recorded {} (diff {:e})", what, a, b, (a - b).abs());
}
#[track_caller]
pub fn assert_vec_close(a: &[f64], b: &[f64], tol: f64, what: &str) {
    assert_eq!(a.len(), b.len(), "{what}: length");
    for i in 0..a.len() {
        assert_close(a[i], b[i], tol, &format!("{what}[{i}]"));
    }
}
/// a float rounded to f32 precision (as f32 storage keeps it)
pub fn f32r(v: f64) -> f64 {
    v as f32 as f64
}

/// mulberry32(seed) with a draw counter.
pub fn counted_rng(seed: u32) -> (Box<dyn FnMut() -> f64>, Rc<Cell<u64>>) {
    let n = Rc::new(Cell::new(0u64));
    let n2 = n.clone();
    let mut m = game::base::rng::Mulberry32::new(seed);
    (
        Box::new(move || {
            n2.set(n2.get() + 1);
            m.next()
        }),
        n,
    )
}

/// Attribute checksum, as recorded in the fixtures: [len, Σv, Σ|v|·((i mod 7) + 1)].
pub fn attr_sum(a: &[f32]) -> [f64; 3] {
    let (mut t, mut ta) = (0.0f64, 0.0f64);
    for (i, &v) in a.iter().enumerate() {
        t += v as f64;
        ta += (v as f64).abs() * ((i % 7) + 1) as f64;
    }
    [a.len() as f64, t, ta]
}
pub fn index_sum(ix: &[u32]) -> [f64; 3] {
    let (mut s, mut sw) = (0.0f64, 0.0f64);
    for (i, &v) in ix.iter().enumerate() {
        s += v as f64;
        sw += v as f64 * ((i % 5) + 1) as f64;
    }
    [ix.len() as f64, s, sw]
}
/// Compare a geometry with its recorded checksums (f32 data, summed in a fixed order: relative 1e-9).
#[track_caller]
pub fn assert_geo(g: &engine::geometry::Geo, rec: &Json, what: &str) {
    let r = g.r();
    let chk = |name: &str, data: &[f32], j: &Json| {
        if j.is_null() {
            assert!(data.is_empty(), "{what}: {name} should be absent");
            return;
        }
        let s = attr_sum(data);
        assert_eq!(s[0], j.at(0).f(), "{what}: {name} length");
        assert_close(s[1], j.at(1).f(), 1e-9, &format!("{what} {name} sum"));
        assert_close(s[2], j.at(2).f(), 1e-9, &format!("{what} {name} weighted sum"));
    };
    chk("position", &r.position, rec.get("pos"));
    chk("normal", &r.normal, rec.get("nor"));
    chk("uv", &r.uv, rec.get("uv"));
    chk("color", &r.color, rec.get("col"));
    let ji = rec.get("idx");
    match &r.index {
        Some(ix) => {
            assert!(!ji.is_null(), "{what}: index should be absent");
            let s = index_sum(ix);
            for k in 0..3 {
                assert_eq!(s[k], ji.at(k).f(), "{what}: index sums");
            }
        }
        None => assert!(ji.is_null(), "{what}: index missing"),
    }
}
/// The material's type name as the fixtures record it
pub fn mat_type(m: &engine::material::Mat) -> &'static str {
    use engine::material::MatKind;
    match m.r().kind {
        MatKind::Basic => "basic",
        MatKind::Lambert => "lambert",
        MatKind::Phong => "phong",
        _ => "other",
    }
}
/// Compare a material with its recorded properties.
#[track_caller]
pub fn assert_mat(m: &engine::material::Mat, rec: &Json, what: &str) {
    use engine::material::Side;
    let r = m.r();
    assert_eq!(mat_type(m), rec.get("type").s(), "{what}: type");
    assert_eq!(r.color.get_hex() as i64, rec.get("color").i(), "{what}: color");
    if rec.get("type").s() == "lambert" {
        assert_eq!(r.emissive.get_hex() as i64, rec.get("emissive").i(), "{what}: emissive");
    }
    assert_eq!(r.map.is_some(), rec.get("map").b(), "{what}: map");
    let side = match r.side {
        Side::Front => 0,
        Side::Back => 1,
        Side::Double => 2,
    };
    assert_eq!(side, rec.get("side").i(), "{what}: side");
    assert_eq!(r.vertex_colors, rec.get("vc").b(), "{what}: vertex_colors");
    assert_eq!(r.fog, rec.get("fog").b(), "{what}: fog");
    assert_eq!(r.transparent, rec.get("tr").b(), "{what}: transparent");
    assert_close(r.opacity, rec.get("op").f(), 1e-12, &format!("{what}: opacity"));
    assert_eq!(r.depth_write, rec.get("dw").b(), "{what}: depth_write");
    assert_eq!(r.polygon_offset, rec.get("po").b(), "{what}: polygon_offset");
    assert_close(r.polygon_offset_factor, rec.get("pof").f(), 0.0, &format!("{what}: polygon_offset_factor"));
    assert_close(r.polygon_offset_units, rec.get("pou").f(), 0.0, &format!("{what}: polygon_offset_units"));
}
