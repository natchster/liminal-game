//! Renderer setup, the pooled panel lights, `update_lights`, and the main
//! loop's draw distance, quality governor and dust.
//!
//! The light belongs to the place, not to where you stand: a clean region is
//! lit flat and pale yellow, a rotted or blacked-out one falls away to near
//! black, and every surface gets the light of its own spot (the light map,
//! one texel per tile, read by the shader's local-light patch).
use crate::base::records::*;
use crate::base::tables::*;
use crate::prelude::*;

pub const AMB_DARK: f64 = 0.11;
pub const AMB_LIT: f64 = 0.62;
pub const HEMI_DARK: f64 = 0.17;
pub const HEMI_LIT: f64 = 0.84;
/// a fully bright panel's spotlight (the fill above stays low, so the light
/// comes from the fixtures and falls off between them)
pub const PANEL_INTENSITY: f64 = 14.0;
pub const FOG_DEN_DARK: f64 = 0.055;
pub const FOG_DEN_LIT: f64 = 0.013;
/// The Building's haze: thin enough to see across the void (a lit view
/// reaches past 130 m), and grey, not the rooms' yellow
pub const BLD_FOG_DARK: f64 = 0.026;
pub const BLD_FOG_LIT: f64 = 0.0065;
pub fn bld_fog_lit() -> Color {
    Color::hex(0x3a3d3e)
}
pub fn bld_fog_dark() -> Color {
    Color::hex(0x07080a)
}
/// How far the far plane may reach, metres (the rooms, the Building)
pub const FAR_MAX: f64 = 90.0;
pub const BLD_FAR_MAX: f64 = 230.0;
pub fn fog_lit() -> Color {
    Color::hex(0xb3a874)
}
pub fn fog_rot() -> Color {
    Color::hex(0x070604)
}
pub fn fog_store() -> Color {
    Color::hex(0xc4c6c2)
}
pub fn ground_lit() -> Color {
    Color::hex(0xaaa176)
}
pub fn ground_rot() -> Color {
    Color::hex(0x1a1610)
}
/// what an open tile keeps of a lit neighbour's light
pub const LIGHT_SPILL: f64 = 0.42;
/// how much of its place's light a room off a walk of the Building has (a
/// recess: the spill from the walk before it lifts it to `LIGHT_SPILL` of
/// the walk's at least)
pub const ROOM_LIGHT: f64 = 0.7;
/// how lit the exit door's tile is at least (its sign and lamps)
pub const EXIT_LIGHT: f64 = 0.8;
/// tiles a panel's light counts for, in the map
pub const PANEL_REACH: f64 = 2.5;
pub const RF_DARK: u32 = 2;
pub const RF_MOSS: u32 = 4;
/// a switched-off panel's brightness in a fully lit room
pub const PANEL_OFF_TONE: f64 = 0.25;
/// panels lit at once; the rest are spares for fades
pub const POOL_WANT: usize = POOLED_LIGHTS - 2;
pub const SCUFF_COUNT: usize = 4;
pub const RIPPLE_COUNT: usize = 6;
pub const SPARKLE_COUNT: usize = 8;

