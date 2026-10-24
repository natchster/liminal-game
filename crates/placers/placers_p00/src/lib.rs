//! Prop placers: the R1 gameplay trigger (a wall switch that lights a distant
//! ceiling glow panel), floor stains and smoking ashtrays.
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use game::base::state::GameplayR1State;
use game::prelude::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_gameplay_r1_trigger", place_gameplay_r1_trigger));
    r.push(("place_floor_stains", place_floor_stains));
    r.push(("place_smoking_ashtrays", place_smoking_ashtrays));
}

fn _gameplay_r1_prng(g: &Game) -> Mulberry32 {
    Mulberry32::new((g.current_seed ^ 0xA1D3F001) as u32)
}

fn place_gameplay_r1_trigger(g: &mut Game, dist: &[i32]) {
    g.gameplay_r1_state = None;
    let mut gr = _gameplay_r1_prng(g);
    // Find wall-adjacent open tiles in medium-distance zone.
    let mut max_d = 0;
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            let d = dist[g.t_idx(x, z)];
            if d > max_d {
                max_d = d;
            }
        }
    }
    let mid_min = 6.max((max_d as f64 * 0.25).floor() as i32);
    let mid_max = (mid_min + 4).max((max_d as f64 * 0.65).floor() as i32);
    let mut switch_cands: Vec<(i32, i32, i32)> = vec![];
    let mut far_cands: Vec<(i32, i32, i32)> = vec![];
    let is_wall_adj = |g: &Game, x: i32, z: i32| [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dz)| g.is_wall_tile(x + dx, z + dz));
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            let d = dist[g.t_idx(x, z)];
            if d < 0 {
                continue;
            }
            if d >= mid_min && d <= mid_max && is_wall_adj(g, x, z) {
                switch_cands.push((x, z, d));
            }
            if d > mid_max + 2 {
                far_cands.push((x, z, d));
            }
        }
    }
    if switch_cands.is_empty() || far_cands.is_empty() {
        return;
    }
    let swi = switch_cands[(gr.next() * switch_cands.len() as f64).floor() as usize];
    // farthest (Manhattan) from the switch first; the sort is stable, so ties keep scan order
    far_cands.sort_by(|a, b| {
        let da = (a.0 - swi.0).abs() + (a.1 - swi.1).abs();
        let db = (b.0 - swi.0).abs() + (b.1 - swi.1).abs();
        db.cmp(&da)
    });
    let top_n = 1.max((far_cands.len() as f64 / 3.0).floor() as usize);
    let target = far_cands[(gr.next() * top_n as f64).floor() as usize];
    let (mut ndx, mut ndz) = (0, 0);
    for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if g.is_wall_tile(swi.0 + dx, swi.1 + dz) {
            ndx = dx;
            ndz = dz;
            break;
        }
    }
    let plate_mat = lambert!({ color: 0x606060 });
    let btn_off_mat = lambert!({ color: 0xa02020 });
    let btn_on_mat = lambert!({ color: 0x40e040, emissive: 0x104010 });
    let sw_group = group();
    let mount_y = 1.35;
    let plate = mesh(pg("GameplayR1Trigger__plate", box_geo!(0.12, 0.16, 0.010)), &plate_mat);
    plate.position().y = mount_y;
    sw_group.add(&plate);
    let btn = mesh(pg("GameplayR1Trigger__button", cyl_geo!(0.030, 0.030, 0.020, 12)), &btn_off_mat);
    btn.rotation().x = PI / 2.0;
    btn.position().set(0, mount_y, 0.014);
    sw_group.add(&btn);
    let label = mesh(pg("GameplayR1Trigger__label", box_geo!(0.080, 0.014, 0.003)), lambert!({ color: 0xf0f0d0 }));
    label.position().set(0, mount_y - 0.048, 0.007);
    sw_group.add(&label);
    sw_group.rotation().y = (-(ndx as f64)).atan2(-(ndz as f64));
    let sx = g.wx(swi.0) + ndx as f64 * (CELL * 0.5 - 0.006);
    let sz = g.wz(swi.1) + ndz as f64 * (CELL * 0.5 - 0.006);
    sw_group.position().set(sx, 0, sz);
    g.world.add(&sw_group);
    // the distant ceiling glow panel (starts OFF)
    let glow_group = group();
    let ceil_y = WALL_H - 0.02;
    let panel_mat = lambert!({ color: 0x404040 });
    let panel_on_mat = basic_mat!({ color: 0xffe090 });
    let frame = mesh(pg("GameplayR1Trigger__panel", box_geo!(0.60, 0.020, 0.60)), &panel_mat);
    frame.position().y = ceil_y - 0.010;
    glow_group.add(&frame);
    let lens = mesh(box_geo!(0.52, 0.010, 0.52), &panel_on_mat);
    lens.position().y = ceil_y - 0.022;
    lens.set_visible(false);
    glow_group.add(&lens);
    let light = point_light!(0xffe090, 0.0, 4.5);
    light.position().set(0, ceil_y - 0.10, 0);
    glow_group.add(&light);
    glow_group.position().set(g.wx(target.0), 0, g.wz(target.1));
    g.world.add(&glow_group);
    g.gameplay_r1_state = Some(GameplayR1State {
        switch_x: sx,
        switch_z: sz,
        switch_tile: (swi.0, swi.1),
        target_x: g.wx(target.0),
        target_z: g.wz(target.1),
        target_tile: (target.0, target.1),
        on: false,
        btn: btn.clone(),
        btn_on: btn_on_mat.clone(),
        btn_off: btn_off_mat.clone(),
        lens: lens.clone(),
        light: light.clone(),
    });
    let use_pos = v3(g.wx(swi.0) + ndx as f64 * (CELL * 0.5 - 0.150), EYE, g.wz(swi.1) + ndz as f64 * (CELL * 0.5 - 0.150));
    g.interact(
        use_pos,
        1.4,
        "[E] TOGGLE SWITCH",
        |_g| true,
        move |g| {
            let Some(st) = g.gameplay_r1_state.as_mut() else { return };
            st.on = !st.on;
            let on = st.on;
            st.btn.set_mat(if on { &st.btn_on } else { &btn_off_mat });
            st.lens.set_visible(on);
            st.light.set_intensity(if on { 0.75 } else { 0.0 });
            g.blip(1200.0, 0.05, "square", 0.02, None, 0.0, 0.0);
        },
    );
}

