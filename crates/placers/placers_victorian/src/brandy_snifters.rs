//! `place_brandy_snifters`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;
use placers_kit::place::vary as t10_vary;

// Brandy snifter + coaster (floor): 4% chance per level. A 5.5 oz
// brandy snifter (Blender family `parlour_and_office_big_things`, the Libbey Embassy class) with a
// pour of brandy still in it, trembling, on a coaster (turned wood, cork,
// stitched leather or dark wood) ringed by years of wet glasses; ash
// flicked on its edge, now and then a small whisky glass beside it, dried.
pub(crate) fn place_brandy_snifters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // the old draws, call for call
    let brandy_color = SN_BRANDY[ri(4)];
    let liquid_fill = 0.35 + rnd() * 0.15;
    let ripple_phase_x = rnd() * PI * 2.0;
    let ripple_phase_z = rnd() * PI * 2.0;
    let ripple_freq_x = 1.1 + rnd() * 0.4;
    let ripple_freq_z = 1.4 + rnd() * 0.4;
    // wood round, cork round, leather square, dark wood round
    let coaster = ri(4);
    let ash = rnd() < 0.3;
    let rocks = if rnd() < 0.25 {
        let dregs = rnd() < 0.6;
        let rx = 0.10 + rnd() * 0.04;
        let rz = (rnd() - 0.5) * 0.05;
        Some((dregs, rx, rz))
    } else {
        None
    };
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let turn = rnd() * PI * 2.0;
    let bs = group();
    bs.position().set(px, 0, pz);
    bs.rotation().y = turn;
    let vary = t10_vary(px, pz, turn);
    // the coaster
    let kind = [0, 1, 2, 0][coaster];
    let (part, tex) = match coaster {
        0 => ("Real__snifterCoasterWood", sn_coaster_tex(vary, 0)),
        1 => ("Real__snifterCoasterCork", sn_coaster_tex(vary, 1)),
        2 => ("Real__snifterCoasterLeather", sn_coaster_tex(vary, 2)),
        _ => ("Real__snifterCoasterWood", sn_coaster_tex(vary, 3)),
    };
    put(&bs, part, &lambert!({ map: &tex }));
    let top = SN_COASTER_TOP[kind];
    // the glass and the brandy in it
    let glass = group();
    glass.position().y = top;
    bs.add(&glass);
    let gmat = sn_glass_mat(&sn_dust_tex(), 0.5, 0.32);
    let pour = ((liquid_fill - 0.35) / 0.05).floor().clamp(0.0, 2.0) as usize;
    let level = SN_LEVELS[pour];
    let brandy = phong!({ color: brandy_color, specular: 0x3a3226, shininess: 80, transparent: true, opacity: 0.82, depthWrite: false });
    if let Some(b) = put(&glass, ["Real__snifterBrandyLow", "Real__snifterBrandyMid", "Real__snifterBrandyHigh"][pour], &brandy) {
        b.set_render_order(1);
    }
    let surf_mat = phong!({ color: brandy_color, specular: 0x6a5a44, shininess: 120, transparent: true, opacity: 0.88, depthWrite: false });
    let surface = mesh(circle_geo!(sn_inner_r(level) - 0.0002, 32), &surf_mat);
    surface.rotation().x = -PI / 2.0;
    surface.position().y = level;
    surface.set_render_order(1);
    glass.add(&surface);
    if let Some(m) = put(&glass, "Real__snifterBowl", &gmat) {
        m.set_render_order(2);
    }
    // the foot and the stem: thick glass, its edges catching the light
    if let Some(m) = put(&glass, "Real__snifterFoot", &sn_glass_mat(&sn_dust_tex(), 0.9, 0.7)) {
        m.set_render_order(2);
    }
    // Liquid ripple animator.
    let surf_base_y = level;
    {
        let surface = surface.clone();
        g.animate(move |_g, _dt, t| {
            surface.rotation().x = -PI / 2.0 + (t * ripple_freq_x + ripple_phase_x).sin() * 0.012;
            surface.rotation().z = (t * ripple_freq_z + ripple_phase_z).cos() * 0.012;
            surface.position().y = surf_base_y + (t * ripple_freq_x * 1.3 + ripple_phase_z).sin() * 0.0004;
        });
    }
    // ash flicked on the coaster's edge
    if ash {
        let a = mesh(
            plane_geo!(0.03, 0.03),
            lambert!({ map: &sn_ash_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
        );
        a.rotation().x = -PI / 2.0;
        a.rotation().z = hash(vary, 1, 1) * PI * 2.0;
        a.position().set(0.036, top + 0.0003, 0.020);
        bs.add(&a);
    }
    // a small whisky glass beside it, the last of what was in it dried in its foot
    if let Some((dregs, rx, rz)) = rocks {
        let rg = group();
        rg.position().set(rx, 0, rz);
        if dregs {
            put(
                &rg,
                "Real__snifterRocksDregs",
                &phong!({ color: brandy_color, specular: 0x4a3a2a, shininess: 60, transparent: true, opacity: 0.9 }),
            );
        }
        if let Some(m) = put(&rg, "Real__snifterRocks", &sn_glass_mat(&sn_dust_tex(), 0.65, 0.45)) {
            m.set_render_order(2);
        }
        bs.add(&rg);
    }
    g.world.add(&bs);
}

/// The snifter's bowl inside (victorian/parlour_and_office_big_things.py `SN_*`): its widest (y), its radius
/// there, below it its half height; where the brandy's surface stands for
/// a low, a middling and a generous pour (25, 44, 65 ml)
const SN_YC: f64 = 0.058;

const SN_A_IN: f64 = 0.0365 - 0.0012;

const SN_B_IN: f64 = 0.034;

const SN_LEVELS: [f64; 3] = [0.040, 0.046, 0.052];

/// The coasters' tops (wood, cork, leather), what the glass stands on
const SN_COASTER_TOP: [f64; 3] = [0.0062, 0.0042, 0.0042];

/// The brandies (sRGB) for the old colours in their order
const SN_BRANDY: [u32; 4] = [0x5a2a0e, 0x6c3610, 0x7e4a1a, 0x4a220c];

/// The bowl's inside radius at height y (below its widest)
fn sn_inner_r(y: f64) -> f64 {
    let k = (y - SN_YC) / SN_B_IN;
    SN_A_IN * (1.0 - k * k).max(0.0).sqrt()
}

/// Clear glass as the windows' (`Patch::Glass`): the room seen in it by
/// Fresnel (`room`: how much), bright at its edges where it is seen at a
/// graze; its dust (the map's grey, `dust` of it)
fn sn_glass_mat(map: &Tex, dust: f32, room: f32) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 260, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "snifter_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = room;
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        f[8] = dust;
        f[9] = 0.20;
        f[10] = 0.19;
        f[11] = 0.17;
        f[12] = 0.0;
        f[13] = 0.07;
    }
    m
}