impl Game {
    /// Renderer startup: scene background and fog, the ambient + hemisphere
    /// lights, the player's glow, the hallucination silhouette, the
    /// flashlight, the dust motes, the rain / scuff / ripple / splash /
    /// thunder / sparkle pools, the shared geometry and materials, and the
    /// pooled panel spotlights. `dust_sprite` is `Textures::dust_sprite`.
    pub fn init_render(&mut self, dust_sprite: &Tex) {
        self.background = Color::hex(0x070604);
        self.fog_color = Color::hex(0x070604);
        self.fog_density = 0.052;
        let cam = self.camera.clone();
        cam.set_rotation_order(EulerOrder::YXZ);
        // ambient + hemisphere lights
        let g = group();
        self.amb_light = ambient_light(0xa89a70, AMB_LIT);
        self.hemi_light = hemisphere_light(0xd0c48c, ground_lit(), HEMI_LIT);
        g.add(&self.amb_light);
        g.add(&self.hemi_light);
        self.scene.add(&g);
        // the light map is lit everywhere until the first level builds its own
        self.ll_on = 0.0;
        self.ll_map = Some(Tex::data_r8(vec![255], 1, 1));
        self.ll_grid = [1.0, 1.0, 0.5, 0.5];
        self.ll_scale = 1.0;
        self.ll_fog_ratio = FOG_DEN_LIT / FOG_DEN_DARK;
        // player wraparound glow
        self.player_light = point_light!(0xfff2cf, 2.4, 7, 1.6);
        self.scene.add(&self.player_light);
        // hallucination silhouettes
        crate::base::rng::with_native(|| {
            let standing = canvas_tex_opt(128, 256, false, |ctx, w, _h| {
                ctx.set_fill_style("rgba(0,0,0,0)");
                ctx.fill_rect(0, 0, w, 256);
                ctx.set_fill_style("rgba(6,6,6,0.95)");
                ctx.begin_path();
                ctx.arc(w / 2.0, 42, 22, 0, PI * 2.0);
                ctx.fill();
                ctx.begin_path();
                ctx.move_to(w / 2.0 - 36.0, 66);
                ctx.line_to(w / 2.0 + 36.0, 66);
                ctx.line_to(w / 2.0 + 26.0, 200);
                ctx.line_to(w / 2.0 - 26.0, 200);
                ctx.close_path();
                ctx.fill();
                ctx.fill_rect(w / 2.0 - 42.0, 78, 10, 96);
                ctx.fill_rect(w / 2.0 + 32.0, 78, 10, 96);
                ctx.fill_rect(w / 2.0 - 22.0, 200, 14, 54);
                ctx.fill_rect(w / 2.0 + 8.0, 200, 14, 54);
                ctx.set_fill_style("rgba(20,20,30,0.35)");
                ctx.begin_path();
                ctx.arc(w / 2.0, 42, 30, 0, PI * 2.0);
                ctx.fill();
            });
            let hunched = canvas_tex_opt(128, 256, false, |ctx, w, _h| {
                ctx.set_fill_style("rgba(0,0,0,0)");
                ctx.fill_rect(0, 0, w, 256);
                ctx.set_fill_style("rgba(6,6,6,0.95)");
                ctx.begin_path();
                ctx.arc(w / 2.0 - 4.0, 62, 20, 0, PI * 2.0);
                ctx.fill();
                ctx.begin_path();
                ctx.move_to(w / 2.0 - 34.0, 82);
                ctx.line_to(w / 2.0 + 30.0, 88);
                ctx.line_to(w / 2.0 + 20.0, 200);
                ctx.line_to(w / 2.0 - 24.0, 200);
                ctx.close_path();
                ctx.fill();
                ctx.begin_path();
                ctx.move_to(w / 2.0 - 38.0, 90);
                ctx.line_to(w / 2.0 - 26.0, 90);
                ctx.line_to(w / 2.0 - 12.0, 190);
                ctx.line_to(w / 2.0 - 22.0, 200);
                ctx.close_path();
                ctx.fill();
                ctx.begin_path();
                ctx.move_to(w / 2.0 + 30.0, 92);
                ctx.line_to(w / 2.0 + 40.0, 92);
                ctx.line_to(w / 2.0 + 30.0, 200);
                ctx.line_to(w / 2.0 + 18.0, 190);
                ctx.close_path();
                ctx.fill();
                ctx.fill_rect(w / 2.0 - 20.0, 200, 12, 44);
                ctx.fill_rect(w / 2.0 + 8.0, 200, 12, 44);
                ctx.set_fill_style("rgba(20,20,30,0.30)");
                ctx.begin_path();
                ctx.arc(w / 2.0 - 4.0, 62, 28, 0, PI * 2.0);
                ctx.fill();
            });
            let reaching = canvas_tex_opt(128, 256, false, |ctx, w, _h| {
                ctx.set_fill_style("rgba(0,0,0,0)");
                ctx.fill_rect(0, 0, w, 256);
                ctx.set_fill_style("rgba(6,6,6,0.95)");
                ctx.begin_path();
                ctx.arc(w / 2.0, 40, 22, 0, PI * 2.0);
                ctx.fill();
                ctx.begin_path();
                ctx.move_to(w / 2.0 - 34.0, 62);
                ctx.line_to(w / 2.0 + 34.0, 62);
                ctx.line_to(w / 2.0 + 24.0, 200);
                ctx.line_to(w / 2.0 - 24.0, 200);
                ctx.close_path();
                ctx.fill();
                ctx.begin_path();
                ctx.move_to(w / 2.0 + 30.0, 78);
                ctx.line_to(w / 2.0 + 60.0, 82);
                ctx.line_to(w / 2.0 + 60.0, 100);
                ctx.line_to(w / 2.0 + 30.0, 96);
                ctx.close_path();
                ctx.fill();
                ctx.begin_path();
                ctx.arc(w / 2.0 + 62.0, 91, 8, 0, PI * 2.0);
                ctx.fill();
                ctx.fill_rect(w / 2.0 - 42.0, 76, 10, 100);
                ctx.fill_rect(w / 2.0 - 22.0, 200, 14, 54);
                ctx.fill_rect(w / 2.0 + 8.0, 200, 14, 54);
                ctx.set_fill_style("rgba(20,20,30,0.35)");
                ctx.begin_path();
                ctx.arc(w / 2.0, 40, 30, 0, PI * 2.0);
                ctx.fill();
            });
            self.halluc_textures = vec![standing.clone(), hunched, reaching];
            self.halluc_shape =
                mesh(plane_geo!(0.9, 1.9), basic_mat!({ map: &standing, transparent: true, opacity: 0, depthWrite: false }));
            self.halluc_shape.set_visible(false);
            self.scene.add(&self.halluc_shape.clone());
        });
        // flashlight rig
        self.flashlight.set_pos(0, 0, 0);
        self.flashlight_target = node();
        self.scene.add(&self.flashlight.clone());
        self.scene.add(&self.flashlight_target.clone());
        self.flashlight.light().target = Some(self.flashlight_target.clone());
        // dust motes
        crate::base::rng::with_native(|| {
            let n = DUST_COUNT;
            let mut pos = vec![0f32; n * 3];
            let mut vel = vec![0f32; n * 3];
            for i in 0..n {
                pos[i * 3] = rand2(-15.0, 15.0) as f32;
                pos[i * 3 + 1] = rand2(0.4, WALL_H - 0.2) as f32;
                pos[i * 3 + 2] = rand2(-15.0, 15.0) as f32;
                vel[i * 3] = rand2(-0.08, 0.08) as f32;
                vel[i * 3 + 1] = rand2(-0.02, 0.02) as f32;
                vel[i * 3 + 2] = rand2(-0.08, 0.08) as f32;
            }
            self.dust_geo = Geo::new();
            self.dust_geo.set_position(pos.clone());
            self.dust_positions = pos;
            self.dust_velocities = vel;
        });
        self.dust_mat.set_map(dust_sprite);
        self.dust = points(&self.dust_geo, &self.dust_mat);
        self.scene.add(&self.dust.clone());
        // Rain streaks
        crate::base::rng::with_native(|| {
            self.rain_streak_tex = canvas_tex_opt(128, 256, false, |ctx, w, h| {
                ctx.set_fill_style("rgba(0,0,0,0)");
                ctx.fill_rect(0, 0, w, h);
                for _ in 0..40 {
                    let x = rand1(w);
                    let y = rand1(h);
                    let len = rand2(15.0, 55.0);
                    let alpha = rand2(0.10, 0.30);
                    ctx.set_fill_style(format!("rgba(210,220,235,{})", alpha));
                    ctx.fill_rect(x, y, 1, len);
                    if rnd() < 0.4 {
                        ctx.begin_path();
                        ctx.arc(x + 0.5, y, 1.4, 0, PI * 2.0);
                        ctx.set_fill_style(format!("rgba(220,230,240,{})", alpha + 0.10));
                        ctx.fill();
                    }
                }
            });
        });
        self.rain_streak_tex.set_repeat_wrapping();
        self.rain_windows.clear();
        // Scuffs
        self.scuffs.clear();
        for _ in 0..SCUFF_COUNT {
            let m = sprite_mat!({ map: dust_sprite, color: 0xa08060, transparent: true, depthWrite: false, blending: AdditiveBlending, opacity: 0 });
            let s = sprite(&m);
            s.scale().set_scalar(0.35);
            s.set_visible(false);
            self.scene.add(&s);
            self.scuffs.push(PooledFx { obj: s, t: 0.0 });
        }
        // Ripples
        crate::base::rng::with_native(|| {
            self.ripple_tex = canvas_tex_opt(96, 96, false, |ctx, w, h| {
                ctx.set_fill_style("rgba(0,0,0,0)");
                ctx.fill_rect(0, 0, w, h);
                for r in [w * 0.30, w * 0.42] {
                    let g = ctx.create_radial_gradient(w / 2.0, h / 2.0, r - 4.0, w / 2.0, h / 2.0, r + 4.0);
                    g.add_color_stop(0.0, "rgba(180,210,230,0)");
                    g.add_color_stop(0.5, "rgba(200,220,240,0.55)");
                    g.add_color_stop(1.0, "rgba(180,210,230,0)");
                    ctx.set_fill_style(&g);
                    ctx.fill_rect(0, 0, w, h);
                }
            });
        });
        // Splash flash light, thunder flash
        self.splash_flash_light = point_light!(0xb8d0f0, 0, 3.5, 2.0);
        self.splash_flash_light.set_pos(0, 0.4, 0);
        self.scene.add(&self.splash_flash_light.clone());
        self.thunder_flash_light = ambient_light(0xa8c0e0, 0);
        self.scene.add(&self.thunder_flash_light.clone());
        self.ripples.clear();
        for _ in 0..RIPPLE_COUNT {
            let m = mesh(
                plane_geo!(0.6, 0.6),
                basic_mat!({ map: &self.ripple_tex, transparent: true, depthWrite: false, opacity: 0, blending: NormalBlending }),
            );
            m.rotation().x = -PI / 2.0;
            m.position().y = 0.015;
            m.set_visible(false);
            self.scene.add(&m);
            self.ripples.push(PooledFx { obj: m, t: 0.0 });
        }
        // Sparkles
        self.sparkles.clear();
        for _ in 0..SPARKLE_COUNT {
            let m = sprite_mat!({ map: dust_sprite, color: 0xfff8d0, transparent: true, depthWrite: false, blending: AdditiveBlending, opacity: 0 });
            let s = sprite(&m);
            s.scale().set_scalar(0.18);
            s.set_visible(false);
            self.scene.add(&s);
            self.sparkles.push(PooledFx { obj: s, t: 0.0 });
        }
        // pooled panel spotlights: a full 90° spot, all of it penumbra
        self.pooled.clear();
        self.pooled_panel.clear();
        self.pooled_w.clear();
        for _ in 0..POOLED_LIGHTS {
            let l = spot_light!(0xffeec4, 0, 10, PI / 2.0, 1, 1.6);
            let t = node();
            l.light().target = Some(t.clone());
            self.scene.add(&l);
            self.scene.add(&t);
            self.pooled.push(l);
            self.pooled_panel.push(None);
            self.pooled_w.push(0.0);
        }
        self.resize();
    }

