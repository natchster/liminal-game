//! `place_sink_faucets`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put as t04_put;
use placers_kit::noise::fbm as t04_fbm;
use placers_kit::noise::hash as t04_hash;
use placers_kit::noise::sstep as t04_ss;
use placers_kit::paint::fill as t04_fill;
use placers_kit::place::mount_on_wall;
use placers_kit::place::vary as t04_vary;
use placers_kit::place::wall_candidates;

// Wall sink faucet: 6% chance per level. A wall-mounted two-valve faucet
// on 4 in centres (Blender batch `t04`, the Chicago Faucets 445 class):
// chromed brass gone dull, limed and tarnished, four-arm handles with their
// red and blue buttons, an L-type swing spout left swung aside. Its basin is
// gone: taken off the wall, it left its steel hanger, the waste's wall bend
// cut off at the slip nut, and its shape on the wall (the paint it kept
// clean, the caulk's line along its rim, the splashes above it, a run of
// lime from a leaking arm).
pub(crate) fn place_sink_faucets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let faucet = group();
    mount_on_wall(g, &faucet, &c, 0.004, 0.95, 0.6);
    let p = faucet.pos();
    let vary = t04_vary(p.x, p.z, 0.95);
    // the parts are modelled with the wall at z = 0: 4 mm behind the spot
    let o = group();
    o.position().z = -0.004;
    faucet.add(&o);
    let chrome_tex = sf_chrome_tex(vary);
    let chrome = phong!({ map: &chrome_tex, color: 0x8a8c8e, specular: 0x6a6c6e, shininess: 60 });
    t04_put(&o, "Real__sinkFaucetBody", &chrome);
    t04_put(&o, "Real__sinkFaucetHandles", &chrome);
    t04_put(&o, "Real__sinkFaucetIndexHot", &lambert!({ color: 0x7a2a22 }));
    t04_put(&o, "Real__sinkFaucetIndexCold", &lambert!({ color: 0x2c4466 }));
    // the spout swung aside on its swivel, as it was last left
    let swing = (t04_hash(vary, 3, 1) - 0.5) * 1.7;
    let pivot = group();
    pivot.position().set(0, 0, SF_Z);
    pivot.rotation().y = swing;
    o.add(&pivot);
    if let Some(sp) = t04_put(&pivot, "Real__sinkFaucetSpout", &chrome) {
        sp.position().z = -SF_Z;
    }
    // what the basin left: its hanger (black paint gone to rust), its waste
    t04_put(&o, "Real__sinkFaucetHanger", &lambert!({ map: &sf_hanger_tex(vary) }));
    t04_put(&o, "Real__sinkFaucetWaste", &phong!({ map: &chrome_tex, color: 0x6e7072, specular: 0x4a4c4e, shininess: 40 }));
    let ghost = mesh(
        plane_geo!(SF_GHOST[0], SF_GHOST[1]),
        lambert!({ map: &sf_ghost_tex(vary), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    ghost.position().set(0, SF_GHOST[2], 0.0006);
    o.add(&ghost);
    g.world.add(&faucet);
}

/// Over: `top` (straight colour and alpha) laid on `base`
fn t04_over(base: [f64; 4], top: [f64; 4]) -> [f64; 4] {
    let a = top[3] + base[3] * (1.0 - top[3]);
    if a <= 1e-6 {
        return [0.0; 4];
    }
    let mut o = [0.0; 4];
    for k in 0..3 {
        o[k] = (top[k] * top[3] + base[k] * base[3] * (1.0 - top[3])) / a;
    }
    o[3] = a;
    o
}

// The faucet's frame (`t04.py`, SF_*): the valve bodies' axis off the wall,
// the swivel's axis; where the basin's back was on the wall, its rim; the
// hanger and the waste
const SF_C: f64 = 0.0508;

const SF_Z: f64 = 0.050;

const SF_RIM: f64 = -0.172;

const SF_HALF: f64 = 0.254;

const SF_BACK_LOW: f64 = -0.360;

const SF_HANGER_Y: f64 = -0.205;

const SF_WASTE_Y: f64 = -0.430;

/// The patch the basin left on the wall: its width and height, its middle
const SF_GHOST: [f64; 3] = [0.58, 0.62, -0.245];

/// Chromed brass after years of hard water: the plating dulled and
/// tarnished in patches, a crust of lime, pits (a grey the chrome's colour
/// multiplies; light where the lime is)
fn sf_chrome_tex(vary: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x51f1;
        t04_fill(ctx, w, h, |u, v| {
            let n = t04_fbm(seed, u * 6.0, v * 6.0, 4);
            let tarnish = t04_ss(0.55, 0.75, t04_fbm(seed ^ 0x77, u * 3.0, v * 3.0, 3)) * 0.35;
            let lime = t04_ss(0.62, 0.8, t04_fbm(seed ^ 0x1234, u * 9.0, v * 9.0, 3));
            let pit = if t04_hash(seed ^ 0x99, (u * 128.0) as i32, (v * 128.0) as i32) < 0.012 { 0.45 } else { 0.0 };
            let g0 = 0.80 + 0.10 * n - tarnish - pit;
            let g1 = g0 + (0.98 - g0) * lime * 0.85;
            [g1 * 0.99, g1 * 0.98, g1 * 0.95, 1.0]
        });
    })
}

