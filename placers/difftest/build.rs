// Generates `pp_lists()` / `pp_lists_mut()`: every `*Positions` list of
// game::positions::PropPositions with its JS name (taken from the doc comment
// above each field), and `pp_extras()`: the parallel `Vec<f64>` fields that
// carry an extra per-entry field of a JS list (doc comment `fooPositions[i].bar`).
use std::fmt::Write;
fn main() {
    let src_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../game/src/positions.rs");
    println!("cargo:rerun-if-changed={}", src_path.display());
    let src = std::fs::read_to_string(&src_path).expect("positions.rs");
    let mut out = String::from("/// Every `*Positions` list with its JS name.\npub fn pp_lists(pp: &game::positions::PropPositions) -> Vec<(&'static str, &Vec<engine::math::Vec3>)> {\n    vec![\n");
    let mut mut_out = String::from("/// Every `*Positions` list with its JS name (mutable).\npub fn pp_lists_mut(pp: &mut game::positions::PropPositions) -> Vec<(&'static str, &mut Vec<engine::math::Vec3>)> {\n    vec![\n");
    let mut ex_out = String::from("#[allow(unused_variables)]\n/// Extra per-entry fields of `*Positions` lists: (JS list, field, values).\npub fn pp_extras(pp: &game::positions::PropPositions) -> Vec<(&'static str, &'static str, &Vec<f64>)> {\n    vec![\n");
    let mut js: Option<String> = None;
    for line in src.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("/// `") {
            if let Some(end) = rest.find('`') { js = Some(rest[..end].to_string()); }
            continue;
        }
        if let Some(rest) = t.strip_prefix("pub ") {
            if let Some((field, ty)) = rest.split_once(':') {
                if ty.trim().starts_with("Vec<f64>") {
                    if let Some((list, fld)) = js.as_deref().and_then(|n| n.split_once("[i].")) {
                        writeln!(ex_out, "        ({:?}, {:?}, &pp.{}),", list, fld, field.trim()).unwrap();
                    }
                    js = None;
                }
                if ty.trim().starts_with("Vec<Vec3>") {
                    let name = js.take().unwrap_or_else(|| field.to_string());
                    writeln!(out, "        ({:?}, &pp.{}),", name, field.trim()).unwrap();
                    writeln!(mut_out, "        ({:?}, &mut pp.{}),", name, field.trim()).unwrap();
                }
            }
        }
        if !t.starts_with("///") { js = None; }
    }
    out.push_str("    ]\n}\n");
    mut_out.push_str("    ]\n}\n");
    out.push_str(&mut_out);
    ex_out.push_str("    ]\n}\n");
    out.push_str(&ex_out);
    let dest = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("pp_lists.rs");
    std::fs::write(dest, out).unwrap();
}