    /// (tile coords) — how lit a region is meant to be
    pub fn region_light_at(&self, x: i32, z: i32) -> f64 {
        if self.region_info.is_empty() || self.region_map.is_empty() {
            return 0.5;
        }
        let info = self.region_at(x, z);
        if info.flags & RF_DARK != 0 {
            return 0.0;
        }
        info.lit.unwrap_or(0.5)
    }

    /// One texel per tile, 0 dead … 1 lit
    pub fn build_light_map(&mut self) {
        let (w, h) = (self.w as usize, self.h as usize);
        let n = w * h;
        if n == 0 || self.region_info.is_empty() {
            return;
        }
        let open = |t: &Vec<u8>, i: usize| t[i] != 1;
        let mut k = vec![0f64; n];
        for i in 0..n {
            let info = &self.region_info[self.region_map[i] as usize];
            k[i] = if info.flags & RF_DARK != 0 { 0.0 } else { info.lit.unwrap_or(0.5) };
        }
        self.light_map_off = self.panels_off_sig();
        if self.light_map_off != 0.0 {
            let mut all = vec![0f64; n];
            let mut on = vec![0f64; n];
            let r = PANEL_REACH.ceil() as i32;
            for p in &self.panels {
                let (px, pz) = (self.tx(p.wx), self.tz(p.wz));
                for dz in -r..=r {
                    for dx in -r..=r {
                        let (x, z) = (px + dx, pz + dz);
                        let wgt = 1.0 - ((dx * dx + dz * dz) as f64).sqrt() / PANEL_REACH;
                        if wgt <= 0.0 || x < 0 || z < 0 || x >= self.w || z >= self.h {
                            continue;
                        }
                        let i = (z * self.w + x) as usize;
                        all[i] += wgt;
                        if !p.switched_off {
                            on[i] += wgt;
                        }
                    }
                }
            }
            for i in 0..n {
                if all[i] > 0.0 {
                    k[i] *= on[i] / all[i];
                }
            }
        }
        // a room off a walk of the Building is a recess, a storey of building
        // over it and on three sides: it has less of the place's light than
        // the open walk before it (counted as open, it was lit as the walk,
        // and its floor read brighter than the walk's: `MILESTONES.md` 11);
        // the spill below grades it from the doorway in
        if let Some(plan) = self.building.as_ref() {
            for r in &plan.rooms {
                let i = (r.tile.1 * self.w + r.tile.0) as usize;
                if i < n {
                    k[i] *= ROOM_LIGHT;
                }
            }
        }
        // the way down is lit: the exit sign and its lamps light the door's
        // tile whatever the region, so the door shows at the end of a dark
        // hall and its glow carries (the light spills from here like any
        // other). The Loop's door is an ordinary office door, unlit.
        if let Some(e) = self.exit_data.as_ref().filter(|_| self.level_scp.as_deref() != Some("loop")) {
            let (x, z) = (self.tx(e.pos.x), self.tz(e.pos.z));
            if x >= 0 && z >= 0 && x < self.w && z < self.h {
                let i = (z * self.w + x) as usize;
                k[i] = k[i].max(EXIT_LIGHT);
            }
        }
        // the vending machines still on: their glass and the floor before them
        for (x, z, l) in self.vending_light() {
            if x >= 0 && z >= 0 && x < self.w && z < self.h {
                let i = (z * self.w + x) as usize;
                k[i] = k[i].max(l);
            }
        }
        // her lamp, while it burns
        if let Some(c) = self.story.camp.as_ref().filter(|c| c.lit) {
            let i = (c.tile.1 * self.w + c.tile.0) as usize;
            if i < k.len() {
                k[i] = k[i].max(crate::story::dress::CAMP_LIGHT);
            }
        }
        let t = &self.tiles;
        for _ in 0..3 {
            let mut next = k.clone();
            for z in 0..h {
                for x in 0..w {
                    let i = z * w + x;
                    if !open(t, i) {
                        continue;
                    }
                    let mut v = k[i];
                    if x > 0 && open(t, i - 1) {
                        v = v.max(k[i - 1] * LIGHT_SPILL);
                    }
                    if x < w - 1 && open(t, i + 1) {
                        v = v.max(k[i + 1] * LIGHT_SPILL);
                    }
                    if z > 0 && open(t, i - w) {
                        v = v.max(k[i - w] * LIGHT_SPILL);
                    }
                    if z < h - 1 && open(t, i + w) {
                        v = v.max(k[i + w] * LIGHT_SPILL);
                    }
                    next[i] = v;
                }
            }
            k = next;
        }
        let mut data = vec![0u8; n];
        for z in 0..h {
            for x in 0..w {
                let i = z * w + x;
                let mut v = k[i];
                if !open(t, i) {
                    let (mut s, mut c) = (0.0, 0);
                    if x > 0 && open(t, i - 1) {
                        s += k[i - 1];
                        c += 1;
                    }
                    if x < w - 1 && open(t, i + 1) {
                        s += k[i + 1];
                        c += 1;
                    }
                    if z > 0 && open(t, i - w) {
                        s += k[i - w];
                        c += 1;
                    }
                    if z < h - 1 && open(t, i + w) {
                        s += k[i + w];
                        c += 1;
                    }
                    if c > 0 {
                        v = s / c as f64;
                    }
                }
                data[i] = round_half_up(clamp(v, 0.0, 1.0) * 255.0) as u8;
            }
        }
        self.light_map_cpu = data.iter().map(|&b| b as f64 / 255.0).collect();
        // the shape beside the light, at `FINE` texels a tile: how open each
        // place is and its ceiling, for the corners, and the floor under it,
        // for the floor's occlusion (`visuals::occlusion`)
        // (made once a level: only the light changes as panels switch)
        use crate::visuals::occlusion::{pack, LightShape, FINE};
        if !self.ll_shape.as_ref().is_some_and(|s| s.fits(self.w, self.h)) {
            let shape =
                LightShape::new(self.w, self.h, &self.tiles, &self.tile_ceil, self.solids.as_ref(), |u, v| self.light_floor_at(u, v));
            self.ll_shape = Some(shape);
        }
        let Some(shape) = self.ll_shape.as_ref() else { return };
        let data = pack(self.w, self.h, &data, &shape.channels);
        let (wf, hf) = ((w * FINE) as u32, (h * FINE) as u32);
        let reuse = self.ll_map.as_ref().map(|m| m.width() == wf && m.height() == hf).unwrap_or(false);
        if reuse {
            let m = self.ll_map.clone().unwrap();
            let img = m.image();
            let mut im = img.borrow_mut();
            im.data = data;
            im.version += 1;
        } else {
            let tex = Tex::data_rgba(data, wf, hf);
            {
                let mut tt = tex.t();
                tt.mag_filter = Filter::Linear;
                tt.min_filter = Filter::Linear;
            }
            self.ll_map = Some(tex);
        }
        self.ll_grid = [1.0 / (w as f64 * CELL), 1.0 / (h as f64 * CELL), 0.5, 0.5];
    }

