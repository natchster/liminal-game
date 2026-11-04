//! Batch `t03`'s props in this crate, made real (`docs/PROPS.md` P3): the
//! powder compact, the perfume flacon and the nail polish dropped on the
//! floor; their placers and what only they use
//! (`assets/src/blender/props/t03.py`, `t03.notes.md`).
use super::*;
use game::assets::props::prop_part_as_modelled;

/// The Blender part `name`, as modelled, in `grp` with `mat`
fn t03_put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let m = mesh(prop_part_as_modelled(name)?, mat);
    grp.add(&m);
    Some(m)
}

// R585 makeup compact: 8% chance per level. A pressed powder compact for a
// 59 mm pan (Blender batch `t03`): a moulded case, its lid on a three-knuckle
// hinge, the catch at the front; shut, or (the old 40%) open, its mirror gone
// black at the edges and spotted, the powder in its aluminium pan worn into a
// hollow by the puff and cracked where it was dropped. The old colours: a
// rose, black, gold-tone brass, ivory or teal case, scratched and scuffed.
pub(super) fn place_makeup_compacts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let case_colors = [0xd050a0, 0x202020, 0xe0c040, 0xffffff, 0x40a0a0];
    let col = choice(&case_colors);
    let kind = case_colors.iter().position(|&k| k == col).unwrap_or(0);
    let compact = group();
    let is_open = rnd() < 0.4; // 40% open
    let (px, pz) = jitter(g, x, z, 2.4);
    let salt = t03_kit::salt(px, pz);
    let (c, spec, shine) =
        [(0x9a5c6e, 0x3a3a3a, 60), (0x1e1d1f, 0x4a4a4a, 80), (0x6a5228, 0xa08850, 45), (0xd4ccbc, 0x3a3a3a, 50), (0x2f6664, 0x3a3a3a, 60)]
            [kind];
    let case = phong!({ map: &t03_kit::case_tex(kind, salt), color: c, specular: spec, shininess: shine });
    t03_put(&compact, "Real__makeupCompactBase", &case);
    // the lid turns open about the hinge's pin (y, z of `HINGE`, t03.py)
    let pivot = group();
    pivot.position().set(0, T03_HINGE.0, T03_HINGE.1);
    let lid = group();
    lid.position().set(0, -T03_HINGE.0, -T03_HINGE.1);
    pivot.add(&lid);
    compact.add(&pivot);
    t03_put(&lid, "Real__makeupCompactLid", &case);
    if kind != 2 {
        // the plastic ones' gilt trim round the lid, rubbed to the base metal
        t03_put(
            &lid,
            "Real__makeupCompactTrim",
            &phong!({ map: &t03_kit::case_tex(2, salt ^ 0x55), color: 0x6a5228, specular: 0xa08850, shininess: 45 }),
        );
    }
    if is_open {
        // as far as the hinge lets it
        pivot.rotation().x = -1.83;
        if let Some(m) = t03_put(&lid, "Real__makeupCompactMirror", &t03_kit::mirror_mat(&t03_kit::mirror_tex(salt))) {
            m.set_render_order(2);
        }
        t03_put(&compact, "Real__makeupCompactPan", &phong!({ color: 0x5a5c5e, specular: 0x8a8a8a, shininess: 50 }));
        t03_put(&compact, "Real__makeupCompactPowder", &lambert!({ map: &t03_kit::powder_tex(salt) }));
    }
    compact.position().set(px, 0, pz);
    compact.rotation().y = rnd() * PI * 2.0;
    g.world.add(&compact);
}

/// The compact's hinge pin: its height and how far back (t03.py prints it)
const T03_HINGE: (f64, f64) = (0.009, -0.0398);

