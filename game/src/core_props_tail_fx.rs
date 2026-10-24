//! backrooms.js `placeProps`: the weather-driven extra props (lines
//! 59036–59764): wet puddles, derelict debris, still cobwebs, electric
//! sparking wires and haunted candles (blow out / relight). Part of
//! `place_props_tail` (core_props_tail.rs).
//!
//! `hauntedCandles[i]` (candleState) is `g.haunted_candles[i]`
//! (`records::CandleState`): `extinguished` → `!lit`; `fadeT`,
//! `hintCooldown`, `fizzleStreak`, `wickGlowT` → `f[..]`; `wasNear` →
//! `b["wasNear"]`. The candle's animator and its two interactables read and
//! write that record by index, the way the JS closures share the object.
use crate::core_props::*;
use crate::core_props_tail_seam::*;
use crate::prelude::*;
use crate::records::*;
use crate::stairwell::js_hypot2;
use crate::tables::*;

/// `BLOWOUT_MILESTONE_AT` (line 3397)
const BLOWOUT_MILESTONE_AT: f64 = 25.0;
/// `RELIGHT_MILESTONE_AT` (line 3398)
const RELIGHT_MILESTONE_AT: f64 = 15.0;

/// `wx(c.x) + rand(-1, 1)`, `y`, `wz(c.z) + rand(-1, 1)` (x drawn first)
pub(crate) fn jitter_pos(g: &Game, c: &Cand, y: f64) -> Vec3 {
    let x = g.wx(c.x) + rand2(-1.0, 1.0);
    let z = g.wz(c.z) + rand2(-1.0, 1.0);
    v3(x, y, z)
}
/// `v.copy(spot.pos).addScaledVector(spot.normal, d)`
pub(crate) fn off_wall(spot: &WallSpot, d: f64) -> Vec3 { let mut p = spot.pos; p.add_scaled_vector(&spot.normal, d); p }
/// `o.lookAt(camera.position.x, o.position.y, camera.position.z)`
pub(crate) fn look_at_level(o: &Obj, cam: &Obj) { let c = cam.pos(); let y = o.pos().y; o.look_at(c.x, y, c.z); }

// ---- candleState accessors (g.haunted_candles[ci])
fn c_out(g: &Game, ci: usize) -> bool { g.haunted_candles.get(ci).map(|c| !c.lit).unwrap_or(false) }
fn set_out(g: &mut Game, ci: usize, out: bool) { if let Some(c) = g.haunted_candles.get_mut(ci) { c.lit = !out; } }
fn cf(g: &Game, ci: usize, k: &str) -> f64 { g.haunted_candles.get(ci).and_then(|c| c.f.get(k).copied()).unwrap_or(0.0) }
fn set_cf(g: &mut Game, ci: usize, k: &str, v: f64) { if let Some(c) = g.haunted_candles.get_mut(ci) { c.f.insert(k.to_string(), v); } }
fn cb(g: &Game, ci: usize, k: &str) -> bool { g.haunted_candles.get(ci).and_then(|c| c.b.get(k).copied()).unwrap_or(false) }
fn set_cb(g: &mut Game, ci: usize, k: &str, v: bool) { if let Some(c) = g.haunted_candles.get_mut(ci) { c.b.insert(k.to_string(), v); } }

/// one smoke layer of the blow-out puff (line 59524)
#[derive(Clone, Copy, Debug)]
struct SmokeLayer { rise: f64, wobble: f64, phase: f64, xoff: f64, life: f64, scale_xk: f64, scale_yk: f64, droop: f64 }
const SMOKE_LAYERS_BASE: [SmokeLayer; 3] = [
    SmokeLayer { rise: 0.38, wobble: 3.2, phase: 0.0, xoff: 0.00, life: 4.5, scale_xk: 1.5, scale_yk: 1.8, droop: 0.03 },
    SmokeLayer { rise: 0.44, wobble: 2.6, phase: 1.7, xoff: 0.04, life: 5.2, scale_xk: 1.2, scale_yk: 1.6, droop: 0.02 },
    SmokeLayer { rise: 0.32, wobble: 3.9, phase: 3.1, xoff: -0.03, life: 3.9, scale_xk: 1.8, scale_yk: 2.1, droop: 0.025 },
];
const SMOKE_EXTRA_WET: SmokeLayer = SmokeLayer { rise: 0.28, wobble: 2.2, phase: 4.4, xoff: 0.02, life: 6.0, scale_xk: 2.2, scale_yk: 2.4, droop: 0.10 };

impl Game {
    // line 59036
    /// `// ---- WEATHER-DRIVEN EXTRA PROPS ----`: each weather adds a distinct
    /// visual signature; decoration only.
    pub(crate) fn tail_weather_extras(&mut self, cx: &mut PropsCtx) {
        let weather = self.weather_key();
        if weather == "wet" { self.tail_wet_puddles(cx); }
        if weather == "derelict" { self.tail_derelict_debris(cx); }
        if weather == "still" { self.tail_still_cobwebs(cx); }
        if weather == "electric" { self.tail_electric_wires(cx); }
        if weather == "haunted" { self.tail_haunted_candles(cx); }
    }