    /// The height of the light map's floor code 0 (m; `occlusion::floor_codes`)
    pub fn light_floor_base(&self) -> f64 {
        self.ll_shape.as_ref().map_or(0.0, |s| s.base)
    }
    /// Some places of the light map have no floor (the Building's void and
    /// masses), and its B holds whether each has one (`occlusion::LightShape`)
    pub fn light_floor_gaps(&self) -> bool {
        self.ll_shape.as_ref().is_some_and(|s| s.gaps)
    }

    /// Which panels are switched off, as one number
    pub fn panels_off_sig(&self) -> f64 {
        self.panels.iter().enumerate().filter(|(_, p)| p.switched_off).map(|(i, _)| (i + 1) as f64).sum()
    }

    /// (world coords) — the light map, filtered as the GPU filters it
    pub fn light_map_at(&self, x: f64, z: f64) -> f64 {
        let (w, h) = (self.w as usize, self.h as usize);
        if self.light_map_cpu.is_empty() || self.light_map_cpu.len() != w * h {
            return 1.0;
        }
        let u = clamp(x / CELL + w as f64 / 2.0 - 0.5, 0.0, (w - 1) as f64);
        let v = clamp(z / CELL + h as f64 / 2.0 - 0.5, 0.0, (h - 1) as f64);
        let (x0, z0) = (u.floor() as usize, v.floor() as usize);
        let (x1, z1) = ((x0 + 1).min(w - 1), (z0 + 1).min(h - 1));
        let m = &self.light_map_cpu;
        lerp(lerp(m[z0 * w + x0], m[z0 * w + x1], u - x0 as f64), lerp(m[z1 * w + x0], m[z1 * w + x1], u - x0 as f64), v - z0 as f64)
            * self.ll_scale
    }

