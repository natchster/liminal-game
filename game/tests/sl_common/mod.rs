//! Shared test helpers: a small JSON reader for the fixtures that
//! tools/sl_fixtures.mjs writes (serde_json is not in the offline registry),
//! float comparison, and the deterministic stub deps.
#![allow(dead_code)]

use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub enum Json { Null, Bool(bool), Num(f64), Str(String), Arr(Vec<Json>), Obj(Vec<(String, Json)>) }

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
        Json::parse(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {} (run `node tools/sl_fixtures.mjs`)", p, e)))
    }
    pub fn get(&self, k: &str) -> &Json {
        match self { Json::Obj(v) => v.iter().find(|(kk, _)| kk == k).map(|(_, v)| v).unwrap_or(&Json::Null), _ => &Json::Null }
    }
    pub fn has(&self, k: &str) -> bool { matches!(self, Json::Obj(v) if v.iter().any(|(kk, _)| kk == k)) }
    pub fn at(&self, i: usize) -> &Json { match self { Json::Arr(v) => &v[i], _ => panic!("not an array: {:?}", self) } }
    pub fn arr(&self) -> &[Json] { match self { Json::Arr(v) => v, _ => panic!("not an array: {:?}", self) } }
    pub fn len(&self) -> usize { self.arr().len() }
    pub fn f(&self) -> f64 { match self { Json::Num(n) => *n, Json::Null => f64::NAN, Json::Bool(b) => *b as i32 as f64, _ => panic!("not a number: {:?}", self) } }
    pub fn i(&self) -> i64 { let f = self.f(); assert!(f == f.trunc(), "not an integer: {}", f); f as i64 }
    pub fn b(&self) -> bool { match self { Json::Bool(b) => *b, Json::Num(n) => *n != 0.0, _ => panic!("not a bool: {:?}", self) } }
    pub fn s(&self) -> &str { match self { Json::Str(s) => s, _ => panic!("not a string: {:?}", self) } }
    pub fn is_null(&self) -> bool { matches!(self, Json::Null) }
    pub fn fs(&self) -> Vec<f64> { self.arr().iter().map(|v| v.f()).collect() }
    pub fn entries(&self) -> &[(String, Json)] { match self { Json::Obj(v) => v, _ => panic!("not an object") } }
}
fn skip_ws(b: &[u8], i: &mut usize) { while *i < b.len() && (b[*i] as char).is_ascii_whitespace() { *i += 1; } }
fn parse_value(b: &[u8], i: &mut usize) -> Json {
    skip_ws(b, i);
    match b[*i] {
        b'{' => {
            *i += 1; let mut v = vec![];
            loop {
                skip_ws(b, i);
                if b[*i] == b'}' { *i += 1; break; }
                let k = match parse_value(b, i) { Json::Str(s) => s, o => panic!("bad key {:?}", o) };
                skip_ws(b, i); assert_eq!(b[*i], b':'); *i += 1;
                let val = parse_value(b, i);
                v.push((k, val));
                skip_ws(b, i);
                if b[*i] == b',' { *i += 1; }
            }
            Json::Obj(v)
        }
        b'[' => {
            *i += 1; let mut v = vec![];
            loop {
                skip_ws(b, i);
                if b[*i] == b']' { *i += 1; break; }
                v.push(parse_value(b, i));
                skip_ws(b, i);
                if b[*i] == b',' { *i += 1; }
            }
            Json::Arr(v)
        }
        b'"' => {
            *i += 1; let mut out: Vec<u8> = vec![];
            while b[*i] != b'"' {
                if b[*i] == b'\\' {
                    *i += 1;
                    match b[*i] {
                        b'n' => out.push(b'\n'), b't' => out.push(b'\t'), b'r' => out.push(b'\r'), b'b' => out.push(8), b'f' => out.push(12),
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
                } else { out.push(b[*i]); *i += 1; }
            }
            *i += 1;
            Json::Str(String::from_utf8(out).unwrap())
        }
        b't' => { *i += 4; Json::Bool(true) }
        b'f' => { *i += 5; Json::Bool(false) }
        b'n' => { *i += 4; Json::Null }
        _ => {
            let st = *i;
            while *i < b.len() && matches!(b[*i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') { *i += 1; }
            Json::Num(std::str::from_utf8(&b[st..*i]).unwrap().parse::<f64>().unwrap())
        }
    }
}

/// |a − b| ≤ tol·max(1, |a|, |b|) (NaN only matches NaN)
pub fn close(a: f64, b: f64, tol: f64) -> bool {
    if a.is_nan() || b.is_nan() { return a.is_nan() && b.is_nan(); }
    (a - b).abs() <= tol * 1f64.max(a.abs()).max(b.abs())
}
#[track_caller]
pub fn assert_close(a: f64, b: f64, tol: f64, what: &str) {
    assert!(close(a, b, tol), "{}: rust {} vs js {} (diff {:e})", what, a, b, (a - b).abs());
}
#[track_caller]
pub fn assert_vec_close(a: &[f64], b: &[f64], tol: f64, what: &str) {
    assert_eq!(a.len(), b.len(), "{}: length", what);
    for i in 0..a.len() { assert_close(a[i], b[i], tol, &format!("{}[{}]", what, i)); }
}
/// floats as a Float32Array stores them
pub fn f32r(v: f64) -> f64 { v as f32 as f64 }

/// mulberry32(seed) with a draw counter (like the harness's `counted`).
pub fn counted_rng(seed: u32) -> (Box<dyn FnMut() -> f64>, Rc<Cell<u64>>) {
    let n = Rc::new(Cell::new(0u64));
    let n2 = n.clone();
    let mut m = game::rng::Mulberry32::new(seed);
    (Box::new(move || { n2.set(n2.get() + 1); m.next() }), n)
}

/// Sums like the harness's `attrSum`: [len, Σv, Σ|v|·((i mod 7) + 1)].
pub fn attr_sum(a: &[f32]) -> [f64; 3] {
    let (mut t, mut ta) = (0.0f64, 0.0f64);
    for (i, &v) in a.iter().enumerate() { t += v as f64; ta += (v as f64).abs() * ((i % 7) + 1) as f64; }
    [a.len() as f64, t, ta]
}
pub fn index_sum(ix: &[u32]) -> [f64; 3] {
    let (mut s, mut sw) = (0.0f64, 0.0f64);
    for (i, &v) in ix.iter().enumerate() { s += v as f64; sw += v as f64 * ((i % 5) + 1) as f64; }
    [ix.len() as f64, s, sw]
}
/// Compare a geometry with the harness's `geoSum` (f32 data, summed in the same order: relative 1e-9).
#[track_caller]
pub fn assert_geo(g: &engine::geometry::Geo, js: &Json, what: &str) {
    let r = g.r();
    let chk = |name: &str, data: &[f32], j: &Json| {
        if j.is_null() { assert!(data.is_empty(), "{}: {} should be absent", what, name); return; }
        let s = attr_sum(data);
        assert_eq!(s[0], j.at(0).f(), "{}: {} length", what, name);
        assert_close(s[1], j.at(1).f(), 1e-9, &format!("{} {} sum", what, name));
        assert_close(s[2], j.at(2).f(), 1e-9, &format!("{} {} weighted sum", what, name));
    };
    chk("position", &r.position, js.get("pos"));
    chk("normal", &r.normal, js.get("nor"));
    chk("uv", &r.uv, js.get("uv"));
    chk("color", &r.color, js.get("col"));
    let ji = js.get("idx");
    match &r.index {
        Some(ix) => {
            assert!(!ji.is_null(), "{}: index should be absent", what);
            let s = index_sum(ix);
            for k in 0..3 { assert_eq!(s[k], ji.at(k).f(), "{}: index sums", what); }
        }
        None => assert!(ji.is_null(), "{}: index missing", what),
    }
}
/// three's `material.type`
pub fn mat_type(m: &engine::material::Mat) -> &'static str {
    use engine::material::MatKind;
    match m.r().kind { MatKind::Basic => "MeshBasicMaterial", MatKind::Lambert => "MeshLambertMaterial", MatKind::Phong => "MeshPhongMaterial", _ => "other" }
}
/// Compare a material with the harness's `matInfo`.
#[track_caller]
pub fn assert_mat(m: &engine::material::Mat, js: &Json, what: &str) {
    use engine::material::Side;
    let r = m.r();
    assert_eq!(mat_type(m), js.get("type").s(), "{}: type", what);
    assert_eq!(r.color.get_hex() as i64, js.get("color").i(), "{}: color", what);
    if js.get("type").s() == "MeshLambertMaterial" { assert_eq!(r.emissive.get_hex() as i64, js.get("emissive").i(), "{}: emissive", what); }
    assert_eq!(r.map.is_some(), js.get("map").b(), "{}: map", what);
    let side = match r.side { Side::Front => 0, Side::Back => 1, Side::Double => 2 };
    assert_eq!(side, js.get("side").i(), "{}: side", what);
    assert_eq!(r.vertex_colors, js.get("vc").b(), "{}: vertexColors", what);
    assert_eq!(r.fog, js.get("fog").b(), "{}: fog", what);
    assert_eq!(r.transparent, js.get("tr").b(), "{}: transparent", what);
    assert_close(r.opacity, js.get("op").f(), 1e-12, &format!("{}: opacity", what));
    assert_eq!(r.depth_write, js.get("dw").b(), "{}: depthWrite", what);
    assert_eq!(r.polygon_offset, js.get("po").b(), "{}: polygonOffset", what);
    assert_close(r.polygon_offset_factor, js.get("pof").f(), 0.0, &format!("{}: polygonOffsetFactor", what));
    assert_close(r.polygon_offset_units, js.get("pou").f(), 0.0, &format!("{}: polygonOffsetUnits", what));
}