    // line 59041
    /// WET: dark puddle decals scattered on the floor
    fn tail_wet_puddles(&mut self, cx: &mut PropsCtx) {
        let puddle_tex = canvas_tex_opt(128, 128, false, |ctx, w, h| {
            ctx.set_fill_style("rgba(0,0,0,0)"); ctx.fill_rect(0, 0, w, h);
            // amorphous water blob using overlapping radial gradients
            for _ in 0..6 {
                let (cx, cy) = (30.0 + rand2(0.0, w - 60.0), 30.0 + rand2(0.0, h - 60.0));
                let grad = ctx.create_radial_gradient(cx, cy, 2, cx, cy, rand2(25.0, 55.0));
                grad.add_color_stop(0, "rgba(20,35,50,0.75)");
                grad.add_color_stop(1, "rgba(20,35,50,0)");
                ctx.set_fill_style(&grad); ctx.fill_rect(0, 0, w, h);
            }
            // faint highlight glint
            let shine = ctx.create_radial_gradient(w * 0.35, h * 0.35, 2, w * 0.35, h * 0.35, 22);
            shine.add_color_stop(0, "rgba(180,200,220,0.35)");
            shine.add_color_stop(1, "rgba(180,200,220,0)");
            ctx.set_fill_style(&shine); ctx.fill_rect(0, 0, w, h);
        });
        let puddle_mat = basic_mat!({ map: puddle_tex, transparent: true, depthWrite: false });
        // the loop bound is re-drawn every iteration, as in JS
        let mut i = 0;
        while i < irand(4, 7) {
            let Some(c) = s_take(self, cx, false) else { break };
            let size = rand2(0.6, 1.2);
            let puddle = mesh(plane_geo!(size, size), &puddle_mat);
            puddle.rotation().x = -PI / 2.0;
            puddle.rotation().z = rand2(0.0, PI * 2.0);
            puddle.set_pos_v(&jitter_pos(self, &c, 0.011));
            self.world.add(&puddle);
            i += 1;
        }
    }

    // line 59071
    /// DERELICT: fallen ceiling tile shards + more debris piles
    fn tail_derelict_debris(&mut self, cx: &mut PropsCtx) {
        let shard_mat = lambert!({ color: 0xa89f8a });
        let rebar_mat = lambert!({ color: 0x6a3f22 });
        let mut i = 0;
        while i < irand(3, 6) {
            let Some(c) = s_take(self, cx, false) else { break };
            let g = group();
            g.set_pos_v(&jitter_pos(self, &c, 0.0));
            g.rotation().y = rand1(PI * 2.0);
            // three angular ceiling-tile shards
            for s in 0..3 {
                let shard = mesh(pg("Props__derelict_shard", box_geo!(rand2(0.25, 0.5), 0.02, rand2(0.20, 0.4))), &shard_mat);
                shard.set_pos(rand2(-0.25, 0.25), 0.02 + s as f64 * 0.015, rand2(-0.25, 0.25));
                shard.set_rot(rand2(-0.6, 0.6), rand2(0.0, 7.0), rand2(-0.6, 0.6));
                g.add(&shard);
            }
            // bent rebar / metal strip poking up
            let rebar = mesh(pg("Props__derelict_rebar", cyl_geo!(0.008, 0.008, rand2(0.35, 0.55), 4)), &rebar_mat);
            rebar.set_pos(rand2(-0.15, 0.15), rand2(0.18, 0.30), rand2(-0.15, 0.15));
            rebar.set_rot(rand2(-0.6, 0.6), rand2(0.0, 7.0), rand2(-0.6, 0.6));
            g.add(&rebar);
            // dust cloud disc
            let dust = mesh(circle_geo!(0.4, 12), basic_mat!({ color: 0x8a7f6a, transparent: true, opacity: 0.25, depthWrite: false }));
            dust.rotation().x = -PI / 2.0;
            dust.position().y = 0.005;
            g.add(&dust);
            self.world.add(&g);
            i += 1;
        }
    }

    // line 59104
    /// STILL: cobweb decals hanging in the corners of walls
    fn tail_still_cobwebs(&mut self, cx: &mut PropsCtx) {
        let web_tex = canvas_tex_opt(128, 128, false, |ctx, w, h| {
            ctx.set_fill_style("rgba(0,0,0,0)"); ctx.fill_rect(0, 0, w, h);
            ctx.set_stroke_style("rgba(232,224,208,0.55)");
            ctx.set_line_width(0.7);
            // radial threads from a corner
            let (cx, cy) = (6.0, 6.0);
            for r in 0..12 {
                let a = (r as f64 / 12.0) * (PI * 0.5);
                ctx.begin_path();
                ctx.move_to(cx, cy);
                ctx.line_to(cx + a.cos() * 130.0, cy + a.sin() * 130.0);
                ctx.stroke();
            }
            // concentric arcs
            for ring in 1..6 {
                ctx.begin_path();
                ctx.arc(cx, cy, ring as f64 * 18.0, 0, PI * 0.5);
                ctx.stroke();
            }
        });
        let web_mat = basic_mat!({ map: web_tex, transparent: true, depthWrite: false });
        let mut i = 0;
        while i < irand(4, 8) {
            let Some(c) = s_take(self, cx, true) else { break };
            if let Some(spot) = s_wall_adj_spot(self, c.x, c.z) {
                let web = mesh(plane_geo!(0.7, 0.7), &web_mat);
                web.set_pos_v(&off_wall(&spot, 0.05));
                web.position().y = WALL_H - 0.35;   // near ceiling corner
                web.rotation().y = spot.yaw;
                // random 90° rotation so the corner attaches to different quadrants
                web.rotation().z = irand(0, 3) as f64 * (PI / 2.0);
                self.world.add(&web);
            }
            i += 1;
        }
    }