/// The basin's hanger: black enamel worn through to rust at its edges and
/// round its screws, rust bloomed through the paint in specks
fn sf_hanger_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 64, false, move |ctx, w, h| {
        let seed = vary ^ 0x4a7;
        t04_fill(ctx, w, h, |u, v| {
            let n = t04_fbm(seed, u * 30.0, v * 8.0, 4);
            let edge = (v.min(1.0 - v) * 6.0).min(1.0);
            let rust = t04_ss(0.62, 0.85, n + (1.0 - edge) * 0.3) * 0.8;
            let speck = if t04_hash(seed ^ 3, (u * 256.0) as i32, (v * 64.0) as i32) < 0.02 { 0.5 } else { 0.0 };
            let r = (rust + speck).min(1.0);
            let paint = [0.085, 0.082, 0.080];
            let ox = [0.24 + 0.06 * n, 0.15 + 0.03 * n, 0.09];
            [paint[0] + (ox[0] - paint[0]) * r, paint[1] + (ox[1] - paint[1]) * r, paint[2] + (ox[2] - paint[2]) * r, 1.0]
        });
    })
}

/// What the basin left on the wall (transparent elsewhere): the paint its
/// back kept from the years (a little lighter, its outline a line of dirt),
/// the caulk's bead along its rim torn off in strips, splashes and soap above
/// it, a run of lime and rust down from the left arm's flange, rust runs from
/// the hanger's screws, dirt round the waste
fn sf_ghost_tex(vary: u32) -> Tex {
    canvas_tex_opt(320, 344, false, move |ctx, w, h| {
        let seed = vary ^ 0x6057;
        let (gw, gh, gy) = (SF_GHOST[0], SF_GHOST[1], SF_GHOST[2]);
        t04_fill(ctx, w, h, |u, v| {
            let x = (u - 0.5) * gw;
            let y = gy + (v - 0.5) * gh;
            let mut c = [0.0, 0.0, 0.0, 0.0];
            let n = t04_fbm(seed, x * 40.0, y * 40.0, 4);
            // the back's outline: a rectangle, its lower corners rounded
            let rr = 0.05;
            let (cx, cy) = ((x.abs() - (SF_HALF - rr)).max(0.0), ((SF_BACK_LOW + rr) - y).max(0.0));
            let out_side = x.abs() - SF_HALF;
            let d_round = (cx * cx + cy * cy).sqrt() - rr;
            let inside = if y < SF_BACK_LOW + rr { d_round } else { out_side }.max(y - SF_RIM);
            // the paint it kept: lighter inside, a soft fringe
            let keep = 1.0 - t04_ss(-0.004, 0.003, inside);
            if keep > 0.0 {
                let a = keep * (0.035 + 0.03 * n);
                c = t04_over(c, [0.93, 0.93, 0.92, a]);
            }
            // a line of dirt where its edge met the wall
            let line = (-(inside / 0.0025).powi(2)).exp() * (0.10 + 0.14 * n);
            c = t04_over(c, [0.20, 0.17, 0.12, line]);
            // the caulk along the rim, torn away in places
            let dr = y - SF_RIM;
            if x.abs() < SF_HALF + 0.004 && dr.abs() < 0.006 {
                let keep_caulk = t04_ss(0.48, 0.56, t04_fbm(seed ^ 0xca, x * 22.0, 0.3, 3));
                let bead = (1.0 - (dr / 0.0045).powi(2)).max(0.0);
                let dirt = 0.75 + 0.25 * t04_fbm(seed ^ 0xcb, x * 120.0, y * 120.0, 2);
                c = t04_over(c, [0.62 * dirt, 0.61 * dirt, 0.56 * dirt, bead * keep_caulk * 0.85]);
            }
            // above the rim: splashes and soap, densest just over it
            if dr > 0.0 && x.abs() < SF_HALF + 0.01 && dr < 0.17 {
                let near = (-dr / 0.06).exp();
                let sp = t04_hash(seed ^ 0x51, (x * 900.0) as i32, (y * 900.0) as i32);
                if sp < 0.05 * near + 0.004 {
                    c = t04_over(c, [0.82, 0.80, 0.74, 0.35]);
                }
                let film = t04_ss(0.45, 0.8, t04_fbm(seed ^ 0x52, x * 14.0, y * 14.0, 3)) * near * 0.22;
                c = t04_over(c, [0.30, 0.26, 0.18, film]);
            }
            // lime and rust run down from the left arm's flange (it leaked)
            let fx = x + SF_C;
            if y < -0.020 && y > -0.48 {
                let fall = -0.020 - y;
                let wob = 0.004 * (y * 37.0 + 1.3).sin() + 0.003 * t04_fbm(seed ^ 0x17, 0.5, y * 12.0, 2);
                let wid = 0.004 + 0.010 * t04_ss(0.0, 0.25, fall);
                let core = (-((fx - wob) / wid).powi(2)).exp();
                let fade = 1.0 - t04_ss(0.15, 0.46, fall);
                let tide = (-((((fx - wob).abs() - wid) / 0.0015).powi(2))).exp() * 0.5;
                let a = (core * 0.30 + tide * 0.25) * fade;
                c = t04_over(c, [0.46, 0.36, 0.22, a]);
            }
            // rust runs down from the hanger's screws
            for sx in [-0.180, -0.062, 0.062, 0.180] {
                let dx = x - sx;
                let fall = SF_HANGER_Y - 0.01 - y;
                if fall > 0.0 && fall < 0.16 && dx.abs() < 0.01 {
                    let len = 0.04 + 0.10 * t04_hash(seed ^ 0x61, (sx * 100.0) as i32, 0);
                    let a = (-(dx / 0.0025).powi(2)).exp() * (1.0 - t04_ss(0.0, len, fall)) * 0.35;
                    c = t04_over(c, [0.38, 0.20, 0.09, a]);
                }
            }
            // dirt round the waste's escutcheon
            let dw = (x * x + (y - SF_WASTE_Y).powi(2)).sqrt();
            let ring = t04_ss(0.031, 0.034, dw) * (1.0 - t04_ss(0.034, 0.06, dw));
            c = t04_over(c, [0.18, 0.15, 0.11, ring * (0.25 + 0.15 * n)]);
            c
        });
    })
}