    /// The floor under a place of the light map (`u`, `v` in tile units, as
    /// `floor_y_at` has it): the Building's floor there (a flight's where it
    /// is along its run), none over the void or in a mass; the rooms stand
    /// on 0, but in a pool
    fn light_floor_at(&self, u: f64, v: f64) -> Option<f64> {
        let (x, z) = ((u - self.w as f64 / 2.0) * CELL, (v - self.h as f64 / 2.0) * CELL);
        if !self.ground.is_empty() {
            let g = self.ground.get((v.floor() as i32 * self.w + u.floor() as i32) as usize)?;
            return g.walkable().then(|| self.floor_y_at(x, z));
        }
        match &self.pool_bounds {
            Some(p) if x > p.min_x && x < p.max_x && z > p.min_z && z < p.max_z => Some(crate::core_props::tail::POOL_FLOOR),
            _ => Some(0.0),
        }
    }

    /// How much lit surfaces lose of their indirect light toward the floor
    /// under them (the light map's: the Building's storeys, a pool's floor);
    /// none on the Stairwell's flights, which the light map does not hold
    pub fn floor_occlusion(&self) -> f64 {
        if self.in_stairwell {
            0.0
        } else {
            1.0
        }
    }

    /// How much a corner darkens where walls meet each other, the floor and
    /// the ceiling (`visuals::occlusion`): the rooms', whose shape the light
    /// map holds; not the Building's floors, stacked over each other, nor
    /// the Stairwell's flights or the Loop's rooms; and never where the
    /// map's B holds whether a place has a floor rather than its ceiling
    /// (`light_floor_gaps`)
    pub fn corner_occlusion(&self) -> f64 {
        if self.building.is_some() || self.in_stairwell || self.in_loop || self.light_floor_gaps() {
            0.0
        } else {
            1.0
        }
    }

    pub fn update_ambience(&mut self) {
        self.ll_on = 1.0;
        self.ll_scale = 1.0 - 0.97 * self.store_state.lights_out;
        self.amb_light.set_intensity(AMB_LIT);
        self.hemi_light.set_intensity(HEMI_LIT);
        self.hemi_light.light().ground_color = ground_lit();
        let a = self.amb_light.light_color();
        let s = self.hemi_light.light_color();
        self.ll_amb_d = Vec3::new(a.r, a.g, a.b) * (AMB_DARK - AMB_LIT);
        self.ll_sky_d = Vec3::new(s.r, s.g, s.b) * (HEMI_DARK - HEMI_LIT);
        let (gr, gl) = (ground_rot(), ground_lit());
        self.ll_gnd_d =
            Vec3::new(gr.r * HEMI_DARK - gl.r * HEMI_LIT, gr.g * HEMI_DARK - gl.g * HEMI_LIT, gr.b * HEMI_DARK - gl.b * HEMI_LIT);
        let bld = self.building.is_some();
        self.fog_density = if bld { BLD_FOG_DARK } else { FOG_DEN_DARK };
        self.ll_fog_ratio = if bld { BLD_FOG_LIT / BLD_FOG_DARK } else { FOG_DEN_LIT / FOG_DEN_DARK };
        self.fog_color = if bld { bld_fog_dark() } else { fog_rot() };
        self.ll_fog_lit = if bld {
            bld_fog_lit()
        } else if self.store_state.on {
            fog_store()
        } else {
            fog_lit()
        };
        let k = self.light_map_at(self.camera.pos().x, self.camera.pos().z);
        let mut bg = self.fog_color;
        bg.lerp(&self.ll_fog_lit, k);
        self.background = bg;
        // inside the self-assembly's fog, the fog is all you see (sealed in
        // a cell, it does not come in)
        if bld && self.assembly.up() && self.assembly.sealed.is_none() && self.camera.pos().y < self.assembly.y {
            let violet = Color::hex(0x2a1a30);
            self.fog_color = violet;
            self.ll_fog_lit = violet;
            self.fog_density = 0.16;
            self.ll_fog_ratio = 1.0;
            self.background = violet;
        }
    }

    /// The camera follows the window's aspect
    pub fn resize(&mut self) {
        let aspect = if self.view_height > 0.0 { self.view_width / self.view_height } else { 1.0 };
        self.camera.camera().aspect = aspect;
    }

    /// Drop every panel the pooled lights hold
    pub fn release_pooled_lights(&mut self) {
        for i in 0..self.pooled.len() {
            self.pooled_panel[i] = None;
            self.pooled_w[i] = 0.0;
            self.pooled[i].set_intensity(0);
        }
        self.pool_t = 0.0;
    }