    // line 59141
    /// ELECTRIC: sparking exposed-wire bundles on the floor; each has an
    /// additive spark that flashes at random intervals and, rarely, flares.
    fn tail_electric_wires(&mut self, cx: &mut PropsCtx) {
        let wire_mat = lambert!({ color: 0x2a1a08 });
        let insul_mat = lambert!({ color: 0xc0a840 });
        let spark_tex = canvas_tex_opt(64, 64, false, |ctx, w, h| {
            ctx.set_fill_style("rgba(0,0,0,0)"); ctx.fill_rect(0, 0, w, h);
            let grad = ctx.create_radial_gradient(w / 2.0, h / 2.0, 2, w / 2.0, h / 2.0, w / 2.0 - 4.0);
            grad.add_color_stop(0, "rgba(255,255,220,0.95)");
            grad.add_color_stop(0.35, "rgba(255,200,90,0.6)");
            grad.add_color_stop(1, "rgba(255,180,60,0)");
            ctx.set_fill_style(&grad); ctx.fill_rect(0, 0, w, h);
            // star cross for the spark shape
            ctx.set_stroke_style("rgba(255,255,240,0.95)");
            ctx.set_line_width(4); ctx.set_line_cap("round");
            ctx.begin_path(); ctx.move_to(w / 2.0, 8); ctx.line_to(w / 2.0, h - 8.0); ctx.stroke();
            ctx.begin_path(); ctx.move_to(8, h / 2.0); ctx.line_to(w - 8.0, h / 2.0); ctx.stroke();
        });
        let mut i = 0;
        while i < irand(2, 4) {
            let Some(c) = s_take(self, cx, false) else { break };
            let g = group();
            g.set_pos_v(&jitter_pos(self, &c, 0.0));
            g.rotation().y = rand1(PI * 2.0);
            // 3 curled wires (small torus arcs) with dark cores and coloured jackets
            for w in 0..3 {
                let wf = w as f64;
                let outer = mesh(pg("Props__electric_cable", torus_geo!(0.10 + wf * 0.02, 0.014, 5, 12, PI * 1.3)), &insul_mat);
                outer.set_pos(rand2(-0.05, 0.05), 0.016 + wf * 0.012, rand2(-0.05, 0.05));
                // lying on the floor: the ring's plane laid flat, spun, a little slack
                let (rx, ry, rz) = (-PI / 2.0 + rand2(-0.25, 0.25), rand2(0.0, 7.0), rand2(0.0, 7.0));
                { let mut r = outer.rotation(); r.set(rx, ry, rz); r.order = EulerOrder::YXZ; }
                g.add(&outer);
            }
            // bare copper tip (small dark cylinder poking up)
            let tip = mesh(pg("Props__electric_wireTip", cyl_geo!(0.012, 0.012, 0.10, 5)), &wire_mat);
            tip.set_pos(0, 0.08, 0);
            tip.rotation().z = rand2(-0.6, 0.6);
            g.add(&tip);
            // spark billboard (usually invisible, flashes on)
            let spark = mesh(plane_geo!(0.20, 0.20), basic_mat!({
                map: spark_tex.clone(), transparent: true, opacity: 0,
                blending: AdditiveBlending, depthWrite: false,
            }));
            spark.set_pos(0, 0.14, 0);
            g.add(&spark);
            // small point light co-located with the tip that pulses with the spark
            let spark_light = point_light!(0xfff2a0, 0, 3, 2.0);
            spark_light.set_pos(0, 0.14, 0);
            g.add(&spark_light);
            self.world.add(&g);
            // Sparks fire on a randomised schedule; ~1/40 flashes escalate into a
            // full flare: white screen flash, crackle + thump, a brief sanity dip.
            let mut next_flash = rand2(0.5, 2.5);
            let mut flash_t = 0.0;
            let mut boom = false;
            self.animate(move |gm, dt, _t| {
                next_flash -= dt;
                if next_flash <= 0.0 {
                    next_flash = rand2(0.8, 3.5);
                    flash_t = rand2(0.06, 0.14);
                    let gp = g.pos();
                    let dp = js_hypot2(gp.x - gm.player.pos.x, gp.z - gm.player.pos.z);
                    boom = rnd() < 0.025 && dp < 22.0;   // rare + audible
                    if boom {
                        // Big flare: extend visible flash, cue white screen flash + audio
                        flash_t = 0.28;
                        spark_flare(gm, &g, dp);
                    } else if dp < 14.0 && gm.ac.is_some() {
                        // brief crackle only if the player is close-ish
                        gm.noise_burst(0.05, 4200.0, 3.0, clamp(0.09 - dp * 0.005, 0.02, 0.09), "highpass", None, 0.0, 0.0);
                    }
                }
                if flash_t > 0.0 {
                    flash_t -= dt;
                    spark.mat().set_opacity(if boom { 1.0 } else { 0.9 });
                    spark_light.set_intensity(if boom { 5.5 } else { 1.6 });
                    // boom sparks scale up dramatically
                    let s = if boom { 1.0 + (0.28 - flash_t) * 4.0 } else { 1.0 };
                    spark.scale().set(s, s, s);
                    spark.look_at_v(&gm.camera.pos());
                } else {
                    spark.mat().set_opacity(0.0);
                    spark_light.set_intensity(0.0);
                    spark.scale().set(1, 1, 1);
                    boom = false;
                }
            });
            i += 1;
        }
    }