/// Dust and fingermarks on glass (grey: how much), shared by every glass
fn sn_dust_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                fill(ctx, w, h, |u, v| {
                    let d = fbm(81, u * 7.0, v * 7.0, 3) * 0.6 + fbm(82, u * 29.0, v * 29.0, 2) * 0.4;
                    // fingers' smears round the bowl's belly
                    let smear = sstep(0.62, 0.8, fbm(83, u * 9.0, v * 3.0, 3)) * (1.0 - sstep(0.15, 0.4, (v - 0.45).abs()));
                    // the dust heavier toward the top (it settles on what faces up)
                    let k = 0.15 + 0.4 * d + 0.5 * smear + 0.15 * v;
                    [k, k, k, 1.0]
                });
            })
        })
        .clone()
    })
}

/// A coaster's face as its top is mapped (u, v across it): turned wood's
/// rings, cork's grains, or leather's grain and its stitching; the white
/// and dark rings of wet glasses, wear
fn sn_coaster_tex(vary: u32, kind: usize) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(vary, 0xc0a5);
        let seed = (r.next() * 4.0e9) as u32;
        let rings: Vec<(f64, f64, f64, bool)> =
            (0..3).map(|_| (r.range(0.35, 0.65), r.range(0.35, 0.65), r.range(0.17, 0.21), r.next() < 0.5)).collect();
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u - 0.5, v - 0.5);
            let rr = (x * x + y * y).sqrt();
            let mut o = match kind {
                0 | 3 => {
                    // end grain turned flat: rings round the middle, the figure
                    let ring = ((rr * 160.0 + fbm(seed, u * 6.0, v * 6.0, 3) * 6.0).sin() * 0.5 + 0.5).powf(2.0);
                    let base = if kind == 0 { [0.40, 0.25, 0.14] } else { [0.18, 0.11, 0.07] };
                    base.map(|c| c * (0.88 + 0.15 * fbm(seed ^ 3, u * 40.0, v * 3.0, 2) - 0.18 * ring))
                }
                1 => {
                    // pressed cork: granules light and dark
                    let g = vn(seed, u * 70.0, v * 70.0);
                    let g2 = vn(seed ^ 5, u * 160.0, v * 160.0);
                    [0.50, 0.37, 0.24].map(|c| c * (0.75 + 0.35 * g + 0.15 * g2))
                }
                _ => {
                    // leather: its pebbled grain, darkened along its stitched edge
                    let g = vn(seed, u * 90.0, v * 90.0);
                    let edge = (0.5 - x.abs()).min(0.5 - y.abs());
                    let mut o = [0.24, 0.15, 0.10].map(|c| c * (0.85 + 0.25 * g));
                    let stitch = (edge - 0.055).abs() < 0.008 && ((u + v) * 60.0).fract() < 0.55;
                    if stitch {
                        o = [0.50, 0.44, 0.34];
                    }
                    o.map(|c| c * (0.75 + 0.25 * sstep(0.0, 0.05, edge)))
                }
            };
            // the rings left by wet glasses: a white bloom on wood, dark on the rest
            for &(cx, cy, cr, white) in &rings {
                let d = (((u - cx).powi(2) + (v - cy).powi(2)).sqrt() - cr).abs();
                let k = (1.0 - sstep(0.0, 0.012, d)) * 0.6;
                o = if white && kind != 1 { o.map(|c| c + (0.6 - c) * k * 0.6) } else { o.map(|c| c * (1.0 - 0.4 * k)) };
            }
            // worn, dusty
            let dust = 0.12 + 0.1 * fbm(seed ^ 9, u * 8.0, v * 8.0, 3);
            [o[0] + (0.5 - o[0]) * dust, o[1] + (0.47 - o[1]) * dust, o[2] + (0.42 - o[2]) * dust, 1.0]
        });
    })
}

/// A flick of cigar ash: a soft grey drift, a few flakes
fn sn_ash_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(64, 64, false, |ctx, w, h| {
                fill(ctx, w, h, |u, v| {
                    let (x, y) = ((u - 0.5) * 2.0, (v - 0.5) * 2.0);
                    let d = (x * x * 1.6 + y * y).sqrt();
                    let n = fbm(91, u * 9.0, v * 9.0, 3);
                    let a = (1.0 - sstep(0.3, 0.95, d + 0.4 * (n - 0.5))) * (0.5 + 0.5 * n);
                    let k = 0.35 + 0.3 * vn(92, u * 40.0, v * 40.0);
                    [k, k * 0.97, k * 0.93, a * 0.9]
                });
            })
        })
        .clone()
    })
}
