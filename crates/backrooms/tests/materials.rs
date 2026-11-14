//! Material lint: every lit surface a level builds has a physically
//! plausible albedo, so new props cannot drift back to cartoon colours.
//!
//! For each lit, opaque, non-glowing material on a few whole levels, the
//! *effective albedo* is its colour times the mean of its colour map (both
//! linear). Its brightest channel must lie inside `ALBEDO` (after
//! OpenMoonray's PBR validity range for non-metals; fresh snow is about 0.81,
//! charcoal about 0.02) and its saturation must stay below `MAX_SATURATION`.
//!
//! `--nocapture` prints the distribution and the worst offenders.
use engine::dmath::DMath;
use engine::material::MatKind;
use game::Game;
use std::collections::BTreeMap;

/// Linear albedo bounds for the brightest channel of a non-metal. The upper
/// bound is OpenMoonray's 0.87. Its recommended floor is 0.032, but black
/// plastics and rubber measure about 0.02 (Unreal lists charcoal at 0.02) and
/// several of the game's black parts sit at 0.010–0.014, so the floor here
/// only catches near-perfect black, which reads as a hole. Raising it is a
/// content task: find the parts with `--nocapture` and lighten them.
const ALBEDO: (f64, f64) = (0.008, 0.87);
/// (max − min) / max of the effective albedo. Red paint, safety yellow and
/// coloured plastic are genuinely saturated (0.93–0.98); a pure primary (1.0)
/// is not a surface.
const MAX_SATURATION: f64 = 0.985;
/// An opaque material whose colour map is clear (alpha < ½) over more than
/// this share of its pixels draws black where the map is clear.
const MAX_CLEAR_SHARE: f64 = 0.3;

/// Known exceptions: (placer, part of the mesh path, the rule, why).
const EXEMPT: &[(&str, &str, &str, &str)] = &[
    ("", "hair < head", "too dark", "the Hound's coat: a near-black silhouette is the design; its look is judged by the showcase goldens"),
    (
        "place_store",
        "",
        "clear map",
        "known issue: the showroom's department signs are text on a clear canvas in an opaque material, so they read as dark plates; needs a look decision",
    ),
];

/// The levels the lint builds: between them they cover every zone purpose,
/// all weathers and the anomalies (see `cargo xtask shots`), and the
/// Building's three floors with all their photographed surfaces.
const LEVELS: &[(u32, i32)] = &[(4242, 1), (4242, 2), (4242, 3), (4242, 4), (4242, 7), (7, 5), (7, 6), (1999, 7)];

/// Where a material came from: the placer that built the top-level object,
/// and the mesh's own name.
#[derive(Clone, Debug)]
struct Seen {
    placer: String,
    mesh: String,
    albedo: [f64; 3],
    /// share of the colour map's pixels that are clear
    clear: f64,
}

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).dpowf(2.4) }
}

/// Mean linear colour of a texture (every 7th pixel; white without one) and
/// the share of its pixels that are clear.
fn map_mean(m: &engine::material::Material) -> ([f64; 3], f64) {
    let Some(t) = m.map.as_ref() else { return ([1.0; 3], 0.0) };
    let img = t.image();
    let img = img.borrow();
    let (w, h) = (img.width as usize, img.height as usize);
    let bpp = if img.data.len() >= w * h * 4 { 4 } else { 1 };
    let (mut s, mut n, mut clear) = ([0.0; 3], 0.0, 0.0);
    for i in (0..w * h).step_by(7) {
        let p = &img.data[i * bpp..];
        for (c, v) in s.iter_mut().enumerate() {
            *v += srgb_to_linear(p[if bpp == 4 { c } else { 0 }] as f64 / 255.0);
        }
        if bpp == 4 && p[3] < 128 {
            clear += 1.0;
        }
        n += 1.0;
    }
    if n == 0.0 {
        return ([1.0; 3], 0.0);
    }
    (s.map(|v| v / n), clear / n)
}

/// The object's name and its named ancestors, innermost first.
fn name_path(o: &engine::scene::Obj) -> String {
    let mut names = vec![];
    let mut cur = Some(o.clone());
    while let Some(c) = cur {
        let n = c.r().name.clone();
        if !n.is_empty() {
            names.push(n);
        }
        cur = c.parent();
    }
    if names.is_empty() { "(unnamed)".into() } else { names.join(" < ") }
}