    // line 59370
    /// HAUNTED: single lit floor candles at random tiles, with a blow-out and
    /// a relight (3% battery) interactable each.
    fn tail_haunted_candles(&mut self, cx: &mut PropsCtx) {
        let wax_mat = lambert!({ color: 0xd8ccb0 });
        let flame_mat = basic_mat!({
            color: 0xffb060, transparent: true, opacity: 0.9,
            blending: AdditiveBlending, depthWrite: false,
        });
        // R106 inner hot core — smaller, brighter, more yellow-white
        let flame_core_mat = basic_mat!({
            color: 0xffe8b0, transparent: true, opacity: 0.85,
            blending: AdditiveBlending, depthWrite: false,
        });
        let mut i = 0;
        while i < irand(2, 4) {
            let Some(c) = s_take(self, cx, false) else { break };
            let g = group();
            g.set_pos_v(&jitter_pos(self, &c, 0.0));
            let wax = mesh(pg("Props__candle_pillar", cyl_geo!(0.035, 0.04, 0.16, 8)), &wax_mat);
            wax.position().y = 0.08; g.add(&wax);
            // wick (own material so the post-relight glow doesn't bleed to other candles)
            let wick = mesh(pg("Props__candle_wick", box_geo!(0.006, 0.022, 0.006)), lambert!({ color: 0x1a1408, emissive: 0x000000 }));
            wick.position().y = 0.17; g.add(&wick);
            // billboarded outer flame + R106 inner hot core
            let flame = mesh(plane_geo!(0.09, 0.15), &flame_mat);
            flame.position().y = 0.24; g.add(&flame);
            let flame_core = mesh(plane_geo!(0.05, 0.09), &flame_core_mat);
            flame_core.position().y = 0.235; g.add(&flame_core);
            // warm point light
            let glow = point_light!(0xffa050, 1.4, 2.6, 1.8);
            glow.position().y = 0.28; g.add(&glow);
            // a small ring of melted wax around the base
            let melt_tex = canvas_tex_opt(64, 64, false, |ctx, w, h| {
                ctx.set_fill_style("rgba(0,0,0,0)"); ctx.fill_rect(0, 0, w, h);
                let grad = ctx.create_radial_gradient(w / 2.0, h / 2.0, 4, w / 2.0, h / 2.0, w / 2.0);
                grad.add_color_stop(0, "rgba(180,170,140,0.7)");
                grad.add_color_stop(1, "rgba(180,170,140,0)");
                ctx.set_fill_style(&grad); ctx.fill_rect(0, 0, w, h);
            });
            let melt = mesh(circle_geo!(0.12, 12), basic_mat!({ map: melt_tex, transparent: true, depthWrite: false }));
            melt.rotation().x = -PI / 2.0;
            melt.position().y = 0.004;
            g.add(&melt);
            self.world.add(&g);
            let off = i as f64 * 0.9;
            // per-candle state shared by the animator and both interactables
            let ci = self.haunted_candles.len();
            let mut st = CandleState { lit: true, ..Default::default() };
            for k in ["fadeT", "hintCooldown", "fizzleStreak", "wickGlowT"] { st.f.insert(k.to_string(), 0.0); }
            st.b.insert("wasNear".to_string(), false);
            self.haunted_candles.push(st);
            let gp = g.pos();
            {
                let (glow, flame, flame_core, wick) = (glow.clone(), flame.clone(), flame_core.clone(), wick.clone());
                self.animate(move |gm, dt, t| candle_frame(gm, ci, gp, off, &glow, &flame, &flame_core, &wick, dt, t));
            }
            // Interactable: press E near the candle to blow it out
            let mut use_pos = gp;
            use_pos.y = EYE;
            self.interact(use_pos, 0.9, "[E] BLOW OUT", move |gm| !c_out(gm, ci), move |gm| candle_blow_out(gm, ci, gp));
            // Companion: relight an extinguished candle for 3% battery
            let (glow, flame, flame_core) = (glow.clone(), flame.clone(), flame_core.clone());
            self.interact(use_pos, 0.9, "[E] RELIGHT (3% battery)",
                move |gm| c_out(gm, ci) && gm.player.battery >= 3.0,
                move |gm| candle_relight(gm, ci, gp, &glow, &flame, &flame_core));
            i += 1;
        }
    }
}

