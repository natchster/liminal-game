//! Prop placers: the R1 gameplay trigger (a wall switch that lights a distant
//! ceiling glow panel), floor stains and smoking ashtrays.
#![allow(unused_variables, unused_mut, unused_assignments, clippy::all)]
use game::base::state::GameplayR1State;
use game::prelude::*;

mod t07;
use t07::place_smoking_ashtrays;

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

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