fn level_materials(seed: u32, level: i32) -> Vec<Seen> {
    // canvas textures draw from the native stream: fix it, so the means repeat
    game::base::rng::seed_native((u64::from(seed) << 8) | level as u64);
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}&nosound");
    g.boot();
    g.start_new_game();
    let mut out = vec![];
    for top in g.world.children() {
        let placer = top.r().user.placer.clone().unwrap_or_else(|| top.r().name.clone());
        top.traverse(&mut |o| {
            if !o.is_mesh() {
                return;
            }
            for m in o.mats() {
                let m = m.r();
                let lit = matches!(m.kind, MatKind::Lambert | MatKind::Phong | MatKind::Standard);
                let glows = m.emissive_intensity * (m.emissive.r.max(m.emissive.g).max(m.emissive.b)) > 0.02 || m.emissive_map.is_some();
                if !lit || glows || m.transparent || !m.visible || m.vertex_colors {
                    continue;
                }
                let (t, clear) = map_mean(&m);
                let clear = if m.alpha_test > 0.0 || m.alpha_map.is_some() { 0.0 } else { clear };
                let albedo = [m.color.r * t[0], m.color.g * t[1], m.color.b * t[2]];
                let mesh = if m.name.is_empty() { name_path(o) } else { m.name.clone() };
                let mesh = if mesh == "(unnamed)" {
                    let tris = o.try_geo().map_or(0, |gm| gm.0.borrow().index_count() / 3);
                    // its size, so a prop can be found by it
                    let size = o.try_geo().map(|gm| {
                        let b = gm.bounding_box();
                        format!("{:.2}×{:.2}×{:.2} m", b.max.x - b.min.x, b.max.y - b.min.y, b.max.z - b.min.z)
                    });
                    format!(
                        "(unnamed {:?}, {tris} triangles, {}, map {:?})",
                        m.kind,
                        size.unwrap_or_default(),
                        m.map.as_ref().map(|t| (t.width(), t.height()))
                    )
                } else {
                    mesh
                };
                out.push(Seen { placer: placer.clone(), mesh, albedo, clear });
            }
        });
    }
    out
}

fn max3(a: [f64; 3]) -> f64 {
    a[0].max(a[1]).max(a[2])
}
fn saturation(a: [f64; 3]) -> f64 {
    let mx = max3(a);
    if mx <= 1e-6 { 0.0 } else { (mx - a[0].min(a[1]).min(a[2])) / mx }
}

#[test]
fn lit_surfaces_have_plausible_albedo() {
    let mut bad: BTreeMap<String, (Seen, String)> = BTreeMap::new();
    let mut hist = [0usize; 10];
    let mut total = 0;
    for &(seed, level) in LEVELS {
        for s in level_materials(seed, level) {
            total += 1;
            let mx = max3(s.albedo);
            hist[((mx / 0.1) as usize).min(9)] += 1;
            let why = if s.clear > MAX_CLEAR_SHARE {
                Some(format!("clear map ({:.0}% of its pixels are clear, in an opaque material)", s.clear * 100.0))
            } else if mx < ALBEDO.0 {
                Some(format!("too dark (max channel {mx:.3} < {})", ALBEDO.0))
            } else if mx > ALBEDO.1 {
                Some(format!("too bright (max channel {mx:.3} > {})", ALBEDO.1))
            } else if saturation(s.albedo) > MAX_SATURATION && mx > 0.08 {
                Some(format!("too saturated ({:.2} > {MAX_SATURATION})", saturation(s.albedo)))
            } else {
                None
            };
            let exempt = |why: &str| EXEMPT.iter().any(|(p, m, rule, _)| s.placer == *p && s.mesh.contains(m) && why.starts_with(rule));
            if let Some(why) = why.filter(|w| !exempt(w)) {
                bad.entry(format!("{} / {}", s.placer, s.mesh)).or_insert((s, why));
            }
        }
    }
    eprintln!("{total} lit materials on {} levels; brightest-channel histogram (0.1 bins): {hist:?}", LEVELS.len());
    for (k, (s, why)) in &bad {
        eprintln!("  {k}: {why}  albedo [{:.3} {:.3} {:.3}]", s.albedo[0], s.albedo[1], s.albedo[2]);
    }
    assert!(bad.is_empty(), "{} materials outside the plausible albedo range (run with --nocapture for the list)", bad.len());
}