// line 59202
/// The rare spark flare (`state.boom`): audio, the white screen flash, the
/// sanity dip and the scorch decal (8 s fade, or 10% permanent with smoke).
fn spark_flare(g: &mut Game, sg: &Obj, dp: f64) {
    if g.ac.is_some() {
        g.noise_burst(0.15, 2800.0, 2.0, 0.22, "bandpass", None, 0.0, 0.0);
        g.noise_burst(0.10, 5000.0, 3.0, 0.14, "highpass", None, 0.02, 0.0);
        g.blip(90.0, 0.4, "sine", 0.20, Some(40.0), 0.02, 0.0);
    }
    // Full-screen white flash via the existing #flash element
    g.dom.flash.set_style("background", "radial-gradient(ellipse at center, rgba(255,255,240,0.7) 0%, rgba(255,240,200,0.35) 60%, transparent 100%)");
    g.dom.flash.set_opacity(1.0);
    g.set_timeout(120.0, |g| {
        g.dom.flash.set_opacity(0.0);
        // restore the standard hurt-flash gradient after the pulse
        g.set_timeout(350.0, |g| {
            g.dom.flash.set_style("background", "radial-gradient(ellipse at center, rgba(120,0,0,0.55) 0%, rgba(60,0,0,0.9) 100%)");
        });
    });
    // brief sanity dip if close enough that it startles the player
    if dp < 10.0 { g.player.sanity = clamp(g.player.sanity - 4.0, 0.0, SAN_MAX); }
    g.show_msg("SPARKS FLARED!", 1.2);
    // Post-boom scorch decal on the floor beneath the wire bundle
    let scorch_tex = canvas_tex_opt(64, 64, false, |ctx, w, h| {
        ctx.set_fill_style("rgba(0,0,0,0)"); ctx.fill_rect(0, 0, w, h);
        // charred core + faint burn ring
        for _ in 0..8 {
            let (cx, cy) = (w / 2.0 + rand2(-8.0, 8.0), h / 2.0 + rand2(-8.0, 8.0));
            let grad = ctx.create_radial_gradient(cx, cy, 1, cx, cy, rand2(12.0, 24.0));
            grad.add_color_stop(0, &format!("rgba(10,8,6,{})", rand2(0.65, 0.85)));
            grad.add_color_stop(1, "rgba(10,8,6,0)");
            ctx.set_fill_style(&grad); ctx.fill_rect(0, 0, w, h);
        }
        // a few outward sooty streaks
        ctx.set_stroke_style("rgba(20,15,10,0.55)"); ctx.set_line_width(1.5);
        for s in 0..6 {
            let a = (s as f64 / 6.0) * PI * 2.0 + rand2(-0.2, 0.2);
            let r = rand2(18.0, 30.0);
            ctx.begin_path();
            ctx.move_to(w / 2.0, h / 2.0);
            ctx.line_to(w / 2.0 + a.cos() * r, h / 2.0 + a.sin() * r);
            ctx.stroke();
        }
    });
    let gp = sg.pos();
    let scorch = mesh(plane_geo!(0.60, 0.60), basic_mat!({ map: scorch_tex.clone(), transparent: true, depthWrite: false, opacity: 0.9 }));
    scorch.rotation().x = -PI / 2.0;
    scorch.rotation().z = rand2(0.0, PI * 2.0);
    scorch.set_pos(gp.x, 0.008, gp.z);
    g.world.add(&scorch);
    // 10% chance the scorch is permanent (rest of the level), otherwise it fades over 8 s
    if rnd() < 0.10 {
        scorch.mat().set_opacity(0.65);
        // brief smoke wisp so the "just happened" beat still lands
        let ps_smoke = mesh(plane_geo!(0.16, 0.32), basic_mat!({
            color: 0xa8a8a8, transparent: true, opacity: 0.42,
            blending: AdditiveBlending, depthWrite: false,
        }));
        ps_smoke.set_pos(gp.x, 0.30, gp.z);
        g.world.add(&ps_smoke);
        let mut ps_t = 0.0;
        let sg1 = sg.clone();
        g.animate(move |g, sdt, _| {
            ps_t += sdt;
            let p = ps_t / 2.4;
            if p >= 1.0 { g.world.remove(&ps_smoke); return; }
            ps_smoke.position().y = 0.30 + p * 0.55;
            ps_smoke.position().x = sg1.pos().x + (ps_t * 3.8).sin() * 0.04;
            ps_smoke.scale().set(1.0 + p * 1.4, 1.0 + p * 1.6, 1.0);
            ps_smoke.mat().set_opacity(0.42 * (1.0 - p) * (1.0 - p * 0.4));
            look_at_level(&ps_smoke, &g.camera);
        });
        // secondary near-ground puff: quick fade, half the wisp's life
        let ps_puff = mesh(plane_geo!(0.22, 0.14), basic_mat!({
            color: 0x808080, transparent: true, opacity: 0.55,
            blending: AdditiveBlending, depthWrite: false,
        }));
        // ±5 cm jitter so overlapping scorches don't stack identical puffs
        let puff_x_off = rand2(-0.05, 0.05);
        let puff_z_off = rand2(-0.05, 0.05);
        ps_puff.set_pos(gp.x + puff_x_off, 0.10, gp.z + puff_z_off);
        g.world.add(&ps_puff);
        let mut puff_t = 0.0;
        g.animate(move |g, sdt, _| {
            puff_t += sdt;
            let q = puff_t / 1.0;
            if q >= 1.0 { g.world.remove(&ps_puff); return; }
            ps_puff.position().y = 0.10 + q * 0.12;
            ps_puff.scale().set(1.0 + q * 1.8, 1.0 + q * 1.5, 1.0);
            ps_puff.mat().set_opacity(0.55 * (1.0 - q * q));   // fast quadratic fade
            look_at_level(&ps_puff, &g.camera);
        });
        // scorch cluster: 2-3 smaller charred fragments around the main scorch
        let fragment_count = irand(2, 3);
        for _ in 0..fragment_count {
            let frag_mat = basic_mat!({ map: scorch_tex.clone(), transparent: true, depthWrite: false, opacity: 0.35 });
            let frag = mesh(plane_geo!(0.22, 0.22), frag_mat);
            frag.rotation().x = -PI / 2.0;
            let angle = rand2(0.0, PI * 2.0);
            // radial "blast" alignment + small orientation jitter
            frag.rotation().z = angle + rand2(-0.3, 0.3);
            let dist = rand2(0.35, 0.55);
            // distance taper: fragments further out are smaller
            let dist_norm = (dist - 0.35) / 0.20;
            let size_scale = 1.0 - dist_norm * 0.35;
            frag.scale().set(size_scale, size_scale, 1.0);
            frag.set_pos(gp.x + angle.cos() * dist, 0.007, gp.z + angle.sin() * dist);
            g.world.add(&frag);
        }
    } else {
        let mut scorch_t = 0.0;
        g.animate(move |g, sdt, _| {
            scorch_t += sdt;
            let p = scorch_t / 8.0;
            if p >= 1.0 { g.world.remove(&scorch); return; }
            scorch.mat().set_opacity(0.9 * (1.0 - p));
        });
    }
}

