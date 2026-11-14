//! `place_floor_stains`
use game::base::state::GameplayR1State;
use game::prelude::*;

pub(crate) fn place_floor_stains(g: &mut Game, _dist: &[i32]) {
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