// R603 perfume bottle: 5% chance per level. A 30 ml rectangular flacon
// (Blender batch `t03`): heavy-bottomed glass, the scent left in it gone
// darker and down to a fifth, a half or three quarters (a dried ring where it
// stood higher), its dip tube; a crimped spray pump and the square overcap on
// it, or off and lying by it; or (the old 40% bulb) a refill fitted with an
// atomizer's gilt collar and a netted rubber bulb on its hose, the bulb and
// its tassel lying on the floor. The old colours are the scent's.
pub(super) fn place_perfume_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let glass_colors = [0xe0c8a0, 0xd0a0c0, 0xa0c0e0, 0xf0d060, 0xe08080];
    let col = choice(&glass_colors);
    let kind = glass_colors.iter().position(|&k| k == col).unwrap_or(0);
    let perfume = group();
    let has_bulb = rnd() < 0.4;
    let (px, pz) = jitter(g, x, z, 2.4);
    let salt = t03_kit::salt(px, pz);
    // how much is left (no draw: from where it stands): a fifth now and
    // then, mostly a half or three quarters
    let level = [0, 1, 1, 2, 2][(salt >> 7) as usize % 5];
    let (lvl_name, lvl_y) =
        [("Real__perfumeFlaconScentLow", 0.0199), ("Real__perfumeFlaconScentMid", 0.0340), ("Real__perfumeFlaconScentHigh", 0.0480)][level];
    // the thick bottom: the solid the floor holds up
    t03_put(
        &perfume,
        "Real__perfumeFlaconGlassBase",
        &phong!({ color: 0x4e5a52, specular: 0xb0b0b0, shininess: 110, transparent: true, opacity: 0.6 }),
    );
    if let Some(m) = t03_put(
        &perfume,
        lvl_name,
        &phong!({ color: T03_SCENT[kind], specular: 0x707070, shininess: 90, transparent: true, opacity: 0.66, depthWrite: false }),
    ) {
        m.set_render_order(2);
    }
    if let Some(m) = t03_put(
        &perfume,
        "Real__perfumeFlaconDipTube",
        &phong!({ color: 0xc8ccc8, specular: 0x505050, shininess: 60, transparent: true, opacity: 0.5, depthWrite: false }),
    ) {
        m.set_render_order(2);
    }
    // the glass: its ring where the scent stood higher, dust, fingers
    let tide = (lvl_y + 0.006 + 0.010 * f64::from((salt >> 11) % 8) / 8.0 - 0.008) / 0.070;
    if let Some(m) = t03_put(&perfume, "Real__perfumeFlaconGlass", &t03_kit::glass_mat(&t03_kit::flacon_glass_tex(salt, tide))) {
        m.set_render_order(3);
    }
    let metal = |k: u32| match k % 3 {
        0 => phong!({ color: 0x6a5530, specular: 0xa89060, shininess: 50 }),
        1 => phong!({ color: 0x1c1b1c, specular: 0x505050, shininess: 70 }),
        _ => phong!({ color: 0x5c5e60, specular: 0xa0a0a0, shininess: 60 }),
    };
    if has_bulb {
        // an atomizer's head on the flacon's neck (6 mm higher than the
        // pressed glass's it was modelled on)
        let gilt = phong!({ map: &t03_kit::tarnish_tex(salt), color: 0x5e4822, specular: 0xa88c50, shininess: 45 });
        if let Some(m) = t03_put(&perfume, "Real__perfumeAtomizerCollar", &gilt) {
            m.position().y = 0.006;
        }
        let silk = T03_SILK[(salt >> 3) as usize % 4];
        let net = lambert!({ map: &t03_kit::net_tex(), color: silk });
        t03_put(&perfume, "Real__perfumeAtomizerBulb", &net);
        t03_put(&perfume, "Real__perfumeAtomizerHoseFlacon", &lambert!({ color: silk }));
        t03_put(&perfume, "Real__perfumeAtomizerTassel", &lambert!({ color: silk }));
    } else {
        let pump = metal(salt >> 5);
        t03_put(&perfume, "Real__perfumeFlaconPump", &pump);
        let cap_mat = metal(salt >> 5);
        if salt % 3 == 0 {
            // the cap off, lying on its side by it
            if let Some(geo) = prop_part_as_modelled("Real__perfumeFlaconCap") {
                let geo = geo.duplicate();
                geo.rotate_z(PI / 2.0);
                geo.rotate_y(0.4 + f64::from(salt % 7) * 0.2);
                let bb = geo.compute_bounding_box();
                geo.translate(0.048 - (bb.min.x + bb.max.x) / 2.0, -bb.min.y, 0.018 - (bb.min.z + bb.max.z) / 2.0);
                perfume.add(&mesh(geo, &cap_mat));
            }
        } else {
            t03_put(&perfume, "Real__perfumeFlaconCap", &cap_mat);
        }
    }
    perfume.position().set(px, 0, pz);
    perfume.rotation().y = rnd() * PI * 2.0;
    g.world.add(&perfume);
}

/// The scent left in it, darkened (sRGB): amber, rose, pale blue, gold, coral
const T03_SCENT: [u32; 5] = [0xb07a2a, 0xa85a6c, 0x6a8cac, 0xb89a2a, 0xb25a4c];
/// The bulb's silk net: crimson, faded rose, black, old gold
const T03_SILK: [u32; 4] = [0x6a1a26, 0x8a5a60, 0x262426, 0x7a6430];