// line 59422
/// A haunted candle's animator: the 0.6 s fade + low-battery hint while out;
/// the LFO-synced flicker, battery boost and wick glow while lit.
fn candle_frame(g: &mut Game, ci: usize, gp: Vec3, off: f64, glow: &Obj, flame: &Obj, flame_core: &Obj, wick: &Obj, dt: f64, t: f64) {
    if c_out(g, ci) {
        // fade over 0.6 s, then a cold wick
        if cf(g, ci, "fadeT") < 0.6 {
            let fade_t = cf(g, ci, "fadeT") + dt;
            set_cf(g, ci, "fadeT", fade_t);
            let k = clamp(1.0 - fade_t / 0.6, 0.0, 1.0);
            glow.set_intensity(1.4 * k * 0.5);
            flame.mat().set_opacity(0.9 * k);
            flame_core.mat().set_opacity(0.85 * k);
            flame.scale().y = 1.0 + k * 0.5;
            flame_core.scale().y = 1.0 + k * 0.7;   // core stretches slightly more
            flame.look_at_v(&g.camera.pos());
            flame_core.look_at_v(&g.camera.pos());
        } else {
            glow.set_intensity(0.0);
            flame.set_visible(false);
            flame_core.set_visible(false);
        }
        // low-battery hint: once per approach, guarded by a cooldown
        if cf(g, ci, "hintCooldown") > 0.0 { let v = cf(g, ci, "hintCooldown") - dt; set_cf(g, ci, "hintCooldown", v); }
        let (dxc, dzc) = (gp.x - g.player.pos.x, gp.z - g.player.pos.z);
        let near_now = (dxc * dxc + dzc * dzc) < 0.81;   // 0.9 m
        if near_now && !cb(g, ci, "wasNear") && g.player.battery < 3.0 && cf(g, ci, "hintCooldown") <= 0.0 {
            g.show_msg("NEED 3% BATTERY", 1.4);
            set_cf(g, ci, "hintCooldown", 6.0);
            if g.ac.is_some() {
                let dips = g.weather_pick(battery_hint_dips, battery_hint_dips("normal").unwrap());
                let (hi, lo) = (dips[0], dips[1]);
                g.blip(hi, 0.30, "sine", 0.06, None, 0.0, 0.0);
                g.blip(lo, 0.35, "sine", 0.05, None, 0.08, 0.0);
            }
        }
        set_cb(g, ci, "wasNear", near_now);
        return;
    }
    set_cb(g, ci, "wasNear", false);   // relit — reset the approach state
    // R108/R110: the safe-room LFO breath, weather-scaled amplitude
    let lfo_hz = g.audio.safe_room_lfo.as_ref().map(|o| o.frequency().value()).filter(|v| *v != 0.0 && !v.is_nan()).unwrap_or(0.35);
    let flame_amp = flame_breath_amp_by_weather(g.weather_key()).or(flame_breath_amp_by_weather("normal")).unwrap() * g.descent_boost_mul(0.4);
    let room_breath = 1.0 + (t * lfo_hz * PI * 2.0 * g.descent_freq_boost() + off * 0.6).sin() * flame_amp;
    let fk = (0.85 + (t * 12.0 + off).sin() * 0.12 + (t * 21.3 + off).sin() * 0.05) * room_breath;
    // R116: candles brighten a little as the flashlight dies
    let candle_boost = if g.player.flashlight_on { 1.0 + 0.15 * (1.0 - g.beam_battery_scale(0.0)) } else { 1.0 };
    // R176 descent dim
    glow.set_intensity(1.4 * fk * candle_boost * (0.65 + 0.35 * g.level_depth_mul()));
    flame.scale().y = 1.0 + (t * 10.0 + off).sin() * 0.18;
    flame.look_at_v(&g.camera.pos());
    // R106 inner core: faster, different-phase wobble
    flame_core.scale().y = 1.0 + (t * 13.0 + off * 1.7).sin() * 0.24;
    flame_core.scale().x = 1.0 + (t * 17.0 + off * 2.1).sin() * 0.06;
    flame_core.look_at_v(&g.camera.pos());
    if flame_core.visible() != flame.visible() { flame_core.set_visible(flame.visible()); }
    // R107 wick glow decay right after a relight
    let wg = cf(g, ci, "wickGlowT");
    if wg > 0.0 {
        let wg = 0f64.max(wg - dt);
        set_cf(g, ci, "wickGlowT", wg);
        let gk = wg / 0.35;
        wick.mat().m().emissive.set_rgb(0.95 * gk, 0.45 * gk, 0.10 * gk);
    } else if wick.mat().emissive_v().r != 0.0 {
        wick.mat().m().emissive.set_rgb(0, 0, 0);
    }
}

