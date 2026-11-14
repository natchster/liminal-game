//! `place_frosted_windows`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::glass::glass_mat;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::pick_wall;
use placers_kit::place::vary as t04_vary;
use placers_kit::place::wall_mount;

// Frosted-glass window: 3% chance per level. A small side-hung casement
// of four lights in the wall (Blender batch `t04`): painted softwood, its
// architrave, window board and apron, the sash with its glazing bars and
// beads, a cockspur fastener; the glass acid-etched (frosted) so nothing
// shows through it but the light behind (`Patch::Glass`, milky). The paint
// gone yellow and chipped to the wood, dust on the board and the bars, a
// web in a pane's corner. The old 50%: a lamp still lit in the room beyond,
// the panes glowing warm with it (and its light, as before).
pub(crate) fn place_frosted_windows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let window_ = group();
    let mount_y = 1.70;
    // 50% chance: a lamp lit in the room behind it
    let lit = rnd() < 0.5;
    wall_mount(g, &window_, x, z, dx, dz, 0.006);
    let p = window_.pos();
    let vary = t04_vary(p.x, p.z, mount_y);
    // the parts are modelled with the wall at z = 0: 6 mm behind the spot
    let body = group();
    body.position().set(0, mount_y, -0.006);
    window_.add(&body);
    put(&body, "Real__frostedWindowFrame", &lambert!({ map: &fw_paint_tex(vary) }));
    put(&body, "Real__frostedWindowIron", &phong!({ color: 0x1e1d1b, specular: 0x262524, shininess: 18 }));
    // behind the glass: the lamp's warm light, or a dark room
    let back = if lit { [0.30, 0.21, 0.11] } else { [0.012, 0.012, 0.013] };
    for (k, pane) in FW_PANES.iter().enumerate() {
        let (w, h) = (pane[1] - pane[0] + FW_UNDER, pane[3] - pane[2] + FW_UNDER);
        let m = glass_mat(fw_dirt_tex(vary, k), "frosted_glass", 2.4, back, 0.9, 0.04, [0.8, 0.16, 0.15, 0.13], 30.0);
        if lit {
            // the lamp behind: brighter toward the middle of the window
            let pu = m.r().patch_uniforms.clone();
            let f = &mut pu.borrow_mut().f;
            f[4] = if k % 2 == 0 { 0.95 } else { 0.05 };
            f[5] = if k < 2 { 0.95 } else { 0.05 };
            f[6] = 1.1;
            f[7] = 0.55;
        }
        let glass = mesh(plane_geo!(w, h), &m);
        glass.position().set((pane[0] + pane[1]) / 2.0, (pane[2] + pane[3]) / 2.0, FW_GLASS_Z);
        glass.set_render_order(2);
        body.add(&glass);
    }
    if lit {
        let glow = point_light!(0xffe4a0, 0.20, 1.5);
        glow.position().set(0, mount_y, -0.05);
        window_.add(&glow);
    }
    g.world.add(&window_);
}

/// The sash's four panes (x0, x1, y0, y1), the glass's distance from the
/// wall, how much of each runs under its bead (`bathroom/washroom_fittings.py` FW_*)
const FW_PANES: [[f64; 4]; 4] =
    [[-0.228, -0.010, -0.146, 0.006], [0.010, 0.228, -0.146, 0.006], [-0.228, -0.010, 0.026, 0.178], [0.010, 0.228, 0.026, 0.178]];

const FW_GLASS_Z: f64 = 0.0055;

const FW_UNDER: f64 = 0.004;

/// The window's paint (the frame part's box faces: its front laid across
/// u, v): gloss white gone the yellow-grey of old paint, chipped at the
/// arrises to the grey wood, dirt in the corners and along the board, dust
/// on what faces up
fn fw_paint_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 216, false, move |ctx, w, h| {
        let seed = vary ^ 0xf0a1;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 20.0, v * 17.0, 4);
            let fine = fbm(seed ^ 0x21, u * 90.0, v * 76.0, 2);
            let base = [0.74 + 0.04 * n, 0.71 + 0.04 * n, 0.62 + 0.03 * n];
            // chips: where the paint let go (most on the board and the foot
            // of the frame, where hands and things knocked it), to the grey
            // wood or the old cream undercoat
            let low = 1.0 - sstep(0.05, 0.45, v);
            let chip = sstep(0.79, 0.82, fbm(seed ^ 0x77, u * 34.0, v * 29.0, 3) + 0.08 * fine + 0.07 * low);
            let wood = [0.40 + 0.05 * fine, 0.36 + 0.04 * fine, 0.30 + 0.03 * fine];
            let undercoat = [0.62, 0.58, 0.48];
            let under = if fbm(seed ^ 0x13, u * 9.0, v * 9.0, 2) > 0.45 { wood } else { undercoat };
            let mut c = [0.0; 3];
            for k in 0..3 {
                c[k] = base[k] + (under[k] - base[k]) * chip;
            }
            // grime toward the foot (the board) and the corners
            let low = 1.0 - sstep(0.0, 0.3, v);
            // the old gloss crazed in a fine net of cracks
            let craze = (1.0 - (vn(seed ^ 0x2b, u * 160.0, v * 135.0) - 0.5).abs() * 14.0).max(0.0) * 0.12;
            for k in 0..3 {
                c[k] *= 1.0 - craze;
            }
            let corner = (1.0 - sstep(0.0, 0.12, u.min(1.0 - u))) * 0.5;
            let dirt = (low * 0.35 + corner * 0.3) * (0.6 + 0.6 * n);
            for k in 0..3 {
                c[k] *= 1.0 - dirt * 0.45;
            }
            // flecks of dirt and fly specks
            if hash(seed ^ 0x5, (u * 256.0) as i32, (v * 216.0) as i32) < 0.003 {
                for k in &mut c {
                    *k *= 0.7;
                }
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A frosted pane's dirt (grey: how much): dust settled along its foot and
/// in its corners, fly specks, a web tangled into one corner of one pane
fn fw_dirt_tex(vary: u32, k: usize) -> Tex {
    canvas_tex_opt(96, 72, false, move |ctx, w, h| {
        let seed = vary ^ (0x91 + k as u32 * 0x3d);
        let web = hash(seed, 7, k as i32) < 0.3;
        let wc = (hash(seed, 1, 2) < 0.5, hash(seed, 2, 1) < 0.5);
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 8.0, v * 6.0, 3);
            let foot = (1.0 - sstep(0.0, 0.25, v)) * 0.5;
            let du = u.min(1.0 - u);
            let dv = v.min(1.0 - v);
            let corner = (1.0 - sstep(0.0, 0.18, (du * du + dv * dv).sqrt())) * 0.5;
            let mut d = (0.12 + 0.18 * n + foot + corner).min(1.0);
            if hash(seed ^ 0x44, (u * 96.0) as i32, (v * 72.0) as i32) < 0.006 {
                d = (d + 0.35).min(1.0);
            }
            if web {
                let cu = if wc.0 { u } else { 1.0 - u };
                let cv = if wc.1 { v } else { 1.0 - v };
                let rr = (cu * cu + cv * cv).sqrt();
                if rr < 0.35 {
                    let a = cv.atan2(cu);
                    let radial = (1.0 - ((a * 9.0).sin().abs() * 40.0 * rr).min(1.0)) * 0.5;
                    let ring = (1.0 - (((rr * 60.0).sin().abs()) * 6.0).min(1.0)) * 0.35;
                    d = (d + (radial + ring) * (1.0 - rr / 0.35)).min(1.0);
                }
            }
            [d, d, d, 1.0]
        });
    })
}