fn place_floor_stains(g: &mut Game, _dist: &[i32]) {
    let spec = floor_stain_by_weather(g.weather_key());
    let mut open_tiles: Vec<(i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                open_tiles.push((x, z));
            }
        }
    }
    let count = 28.min(4.max((open_tiles.len() as f64 * spec.density_mul / 15.0).floor() as usize));
    // Fisher-Yates
    for i in (1..open_tiles.len()).rev() {
        let j = (rnd() * (i + 1) as f64).floor() as usize;
        open_tiles.swap(i, j);
    }
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        let shape_type = (rnd() * 3.0).floor() as i32;
        let radius = 0.30 + rnd() * 0.40;
        let geo;
        if shape_type == 0 {
            geo = circle_geo!(radius, 8.0 + (rnd() * 4.0).floor());
        } else if shape_type == 1 {
            geo = circle_geo!(radius, 10);
            geo.scale(1.6, 0.7, 1);
        } else {
            geo = circle_geo!(radius, 12);
            geo.scale(2.4, 0.4, 1);
        }
        let mat = basic_mat!({ color: spec.color, transparent: true, opacity: spec.opacity_base * (0.7 + rnd() * 0.5), depthWrite: false });
        let decal = mesh(geo, &mat);
        decal.rotation().x = -PI / 2.0;
        decal.rotation().z = rnd() * PI * 2.0;
        let clearance = if shape_type == 2 { 1.5 } else { 2.4 };
        let ox = (rnd() - 0.5) * clearance;
        let oz = (rnd() - 0.5) * clearance;
        decal.position().set(g.wx(x) + ox, 0.003, g.wz(z) + oz);
        g.world.add(&decal);
        i += 1;
    }
}