// R603 nail polish bottle: 8% chance per level. A 13.5 ml bottle of nail
// polish (batch `r18`'s parts): heavy-bottomed square glass, a white screw cap
// with its brush, the lacquer dried down in it (pooled along its side when it
// fell over wet), a dried drip down the glass; no maker's name. The old
// colour draw picks the lacquer, dried dark.
pub(super) fn place_nail_polish(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let polish_colors = [0xc02040, 0xd050a0, 0x804060, 0x40a0c0, 0x808080, 0x202020, 0xe0d040];
    let polish_color = choice(&polish_colors);
    let kind = polish_colors.iter().position(|&k| k == polish_color).unwrap_or(0);
    let lacquer = if kind == 4 {
        // a silver: the shimmer catches the light
        phong!({ color: T03_LACQUER[kind], specular: 0x8a8a8a, shininess: 60 })
    } else {
        phong!({ color: T03_LACQUER[kind], specular: 0x4a4a4a, shininess: 90 })
    };
    let polish = group();
    t03_put(&polish, "Real__nailPolishCap", &phong!({ color: 0xd6d3cb, specular: 0x3a3a3a, shininess: 60 }));
    t03_put(&polish, "Real__nailPolishBrush", &lambert!({ color: 0xcfcab9 }));
    t03_put(&polish, "Real__nailPolishDrip", &lacquer);
    let (px, pz) = jitter(g, x, z, 2.4);
    polish.position().set(px, 0, pz);
    polish.rotation().y = rnd() * PI * 2.0;
    // 25% chance tipped over (it fell while the lacquer was wet)
    let tipped = rnd() < 0.25;
    t03_put(&polish, if tipped { "Real__nailPolishLacquerLying" } else { "Real__nailPolishLacquer" }, &lacquer);
    // the glass over what is in it; its thick bottom less clear, the solid
    // the floor holds up
    t03_put(
        &polish,
        "Real__nailPolishGlassBase",
        &phong!({ color: 0x7f8a82, specular: 0xc8c8c8, shininess: 110, transparent: true, opacity: 0.62 }),
    );
    t03_put(
        &polish,
        "Real__nailPolishGlass",
        &phong!({ color: 0xd2d6d0, specular: 0xc8c8c8, shininess: 110, transparent: true, opacity: 0.42 }),
    );
    if tipped {
        polish.rotation().z = PI / 2.0;
        polish.position().y = 0.0125;
    }
    g.world.add(&polish);
}

/// The lacquers dried dark in the glass (sRGB): red, pink, plum, teal, silver,
/// black, yellow
const T03_LACQUER: [u32; 7] = [0x8a1c28, 0x8e2a56, 0x5a2642, 0x1f5660, 0x6c6c70, 0x18191c, 0x8e7e26];

/// The compact's and the perfume's painters and materials
mod t03_kit {
    use super::*;

    /// A placed thing's own number, from where it stands (no draw): its
    /// painters start their strokes from it, so no two floors share them
    pub(super) fn salt(px: f64, pz: f64) -> u32 {
        ((px * 1013.0).floor() as i64 ^ ((pz * 733.0).floor() as i64).rotate_left(17)) as u32
    }

    // ---- painting kit (per-pixel, pure: safe for `par_rows`) ----
    fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    /// A thin line's darkness at distance d from it (soft, `w` wide)
    fn line(d: f64, w: f64) -> f64 {
        (1.0 - (d / w).abs()).max(0.0)
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right,
    /// v up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
        let iw = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                for k in 0..4 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    }

    /// Scratches as straight short strokes from a seeded scatter: the
    /// darkness (or lightness) of the nearest at (u, v)
    fn scratches(seed: u32, u: f64, v: f64, n: i32, len: f64) -> f64 {
        let mut s: f64 = 0.0;
        for k in 0..n {
            let (cx, cy) = (hash(seed, k, 1), hash(seed, k, 2));
            let a = hash(seed, k, 3) * 3.2;
            let (dx, dy) = (u - cx, v - cy);
            let along = dx * a.cos() + dy * a.sin();
            let across = -dx * a.sin() + dy * a.cos();
            if along.abs() < len * (0.4 + hash(seed, k, 4)) {
                s = s.max(line(across, 0.004) * (0.4 + 0.6 * hash(seed, k, 5)));
            }
        }
        s
    }