    pub fn update_lights(&mut self, dt: f64, _t: f64) {
        self.update_ambience();
        if self.panels_off_sig() != self.light_map_off {
            self.build_light_map();
        }
        let mut dirty = false;
        for i in 0..self.panels.len() {
            if self.panels[i].switched_off {
                if self.panels[i].bright != 0.0 {
                    self.panels[i].bright = 0.0;
                    dirty = true;
                }
                self.panels[i].flicker = 0.0;
            } else if self.panels[i].flicker > 0.0 {
                self.panels[i].flicker -= dt;
                self.panels[i].bright = if rnd() < 0.45 { rand2(0.05, 0.3) } else { 1.0 };
                dirty = true;
            } else {
                if self.panels[i].bright != 1.0 {
                    self.panels[i].bright = 1.0;
                    dirty = true;
                }
                let rot = self.panels[i].rot;
                if rnd() < dt * (0.012 + 0.20 * rot) {
                    self.panels[i].flicker = rand2(0.25, 1.1);
                }
            }
            if dirty {
                let p = &self.panels[i];
                let v = if p.switched_off { PANEL_OFF_TONE * self.light_map_at(p.wx, p.wz) } else { p.bright };
                let mut c = Color::default();
                c.set_scalar(v);
                if let Some(pm) = &self.panel_mesh {
                    pm.set_color_at(i, &c);
                }
            }
        }
        // the pooled lights go to the panels nearest the player, fading in and out
        self.pool_t -= dt;
        if self.pool_t <= 0.0 {
            self.pool_t = 0.3;
            let (px, pz) = (self.player.pos.x, self.player.pos.z);
            let mut sorted: Vec<(usize, f64)> =
                self.panels.iter().enumerate().map(|(i, p)| (i, (p.wx - px).powi(2) + (p.wz - pz).powi(2))).collect();
            sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            self.pool_edge = if sorted.len() > POOL_WANT { sorted[POOL_WANT].1.sqrt() } else { f64::INFINITY };
            sorted.truncate(POOL_WANT);
            self.pool_wanted.clear();
            for s in &sorted {
                self.pool_wanted.insert(s.0);
            }
            let held: std::collections::HashSet<usize> = self.pooled_panel.iter().flatten().copied().collect();
            for s in &sorted {
                if held.contains(&s.0) {
                    continue;
                }
                let Some(li) = self.pooled_panel.iter().position(|q| q.is_none()) else { break };
                let p = &self.panels[s.0];
                let py = if p.y != 0.0 { p.y } else { WALL_H };
                let l = &self.pooled[li];
                // (aimed at the floor under it: on the Building's floors that has height)
                let fy = self.floor_y_at(p.wx, p.wz);
                l.set_pos(p.wx, py - 0.1, p.wz);
                if let Some(t) = l.target() {
                    t.set_pos(p.wx, fy, p.wz);
                }
                l.light().distance = 10f64.max((py - fy) * 2.8);
                self.pooled_panel[li] = Some(s.0);
                self.pooled_w[li] = 0.0;
            }
        }
        let k_fade = (dt * 5.0).min(1.0);
        for li in 0..self.pooled.len() {
            let Some(pi) = self.pooled_panel[li] else { continue };
            let mut want = 0.0;
            if self.pool_wanted.contains(&pi) {
                let p = &self.panels[pi];
                let d = (p.wx - self.player.pos.x).hypot(p.wz - self.player.pos.z);
                want = 1.0 - smoothstep(d, self.pool_edge * 0.8, self.pool_edge);
            }
            self.pooled_w[li] += (want - self.pooled_w[li]) * k_fade;
            if want == 0.0 && self.pooled_w[li] < 0.02 {
                self.pooled_panel[li] = None;
                self.pooled_w[li] = 0.0;
                self.pooled[li].set_intensity(0);
            }
        }
        let wk = self.weather_key();
        let cry_panel_peak = cry_panel_peak_by_weather(wk).unwrap_or(cry_panel_peak_by_weather("normal").unwrap_or(0.0));
        let mut dim_scale = 1.0 - cry_panel_peak * self.cry_dim_lerp;
        if self.electric_surge_t > 0.0 {
            dim_scale *= 0.4 + 0.6 * (1.0 - self.electric_surge_t / 0.35);
        }
        if self.door_dim_pulse_t > 0.0 {
            self.door_dim_pulse_t -= dt;
            if self.door_dim_pulse_t > 0.0 {
                let door_dim = door_dim_by_weather(wk).unwrap_or(door_dim_by_weather("normal").unwrap_or(0.0));
                let env = (PI * self.door_dim_pulse_t / 0.5).sin();
                dim_scale *= 1.0 - door_dim * env;
            }
        }
        if self.ding_bright_pulse_t > 0.0 {
            self.ding_bright_pulse_t -= dt;
            if self.ding_bright_pulse_t > 0.0 {
                let ding = ding_bright_by_weather(wk).unwrap_or(ding_bright_by_weather("normal").unwrap_or(0.0));
                let env = (PI * self.ding_bright_pulse_t / 0.4).sin();
                dim_scale *= 1.0 + ding * env;
            }
        }
        if self.settle_visual_t > 0.0 {
            self.settle_visual_t -= dt;
            if self.settle_visual_t > 0.0 {
                let env = (PI * (2.0 - self.settle_visual_t) / 2.0).sin();
                dim_scale *= 1.0 + 0.05 * env;
            }
        }
        let now = self.clock_ms() / 1000.0;
        let panel_descent_boost = self.descent_boost_mul(0.4);
        let amp_target = panel_lfo_amp_by_weather(wk).unwrap_or(panel_lfo_amp_by_weather("normal").unwrap_or(0.0)) * panel_descent_boost;
        let freq_target = panel_lfo_freq_mul_by_weather(wk).unwrap_or(1.0) * self.descent_freq_boost();
        let k_panel = (dt * 1.5).min(1.0);
        self.panel_lfo_amp_current += (amp_target - self.panel_lfo_amp_current) * k_panel;
        self.panel_lfo_freq_mul_current += (freq_target - self.panel_lfo_freq_mul_current) * k_panel;
        let dim_floor = panel_dim_floor_by_weather(wk).unwrap_or(panel_dim_floor_by_weather("normal").unwrap_or(1.0));
        let battery_dim = self.beam_battery_scale(dim_floor);
        let dim_radius = panel_dim_radius_by_weather(wk).unwrap_or(panel_dim_radius_by_weather("normal").unwrap_or(4.0));
        let dim_r2 = dim_radius * dim_radius;
        let dim_falloff = panel_dim_falloff_by_weather(wk).unwrap_or(panel_dim_falloff_by_weather("normal").unwrap_or(2.0));
        let depth = self.level_depth_mul();
        let voice_env = if self.voice_hue_shift_t > 0.0 { self.voice_hue_shift_t / 1.0 } else { 0.0 };
        for li in 0..self.pooled.len() {
            let Some(pi) = self.pooled_panel[li] else { continue };
            let p = self.panels[pi].clone();
            let l = self.pooled[li].clone();
            let mut mul = 1.0;
            if p.bright > 0.85 && self.clock_skip_t <= 0.0 {
                mul = 1.0
                    + (now * p.lfo_freq * self.panel_lfo_freq_mul_current * PI * 2.0 + p.lfo_phase + p.lfo_phase_offset).sin()
                        * self.panel_lfo_amp_current;
            }
            let mut b_dim = 1.0;
            let lp = l.pos();
            if battery_dim < 1.0 {
                let (dx, dz) = (lp.x - self.player.pos.x, lp.z - self.player.pos.z);
                let d2 = dx * dx + dz * dz;
                if d2 < dim_r2 {
                    let near = clamp(1.0 - d2.sqrt() / dim_radius, 0.0, 1.0);
                    b_dim = 1.0 - (1.0 - battery_dim) * near.powf(dim_falloff);
                }
            }
            let h_boost = if p.h_boost != 0.0 { p.h_boost } else { 1.0 };
            l.set_intensity(p.bright * PANEL_INTENSITY * dim_scale * mul * b_dim * depth * h_boost * self.pooled_w[li]);
            if self.knock_tint_t > 0.0 {
                let env = self.knock_tint_t / 0.7;
                let yaw = self.player.yaw;
                let (rx, rz) = (yaw.cos(), -yaw.sin());
                let (dxp, dzp) = (lp.x - self.player.pos.x, lp.z - self.player.pos.z);
                let side = dxp * rx + dzp * rz;
                let mtch = if side * self.knock_tint_pan > 0.0 { 1.0 } else { 0.0 };
                let tint = env * mtch * 0.35;
                let kt = knock_tint_by_weather(wk).or(knock_tint_by_weather("normal")).unwrap();
                l.light().color.set_rgb(1.0 + (kt.r - 1.0) * tint, 0.933 + (kt.g - 0.933) * tint, 0.769 + (kt.b - 0.769) * tint);
            } else if voice_env > 0.0 {
                let vh = voice_hue_by_weather(wk).or(voice_hue_by_weather("normal")).unwrap();
                let shift = voice_env * 0.25;
                l.light().color.set_rgb(1.0 + (vh.r - 1.0) * shift, 0.933 + (vh.g - 0.933) * shift, 0.769 + (vh.b - 0.769) * shift);
            } else {
                l.light().color.set_rgb(1.0, 0.933, 0.769);
            }
        }
        if self.voice_hue_shift_t > 0.0 {
            self.voice_hue_shift_t = (self.voice_hue_shift_t - dt).max(0.0);
        }
        if self.knock_tint_t > 0.0 {
            self.knock_tint_t = (self.knock_tint_t - dt).max(0.0);
        }
        let pl_weather = player_light_scale_by_weather(wk).or(player_light_scale_by_weather("normal")).unwrap();
        let pl_battery = self.beam_battery_scale(0.5);
        let int_descent = 0.65 + 0.35 * self.level_depth_mul();
        let cry_beam_peak = cry_beam_peak_by_weather(wk).unwrap_or(cry_beam_peak_by_weather("normal").unwrap_or(0.0));
        let cry_beam_mul = 1.0 - cry_beam_peak * self.cry_dim_lerp;
        self.player_light.set_intensity(PLAYER_LIGHT_BASE_INTENSITY * pl_weather.int_mul * pl_battery * int_descent * cry_beam_mul);
    }

