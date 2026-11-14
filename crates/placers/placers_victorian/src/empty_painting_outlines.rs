//! `place_empty_painting_outlines`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::sstep;
use placers_kit::paint::fill;
use placers_kit::place::mount_on_wall;
use placers_kit::place::salt;
use placers_kit::place::wall_candidates;

// Empty painting outline (wall): 3% chance per level. Absence-as-prop.
pub(crate) fn place_empty_painting_outlines(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let ep = group();
    // Random landscape / portrait orientation.
    let is_landscape = rnd() < 0.5;
    let outline_w = if is_landscape { 0.28 + rnd() * 0.14 } else { 0.20 + rnd() * 0.10 };
    let outline_h = if is_landscape { 0.20 + rnd() * 0.10 } else { 0.30 + rnd() * 0.14 };
    // the old patch and its 5 mm border: the decal's size
    let (pw, ph) = (outline_w + 0.010, outline_h + 0.010);
    // 40%: the cut end of its wire still caught on one hook
    let wire = rnd() < 0.4;
    // 30%: a scrape in the paint below where its corner caught
    let scrape = if rnd() < 0.3 {
        let sx = (rnd() - 0.5) * outline_w * 0.7;
        let sy = -outline_h / 2.0 - 0.03 - rnd() * 0.04;
        let rot = (rnd() - 0.5) * 0.3;
        Some((sx, sy, rot))
    } else {
        None
    };
    mount_on_wall(g, &ep, &c, 0.002, 1.60, 0.5);
    let p = ep.pos();
    let k = salt(p.x, p.z, 37);
    let decal = |tex: &Tex| lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
    let ghost = mesh(plane_geo!(pw, ph), decal(&ghost_tex(k, pw, ph)));
    ghost.position().set(0, 0, -0.0015);
    ep.add(&ghost);
    // the two picture hooks at the old hooks' places, their pins in the wall
    let brass = phong!({ color: 0x6a5838, specular: 0x4a4030, shininess: 36 });
    let pin = phong!({ color: 0x3c3a36, specular: 0x3a3a38, shininess: 40 });
    for side in [-1.0, 1.0] {
        let hook = group();
        hook.position().set(side * outline_w * 0.32, outline_h * 0.34, -0.002);
        put(&hook, "Real__outlineHook", &brass);
        put(&hook, "Real__outlinePin", &pin);
        if wire && side < 0.0 {
            put(&hook, "Real__outlineWire", &phong!({ color: 0x4a4640, specular: 0x3a3834, shininess: 30 }));
        }
        ep.add(&hook);
    }
    if let Some((sx, sy, rot)) = scrape {
        let s = mesh(plane_geo!(0.015, 0.06), decal(&scrape_tex(k)));
        s.position().set(sx, sy, -0.0015);
        s.rotation().z = rot;
        ep.add(&s);
    }
    g.world.add(&ep);
}

/// How far in from the decal's edge the picture's footprint begins: the
/// ghost fades out across this margin
const MARGIN: f64 = 0.025;

/// The mark (the decal `pw` x `ph` m): inside the picture's footprint the
/// wall a shade lighter and fresher than the faded wall round it, its edge
/// sharp; outside it a ghost of grey-brown dust along each edge, darkest
/// and widest along the top, faint along the foot, fading into the wall
fn ghost_tex(salt: u32, pw: f64, ph: f64) -> Tex {
    let ppm = 500.0;
    canvas_tex_opt((pw * ppm).round(), (ph * ppm).round(), false, move |ctx, w, h| {
        let seed = salt ^ 0x9d;
        let (a, b) = (pw / 2.0 - MARGIN, ph / 2.0 - MARGIN);
        fill(ctx, w, h, |u, v| {
            let (x, y) = ((u - 0.5) * pw, (v - 0.5) * ph);
            let (ox, oy) = (x.abs() - a, y.abs() - b);
            // signed distance from the footprint's edge (m, outside > 0)
            let d = if ox > 0.0 && oy > 0.0 { (ox * ox + oy * oy).sqrt() } else { ox.max(oy) };
            let edge = (pw / 2.0 - x.abs()).min(ph / 2.0 - y.abs());
            let fade = sstep(0.0, 0.008, edge);
            let mottle = fbm(seed, x * 30.0, y * 30.0, 3);
            if d < 0.0 {
                // the fresher wall: lighter, its edge as sharp as the frame's
                let k = sstep(0.0, 0.0012, -d);
                let al = (0.15 + 0.06 * mottle) * k;
                return [0.94, 0.90, 0.82, al];
            }
            let top = oy > ox && y > 0.0;
            let foot = oy > ox && y < 0.0;
            let (line, wide) = if top {
                (0.46, 0.012)
            } else if foot {
                (0.14, 0.005)
            } else {
                (0.26, 0.007)
            };
            let ghost = line * (-d / 0.0045).dexp() + 0.16 * (-d / wide).dexp();
            let al = ghost * (0.55 + 0.7 * mottle) * fade;
            [0.22, 0.19, 0.15, al.min(0.6)]
        });
    })
}

/// A scrape through the paint to the plaster, its edges grimed, thinning
/// to nothing at its ends
fn scrape_tex(salt: u32) -> Tex {
    canvas_tex_opt(24, 96, false, move |ctx, w, h| {
        let seed = salt ^ 0xb5;
        fill(ctx, w, h, |u, v| {
            let mid = 0.5 + 0.12 * (fbm(seed, v * 3.0, 0.5, 2) - 0.5);
            let width = 0.10 * (v * PI).dsin().max(0.0).dpowf(0.7);
            let d = (u - mid).abs();
            if d < width * 0.55 {
                return [0.84, 0.81, 0.74, 0.75 * sstep(0.0, 0.15, v.min(1.0 - v))];
            }
            let lip = 1.0 - sstep(width * 0.55, width * 1.4, d);
            [0.24, 0.20, 0.16, 0.4 * lip * (v * PI).dsin()]
        });
    })
}

fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = game::assets::props::prop_part_as_modelled(name)?;
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}