fn place_smoking_ashtrays(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let mut open_tiles: Vec<(i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                open_tiles.push((x, z));
            }
        }
    }
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let glass_colors = [0xd0d0e0, 0xc0a880, 0x808080, 0x606080];
    let ash_mat = lambert!({ color: glass_colors[(rnd() * glass_colors.len() as f64).floor() as usize], transparent: true, opacity: 0.55 });
    let filter_mat = lambert!({ color: 0xd0a060 });
    let paper_mat = lambert!({ color: 0xf0e8d0 });
    let ember_mat = basic_mat!({ color: 0xff4020 });
    let a = group();
    let base = mesh(cyl_geo!(0.055, 0.060, 0.010, 12), &ash_mat);
    base.position().y = 0.005;
    a.add(&base);
    for i in 0..4 {
        let ang = (i as f64 / 4.0) * PI * 2.0;
        let notch = mesh(box_geo!(0.010, 0.004, 0.005), &ash_mat);
        notch.position().set(ang.cos() * 0.052, 0.010, ang.sin() * 0.052);
        notch.rotation().y = ang;
        a.add(&notch);
    }
    let ash_pile_mat = lambert!({ color: 0x404040 });
    let ash_pile = mesh(pg("SmokingAshtrays__ash", cyl_geo!(0.020, 0.020, 0.003, 8)), &ash_pile_mat);
    ash_pile.position().set(-0.010, 0.011, -0.005);
    a.add(&ash_pile);
    let (cig_x, cig_z) = (0.030, 0.030);
    let mut cig_dir = v3(-1, 0, -1);
    cig_dir.normalize();
    let cig_len = 0.070;
    let cig_paper = mesh(pg("SmokingAshtrays__paper", cyl_geo!(0.003, 0.003, cig_len * 0.7, 6)), &paper_mat);
    cig_paper.rotation().z = -PI / 2.0;
    cig_paper.rotation().y = -PI / 4.0;
    cig_paper.position().set(cig_x - cig_dir.x * cig_len * 0.35, 0.013, cig_z - cig_dir.z * cig_len * 0.35);
    a.add(&cig_paper);
    let filter = mesh(pg("SmokingAshtrays__filter", cyl_geo!(0.003, 0.003, cig_len * 0.3, 6)), &filter_mat);
    filter.rotation().z = -PI / 2.0;
    filter.rotation().y = -PI / 4.0;
    filter.position().set(cig_x + cig_dir.x * cig_len * 0.15, 0.013, cig_z + cig_dir.z * cig_len * 0.15);
    a.add(&filter);
    let ember_tip_x = cig_x - cig_dir.x * cig_len * 0.7;
    let ember_tip_z = cig_z - cig_dir.z * cig_len * 0.7;
    let ember = mesh(sphere_geo!(0.0045, 6, 5), &ember_mat);
    ember.position().set(ember_tip_x, 0.013, ember_tip_z);
    a.add(&ember);
    let ember_light = point_light!(0xff4020, 0.15, 0.4, 2);
    ember_light.position().copy(&ember.pos());
    a.add(&ember_light);
    let smoke_mat = basic_mat!({ color: 0xe8e8e8, transparent: true, opacity: 0.20 });
    struct Wisp {
        mesh: Obj,
        phase: f64,
        base_y: f64,
        base_x: f64,
        base_z: f64,
    }
    let mut wisps: Vec<Wisp> = vec![];
    for i in 0..2 {
        let wisp = mesh(sphere_geo!(0.012, 6, 5), &smoke_mat);
        wisp.scale().y = 2.5;
        let px = ember_tip_x + (rnd() - 0.5) * 0.01;
        let py = 0.030 + i as f64 * 0.03;
        let pz = ember_tip_z + (rnd() - 0.5) * 0.01;
        wisp.position().set(px, py, pz);
        let p = wisp.pos();
        wisps.push(Wisp { mesh: wisp.clone(), phase: rnd() * PI * 2.0, base_y: p.y, base_x: p.x, base_z: p.z });
        a.add(&wisp);
    }
    let ember_base_intensity = 0.15;
    let ember_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let f = 0.85 + (t * 3.0 + ember_phase).sin() * 0.12 + (t * 7.0 + ember_phase * 1.7).sin() * 0.06;
        ember_light.set_intensity(ember_base_intensity * f);
        ember.scale().set(f, f, f);
        for w in &wisps {
            {
                let mut p = w.mesh.position();
                p.y = w.base_y + (t * 0.5 + w.phase).sin() * 0.020 + 0.010;
                p.x = w.base_x + (t * 0.4 + w.phase).sin() * 0.012;
                p.z = w.base_z + (t * 0.6 + w.phase).cos() * 0.012;
            }
            let s = 0.9 + (t * 0.7 + w.phase).sin() * 0.20;
            w.mesh.scale().set(s, 2.5 * s, s);
        }
    });
    a.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    a.rotation().y = rnd() * PI * 2.0;
    g.world.add(&a);
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