    /// The far plane reaches the farthest tile still visible through its own haze
    pub fn sync_far_to_fog(&mut self) {
        let want;
        let (w, h) = (self.w as usize, self.h as usize);
        if self.ll_on > 0.5 && !self.light_map_cpu.is_empty() && self.light_map_cpu.len() == w * h {
            let cp = self.camera.pos();
            let (cx, cz, r) = (cp.x, cp.z, CELL * 0.71);
            let (ratio, scale) = (self.ll_fog_ratio, self.ll_scale);
            let yaw = self.camera.rot().y;
            let (fx, fz) = (-yaw.sin(), -yaw.cos());
            let (fov, aspect) = {
                let c = self.camera.camera();
                (c.fov, c.aspect)
            };
            let half = (deg_to_rad(fov / 2.0).tan() * aspect).atan() + 0.2;
            let mut wv: f64 = 0.0;
            for z in 0..h {
                let dz = self.wz(z) - cz;
                for x in 0..w {
                    let dx = self.wx(x) - cx;
                    let d = (dx * dx + dz * dz).sqrt();
                    if d > r && clamp((dx * fx + dz * fz) / d, -1.0, 1.0).acos() > half + (r / d).min(1.0).asin() {
                        continue;
                    }
                    let reach = 2.2 / (self.fog_density * (1.0 + (ratio - 1.0) * self.light_map_cpu[z * w + x] * scale));
                    if d - r < reach {
                        wv = wv.max(reach.min(d + r));
                    }
                }
            }
            want = clamp(wv, 28.0, if self.building.is_some() { BLD_FAR_MAX } else { FAR_MAX });
        } else {
            want = clamp(2.2 / self.fog_density.max(1e-4), 28.0, if self.building.is_some() { BLD_FAR_MAX } else { FAR_MAX });
        }
        let far = self.camera.camera().far;
        if (far - want).abs() > 1.5 {
            self.camera.camera().far = want;
        }
    }

    /// Trade resolution for frame rate
    pub fn quality_governor(&mut self, dt: f64) {
        if self.state != "playing" || dt <= 0.0 || dt >= 0.049 {
            return;
        }
        let q = &mut self.quality;
        q.ema += (dt - q.ema) * 0.05;
        if q.ema > 1.0 / 45.0 {
            q.slow_t += dt;
            q.fast_t = 0.0;
        } else if q.ema < 1.0 / 75.0 {
            q.fast_t += dt;
            q.slow_t = 0.0;
        } else {
            q.slow_t = 0.0;
            q.fast_t = 0.0;
        }
        let mut next = q.pr;
        if q.slow_t > 2.0 && q.pr > q.min_pr {
            next = q.min_pr.max(q.pr - 0.25);
        } else if q.fast_t > 8.0 && q.pr < q.max_pr {
            next = q.max_pr.min(q.pr + 0.25);
        }
        if next != q.pr {
            q.pr = next;
            q.slow_t = 0.0;
            q.fast_t = 0.0;
            q.changes += 1;
            self.resize();
        }
    }