    /// The case's wear (a grey to multiply its colour by): scuffs, hairline
    /// scratches, grime in its seams; the gold-tone one's lid turned in
    /// rings and its plating rubbed through to the brass at the edges
    pub(super) fn case_tex(kind: usize, salt: u32) -> Tex {
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let seed = salt ^ 0xc0a7;
            fill(ctx, w, h, |u, v| {
                let n = fbm(seed, u * 7.0, v * 7.0, 3);
                let mut g = 0.84 + 0.12 * n;
                let sc = scratches(seed, u, v, 26, 0.12);
                let r = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt();
                if kind == 2 {
                    g += 0.06 * (r * 260.0).sin() * sstep(0.46, 0.2, r);
                    g += 0.25 * sstep(0.40, 0.48, r) * sstep(0.5, 0.75, fbm(seed ^ 1, u * 14.0, v * 14.0, 2));
                    g += 0.35 * sc;
                } else if kind == 1 {
                    // black shows its scratches pale
                    g += 0.9 * sc;
                } else {
                    g -= 0.25 * sc;
                }
                g -= 0.10 * sstep(0.47, 0.5, r);
                [g, g * 0.99, g * 0.97, 1.0]
            });
        })
    }

    /// The mirror's glass: silver behind it (`Patch::Glass`, as the wall
    /// mirror's: nearly all of the light back), its map where the silver
    /// has gone to black
    pub(super) fn mirror_mat(map: &Tex) -> Mat {
        let m = phong!({ color: 0xffffff, specular: 0x2a2a2a, shininess: 3000, transparent: true, depthWrite: false });
        {
            let mut mm = m.m();
            mm.map = Some(map.clone());
            mm.patch = Patch::Glass;
            mm.look_settled = true;
            mm.name = "compact_mirror".into();
        }
        {
            let pu = m.r().patch_uniforms.clone();
            let f = &mut pu.borrow_mut().f;
            f[0] = 0.13;
            f[4] = 0.5;
            f[5] = 0.6;
            f[6] = 0.3;
            f[7] = 0.0;
            f[8] = 1.0;
            f[9] = 0.018;
            f[10] = 0.017;
            f[11] = 0.015;
            f[12] = 0.0;
            f[13] = 0.86;
        }
        m
    }

    /// Where the mirror's silver has gone (grey: how much): black creeping
    /// in from its edge, a few spots and pinholes, a smear of powder
    pub(super) fn mirror_tex(salt: u32) -> Tex {
        canvas_tex_opt(64, 64, false, move |ctx, w, h| {
            let seed = salt ^ 0x3177;
            fill(ctx, w, h, |u, v| {
                let r = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
                let edge = sstep(0.80, 0.98, r + 0.25 * (fbm(seed, u * 9.0, v * 9.0, 3) - 0.5));
                let spot = sstep(0.90, 0.95, vn(seed ^ 2, u * 26.0, v * 26.0)) * sstep(0.4, 0.7, fbm(seed ^ 8, u * 4.0, v * 4.0, 2));
                let smear = 0.25 * sstep(0.6, 0.75, fbm(seed ^ 4, u * 3.0 + v * 6.0, v * 2.0, 2));
                let g = (0.06 + 0.9 * edge + 0.8 * spot + smear).clamp(0.0, 1.0);
                [g, g, g, 1.0]
            });
        })
    }

    /// The pressed powder seen from above: its shade (light beige, medium,
    /// tan), the hollow the puff wore (greyed and glazed with skin's oil, the
    /// pan's metal showing at its middle), cracks from a fall, a fingerprint
    pub(super) fn powder_tex(salt: u32) -> Tex {
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let seed = salt ^ 0x90d3;
            let base = [[0.90, 0.78, 0.66], [0.85, 0.70, 0.57], [0.78, 0.61, 0.48]][(salt >> 9) as usize % 3];
            fill(ctx, w, h, |u, v| {
                let r = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
                // the hollow, where t03.py wore it (x -4 mm, z +3 mm of 58)
                let dh = ((u - 0.431).powi(2) + (v - 0.448).powi(2)).sqrt() / 0.30;
                let worn = (-dh * dh * 1.6).exp();
                let n = fbm(seed, u * 30.0, v * 30.0, 3);
                let mut c = [0.0; 3];
                for k in 0..3 {
                    c[k] = base[k] * (0.94 + 0.10 * n) * (1.0 - 0.22 * worn);
                }
                // the pan's metal through the middle of the hollow
                let bare = sstep(0.25, 0.12, dh + 0.08 * (n - 0.5));
                for k in 0..3 {
                    c[k] = c[k] * (1.0 - bare) + [0.55, 0.56, 0.57][k] * bare;
                }
                // cracks: lines where the cake broke, from a cell lattice
                let (x, y) = (u * 3.0, v * 3.0);
                let (mut d1, mut d2) = (9.0f64, 9.0f64);
                for dj in -1..=1 {
                    for di in -1..=1 {
                        let (ii, jj) = (x.floor() as i32 + di, y.floor() as i32 + dj);
                        let fx = f64::from(ii) + hash(seed, ii, jj);
                        let fy = f64::from(jj) + hash(seed ^ 77, ii, jj);
                        let d = (fx - x).hypot(fy - y);
                        if d < d1 {
                            d2 = d1;
                            d1 = d;
                        } else if d < d2 {
                            d2 = d;
                        }
                    }
                }
                let crack = sstep(0.035, 0.0, d2 - d1) * sstep(0.45, 0.65, fbm(seed ^ 3, u * 2.5, v * 2.5, 2));
                for ch in c.iter_mut() {
                    *ch *= 1.0 - 0.55 * crack;
                }
                // the rim of the pan round it
                let rim = sstep(0.93, 0.99, r);
                for k in 0..3 {
                    c[k] = c[k] * (1.0 - rim) + 0.5 * rim;
                }
                [c[0], c[1], c[2], 1.0]
            });
        })
    }

    /// Clear glass as the windows' (`Patch::Glass`): the room seen in it by
    /// Fresnel, its dirt from the map's grey
    pub(super) fn glass_mat(map: &Tex) -> Mat {
        let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 240, transparent: true, depthWrite: false });
        {
            let mut mm = m.m();
            mm.map = Some(map.clone());
            mm.patch = Patch::Glass;
            mm.look_settled = true;
            mm.name = "perfume_glass".into();
        }
        {
            let pu = m.r().patch_uniforms.clone();
            let f = &mut pu.borrow_mut().f;
            f[0] = 0.45;
            f[4] = 0.5;
            f[5] = 0.6;
            f[6] = 0.3;
            f[7] = 0.0;
            f[8] = 1.0;
            f[9] = 0.22;
            f[10] = 0.20;
            f[11] = 0.17;
            f[12] = 0.0;
            f[13] = 0.08;
        }
        m
    }

    /// The flacon's glass (v up its side): dust heavier toward its shoulder,
    /// the brown ring the scent dried at when it stood at `tide`, a
    /// thumb's print
    pub(super) fn flacon_glass_tex(salt: u32, tide: f64) -> Tex {
        canvas_tex_opt(64, 128, false, move |ctx, w, h| {
            let seed = salt ^ 0xf1ac;
            fill(ctx, w, h, |u, v| {
                let d = fbm(seed, u * 6.0, v * 9.0, 3);
                let mut g = 0.10 + 0.22 * d + 0.25 * sstep(0.7, 1.0, v);
                g += 0.5 * line(v - tide - 0.01 * (u * 18.0).sin(), 0.018);
                let (fx, fy) = (u - 0.38, v - 0.40);
                let fr = (fx * fx / 0.012 + fy * fy / 0.02).sqrt();
                g += 0.25 * sstep(1.0, 0.6, fr) * (0.5 + 0.5 * (fr * 60.0 + fy * 20.0).sin());
                let g = g.clamp(0.0, 1.0);
                [g, g, g, 1.0]
            });
        })
    }

    /// Gilt brass tarnished dark in its knurls and patches (a grey)
    pub(super) fn tarnish_tex(salt: u32) -> Tex {
        canvas_tex_opt(32, 32, false, move |ctx, w, h| {
            let seed = salt ^ 0x7a2;
            fill(ctx, w, h, |u, v| {
                let g = 0.65 + 0.35 * sstep(0.35, 0.7, fbm(seed, u * 5.0, v * 5.0, 3));
                [g, g * 0.97, g * 0.92, 1.0]
            });
        })
    }

    /// The bulb's knotted silk net over its rubber: a diamond mesh (a grey)
    pub(super) fn net_tex() -> Tex {
        canvas_tex_opt(64, 64, false, |ctx, w, h| {
            fill(ctx, w, h, |u, v| {
                let (a, b) = ((u + v) * 9.0, (u - v) * 9.0);
                let da = (a - a.round()).abs();
                let db = (b - b.round()).abs();
                let cord = line(da, 0.16).max(line(db, 0.16));
                let knot = line(da, 0.2) * line(db, 0.2);
                let g = 0.45 + 0.55 * cord.max(knot * 1.5).min(1.0);
                [g, g, g, 1.0]
            });
        })
    }
}