// line 59504
/// `[E] BLOW OUT`
fn candle_blow_out(g: &mut Game, ci: usize, gp: Vec3) {
    set_out(g, ci, true);
    g.candle_blowout_tally += 1.0;
    // R68 milestone: a quiet toast on the 25th blowout of the session
    if !g.blowout_milestone_shown && g.candle_blowout_tally >= BLOWOUT_MILESTONE_AT {
        g.blowout_milestone_shown = true;
        g.set_timeout(1200.0, |g| g.show_msg("A HUNDRED SMALL DARKNESSES", 2.2));
    }
    // soft breath / puff sfx
    if g.ac.is_some() { g.noise_burst(0.22, 3600.0, 1.5, 0.14, "highpass", None, 0.0, 0.0); }
    // layered smoke planes; 4 layers when wet, 2 when still, else 3
    let smoke_layers: Vec<SmokeLayer> = match g.weather_key() {
        "wet" => { let mut v = SMOKE_LAYERS_BASE.to_vec(); v.push(SMOKE_EXTRA_WET); v }
        "still" => SMOKE_LAYERS_BASE[..2].to_vec(),
        _ => SMOKE_LAYERS_BASE.to_vec(),
    };
    let smoke_start = 0.30;
    // smoke picks up a subtle tint from the weather
    let smoke_color = g.weather_pick(|k| Some(match k {
        "wet" => 0xb0c8dcu32, "electric" => 0xc0b8e0, "haunted" => 0xd8b4b0, "derelict" => 0xd8bfa0,
        "still" => 0xc8c4b8, "normal" => 0xd8d8d8, _ => return None,
    }), 0xd8d8d8);
    for layer in smoke_layers {
        let smoke = mesh(plane_geo!(0.14, 0.28), basic_mat!({
            color: smoke_color, transparent: true, opacity: 0.35,
            blending: AdditiveBlending, depthWrite: false,
        }));
        smoke.set_pos(gp.x + layer.xoff, smoke_start, gp.z);
        g.world.add(&smoke);
        let mut smoke_t = 0.0;
        g.animate(move |g, dt, _| {
            smoke_t += dt;
            let p = smoke_t / layer.life;
            if p >= 1.0 { g.world.remove(&smoke); return; }
            // rise, then droop back by up to `droop` metres past p = 0.6
            let droop_p = if layer.droop != 0.0 { 0f64.max((p - 0.6) / 0.4) } else { 0.0 };
            smoke.position().y = smoke_start + p * layer.rise - droop_p * layer.droop;
            // wobble: amplitude grows while drooping; weather scales amplitude and frequency
            let wk = g.weather_key();
            let wobble_weather_mul = if wk == "wet" { 1.35 } else if wk == "still" { 0.75 } else { 1.0 };
            let wobble_freq_mul = if wk == "wet" { 0.75 } else if wk == "still" { 1.30 } else { 1.0 };
            let wobble_amp = 0.03 * wobble_weather_mul * (1.0 + droop_p * 1.5);
            smoke.position().x = gp.x + layer.xoff + (smoke_t * layer.wobble * wobble_freq_mul + layer.phase).sin() * wobble_amp;
            smoke.scale().set(1.0 + p * layer.scale_xk, 1.0 + p * layer.scale_yk, 1.0);
            smoke.mat().set_opacity(0.35 * (1.0 - p) * (1.0 - p * 0.5));
            look_at_level(&smoke, &g.camera);
        });
    }
    // R66 tail beat ~2.8 s in (TAIL_BEAT_SPECS by weather)
    if g.ac.is_some() {
        let specs = tail_beat_specs(g.weather_key()).or(tail_beat_specs("normal")).unwrap();
        g.set_timeout(2800.0, move |g| {
            for &(f, jit, dur, ty, amp, delay_ms) in specs {
                let extra_sec = delay_ms / 1000.0;
                g.blip(f + rand2(-jit, jit), dur, ty, amp, None, extra_sec, 0.0);
            }
        });
    }
}