    /// Motes, the player's glow tint and reach, sparkles
    pub fn update_dust(&mut self, dt: f64) {
        let fl = self.player.flashlight_on;
        let b_scale = self.beam_battery_scale(0.4);
        // a mote is a glint of dust a few millimetres across at most, seen
        // in the lamp's beam and faintly in the room's light; at 4.5-6.5 cm
        // each one near a wall read as a glowing speck (`docs/PROPS.md`, M8)
        {
            let mut m = self.dust_mat.m();
            m.opacity = if fl { 0.50 * b_scale } else { 0.20 };
            m.size = if fl { 0.018 + 0.009 * b_scale } else { 0.016 };
        }
        let wk = self.weather_key();
        let tint_hex = dust_tint_by_weather(wk).unwrap_or(dust_tint_by_weather("normal").unwrap_or(16777215.0)) as u32;
        let dust_depth = 0.75 + 0.25 * self.level_depth_mul();
        let mut tr = (((tint_hex >> 16) & 0xff) as f64 / 255.0) * dust_depth;
        let mut tg = (((tint_hex >> 8) & 0xff) as f64 / 255.0) * dust_depth;
        let mut tb = ((tint_hex & 0xff) as f64 / 255.0) * dust_depth;
        if fl {
            let warm = 1.0 - b_scale;
            tr = tr * (1.0 - warm) + 1.0 * warm;
            tg = tg * (1.0 - warm) + 0.55 * warm;
            tb = tb * (1.0 - warm) + 0.25 * warm;
        }
        let k = (dt * 1.5).min(1.0);
        {
            let mut m = self.dust_mat.m();
            m.color.r += (tr - m.color.r) * k;
            m.color.g += (tg - m.color.g) * k;
            m.color.b += (tb - m.color.b) * k;
        }
        let pl_hex = player_light_tint_by_weather(wk).unwrap_or(player_light_tint_by_weather("normal").unwrap_or(16773839.0)) as u32;
        let pl_depth = 0.7 + 0.3 * self.level_depth_mul();
        let (plr, plg, plb) = (
            (((pl_hex >> 16) & 0xff) as f64 / 255.0) * pl_depth,
            (((pl_hex >> 8) & 0xff) as f64 / 255.0) * pl_depth,
            ((pl_hex & 0xff) as f64 / 255.0) * pl_depth,
        );
        {
            let mut l = self.player_light.light();
            l.color.r += (plr - l.color.r) * k;
            l.color.g += (plg - l.color.g) * k;
            l.color.b += (plb - l.color.b) * k;
        }
        let pl_w = player_light_scale_by_weather(wk).or(player_light_scale_by_weather("normal")).unwrap();
        let dist_battery = self.beam_battery_scale(0.6);
        let dist_descent = 0.5 + 0.5 * self.level_depth_mul();
        let dist_target = PLAYER_LIGHT_BASE_DISTANCE * pl_w.dist_mul * dist_battery * dist_descent;
        {
            let mut l = self.player_light.light();
            l.distance += (dist_target - l.distance) * k;
        }
        let yaw = self.player.yaw;
        let (fx, fz) = (-yaw.sin(), -yaw.cos());
        let (ppx, ppz) = (self.player.pos.x, self.player.pos.z);
        {
            let arr = &mut self.dust_positions;
            let vel = &mut self.dust_velocities;
            for i in 0..DUST_COUNT {
                let ix = i * 3;
                // f32 storage: each element is stored back as f32 after the f64 step
                arr[ix] = (arr[ix] as f64 + vel[ix] as f64 * dt) as f32;
                arr[ix + 1] = (arr[ix + 1] as f64 + vel[ix + 1] as f64 * dt) as f32;
                arr[ix + 2] = (arr[ix + 2] as f64 + vel[ix + 2] as f64 * dt) as f32;
                let relx = arr[ix] as f64 - ppx;
                let relz = arr[ix + 2] as f64 - ppz;
                let y = arr[ix + 1] as f64;
                if relx.abs() > 12.0 || relz.abs() > 12.0 || !(0.2..=WALL_H - 0.1).contains(&y) {
                    if fl {
                        let along = 1.5 + rnd() * 6.5;
                        let side = rand2(-3.0, 3.0);
                        arr[ix] = (ppx + fx * along - fz * side) as f32;
                        arr[ix + 2] = (ppz + fz * along + fx * side) as f32;
                    } else {
                        arr[ix] = (ppx + rand2(-11.0, 11.0)) as f32;
                        arr[ix + 2] = (ppz + rand2(-11.0, 11.0)) as f32;
                    }
                    arr[ix + 1] = rand2(0.5, WALL_H - 0.2) as f32;
                    vel[ix] = rand2(-0.08, 0.08) as f32;
                    vel[ix + 1] = rand2(-0.02, 0.02) as f32;
                    vel[ix + 2] = rand2(-0.08, 0.08) as f32;
                }
            }
        }
        {
            let mut gg = self.dust_geo.g();
            gg.position.copy_from_slice(&self.dust_positions);
            gg.touch();
        }
        let spark_life = 0.4;
        for si in 0..self.sparkles.len() {
            let sp = self.sparkles[si].obj.clone();
            if self.sparkles[si].t > 0.0 {
                self.sparkles[si].t -= dt;
                if self.sparkles[si].t <= 0.0 {
                    sp.set_visible(false);
                    sp.mat().m().opacity = 0.0;
                } else {
                    let p = self.sparkles[si].t / spark_life;
                    sp.mat().m().opacity = p * 0.9;
                    sp.scale().set_scalar(0.04 + (1.0 - p) * 0.025);
                }
            } else if fl && rnd() < dt * 1.8 * self.beam_battery_scale(0.3) {
                let idx = to_i32(rnd() * DUST_COUNT as f64) as usize;
                let ix = idx * 3;
                let a = &self.dust_positions;
                sp.position().set(a[ix], a[ix + 1], a[ix + 2]);
                let spark_hex = dust_tint_by_weather(wk).unwrap_or(dust_tint_by_weather("normal").unwrap_or(16777215.0)) as u32;
                let (sr, sg, sb) = (
                    ((spark_hex >> 16) & 0xff) as f64 / 255.0,
                    ((spark_hex >> 8) & 0xff) as f64 / 255.0,
                    (spark_hex & 0xff) as f64 / 255.0,
                );
                let (mut br, mut bg, mut bb) = (sr * 0.7 + 0.3, sg * 0.7 + 0.3, sb * 0.7 + 0.3);
                let warm = 1.0 - b_scale;
                br = br * (1.0 - warm) + 1.0 * warm;
                bg = bg * (1.0 - warm) + 0.55 * warm;
                bb = bb * (1.0 - warm) + 0.25 * warm;
                sp.mat().m().color.set_rgb(br, bg, bb);
                sp.set_visible(true);
                sp.mat().m().opacity = 0.9 * b_scale;
                // a glint, not a flare (15 cm, it read as a glow: M8)
                sp.scale().set_scalar(0.04 * (0.6 + 0.4 * b_scale));
                self.sparkles[si].t = spark_life * b_scale;
            }
        }
    }
}