// line 59606
/// `[E] RELIGHT (3% battery)`: 15% (by weather) fizzles for 1%, a third try
/// after two fizzles always works and costs 2%.
fn candle_relight(g: &mut Game, ci: usize, gp: Vec3, glow: &Obj, flame: &Obj, flame_core: &Obj) {
    if !c_out(g, ci) || g.player.battery < 3.0 { return; }
    let force_success = cf(g, ci, "fizzleStreak") >= 2.0;
    let fizzle_prob = fizzle_prob_by_weather(g.weather_key()).or(fizzle_prob_by_weather("normal")).unwrap();
    if !force_success && rnd() < fizzle_prob {
        let streak = cf(g, ci, "fizzleStreak") + 1.0;
        set_cf(g, ci, "fizzleStreak", streak);
        g.candle_fizzle_tally += 1.0;
        g.player.battery = clamp(g.player.battery - 1.0, 0.0, BAT_MAX);
        if g.ac.is_some() {
            // the second fizzle sounds lower and longer: an escalating refusal
            if streak >= 2.0 {
                g.noise_burst(0.05, 3200.0, 3.0, 0.11, "highpass", None, 0.0, 0.0);
                g.noise_burst(0.05, 2800.0, 3.0, 0.09, "highpass", None, 0.10, 0.0);
                g.noise_burst(0.28, 600.0, 1.0, 0.08, "lowpass", None, 0.16, 0.0);
                g.blip(80.0, 0.35, "sine", 0.06, Some(60.0), 0.05, 0.0);
            } else {
                g.noise_burst(0.04, 4200.0, 3.0, 0.09, "highpass", None, 0.0, 0.0);
                g.noise_burst(0.04, 3600.0, 3.0, 0.07, "highpass", None, 0.08, 0.0);
                g.noise_burst(0.15, 900.0, 1.2, 0.05, "lowpass", None, 0.15, 0.0);
            }
        }
        // second consecutive fizzle hints that the mercy is coming
        if streak >= 2.0 { g.show_msg("THE ROOM RESISTS...", 1.6); } else { g.show_msg("...THE WICK REFUSED.", 1.4); }
        // a small dark wisp so the failed spark leaves a trace
        {
            let fizz_color = g.weather_pick(|k| Some(match k {
                "wet" => 0x14202au32, "electric" => 0x181428, "haunted" => 0x281414, "derelict" => 0x241a10,
                "still" => 0x1a1a1a, "normal" => 0x1a1a1a, _ => return None,
            }), 0x1a1a1a);
            let f_smoke = mesh(plane_geo!(0.08, 0.18), basic_mat!({
                color: fizz_color, transparent: true, opacity: 0.55,
                blending: NormalBlending, depthWrite: false,
            }));
            f_smoke.set_pos(gp.x, 0.28, gp.z);
            g.world.add(&f_smoke);
            let mut f_t = 0.0;
            g.animate(move |g, dt, _| {
                f_t += dt;
                let p = f_t / 1.8;
                if p >= 1.0 { g.world.remove(&f_smoke); return; }
                f_smoke.position().y = 0.28 + p * 0.18;
                f_smoke.position().x = gp.x + (f_t * 4.5).sin() * 0.015;
                f_smoke.scale().set(1.0 + p * 0.8, 1.0 + p * 1.0, 1.0);
                f_smoke.mat().set_opacity(0.55 * (1.0 - p));
                look_at_level(&f_smoke, &g.camera);
            });
            // R52/R72: sometimes a fainter, warmer echo wisp 200 ms later
            let echo_prob = fizzle_echo_prob_by_weather(g.weather_key()).or(fizzle_echo_prob_by_weather("normal")).unwrap();
            if rnd() < echo_prob {
                g.set_timeout(200.0, move |g| {
                    if g.state != "playing" { return; }
                    let echo_color = g.weather_pick(|k| Some(match k {
                        "wet" => 0x1a2418u32, "electric" => 0x241a1a, "haunted" => 0x2a1c14, "derelict" => 0x2a1e10,
                        "still" => 0x2a1e14, "normal" => 0x2a1e14, _ => return None,
                    }), 0x2a1e14);
                    let e_smoke = mesh(plane_geo!(0.06, 0.14), basic_mat!({
                        color: echo_color, transparent: true, opacity: 0.35,
                        blending: NormalBlending, depthWrite: false,
                    }));
                    e_smoke.set_pos(gp.x + rand2(-0.03, 0.03), 0.28, gp.z);
                    g.world.add(&e_smoke);
                    let mut e_t = 0.0;
                    g.animate(move |g, dt, _| {
                        e_t += dt;
                        let q = e_t / 1.4;
                        if q >= 1.0 { g.world.remove(&e_smoke); return; }
                        e_smoke.position().y = 0.28 + q * 0.14;
                        e_smoke.scale().set(1.0 + q * 0.6, 1.0 + q * 0.9, 1.0);
                        e_smoke.mat().set_opacity(0.35 * (1.0 - q));
                        look_at_level(&e_smoke, &g.camera);
                    });
                });
            }
        }
        return;
    }
    let was_forced_mercy = force_success;
    set_out(g, ci, false);
    g.candle_relight_tally += 1.0;
    // R69 milestone on the 15th relight of the session
    if !g.relight_milestone_shown && g.candle_relight_tally >= RELIGHT_MILESTONE_AT {
        g.relight_milestone_shown = true;
        g.set_timeout(1400.0, |g| g.show_msg("THE ROOM REMEMBERS YOUR HANDS", 2.2));
    }
    set_cf(g, ci, "fadeT", 0.0);
    set_cf(g, ci, "fizzleStreak", 0.0);   // reset on any success
    flame.set_visible(true);
    flame.mat().set_opacity(0.9);
    flame_core.set_visible(true);
    flame_core.mat().set_opacity(0.85);
    glow.set_intensity(1.4);
    set_cf(g, ci, "wickGlowT", 0.35);   // R107: brief post-relight wick glow
    // a mercy relight refunds 1%
    let cost = if was_forced_mercy { 2.0 } else { 3.0 };
    g.player.battery = clamp(g.player.battery - cost, 0.0, BAT_MAX);
    // strike + 660→880 Hz chirp (50 ms late, with the wick glow peak)
    if g.ac.is_some() {
        g.noise_burst(0.06, 5200.0, 3.0, 0.10, "highpass", None, 0.0, 0.0);
        g.blip(660.0, 0.10, "sine", 0.06, Some(880.0), 0.05, 0.0);
        if was_forced_mercy { g.blip(1320.0, 0.14, "sine", 0.05, Some(1760.0), 0.11, 0.0); }
        // R68 per-weather relight accent
        let spark = relight_specs(g.weather_key()).or(relight_specs("normal")).unwrap();
        for &(f, jit, dur, ty, amp, delay_ms) in spark {
            g.blip(f + rand2(-jit, jit), dur, ty, amp, None, delay_ms / 1000.0, 0.0);
        }
    }
    if was_forced_mercy { g.show_msg("THE WICK CAUGHT — AT LAST.", 1.4); }
}
