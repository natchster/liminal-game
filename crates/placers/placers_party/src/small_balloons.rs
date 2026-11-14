//! `place_small_balloons`: Small balloon: 10% chance per level. Colored sphere floating low with string extending down to floor. Children's party remnant.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small balloon: 10% chance per level. Colored sphere floating
/// low with string extending down to floor. Children's party remnant.
pub(crate) fn place_small_balloons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let balloon_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0xd050a0, 0xa020a0];
    let balloon_mat = lambert!({ color: pick(&balloon_colors) });
    let string_mat = lambert!({ color: 0xd8d8d8 });
    let balloon = group();
    // 25% chance: fully deflated variant — flat rubber puddle.
    let is_deflated = rnd() < 0.25;
    let body = if is_deflated {
        mesh(pg("SmallBalloons__skin", circle_geo!(0.08 + rnd() * 0.04, 10)), &balloon_mat)
    } else {
        mesh(pg("SmallBalloons__body", sphere_geo!(0.10, 12, 10)), &balloon_mat)
    };
    if is_deflated {
        body.rotation().x = -PI / 2.0;
        body.rotation().z = rnd() * PI * 2.0;
        body.scale().set(1.3, 0.85, 1);
        body.position().y = 0.003;
    } else {
        body.scale().set(1, 1.15, 1);
        body.position().y = 0.5 + rnd() * 0.7;
    }
    let float_y = body.pos().y;
    balloon.add(&body);
    if !is_deflated {
        // Small tie-off pinch at bottom.
        let tie = mesh(pg("SmallBalloons__knot", cone_geo!(0.015, 0.03, 5)), &balloon_mat);
        tie.rotation().x = PI;
        tie.position().y = float_y - 0.11;
        balloon.add(&tie);
        // String from tie down to floor.
        let string_len = float_y - 0.14;
        let string = mesh(pg("SmallBalloons__string", cyl_geo!(0.002, 0.002, string_len, 4)), &string_mat);
        string.position().y = float_y - 0.14 - string_len / 2.0;
        balloon.add(&string);
    }
    balloon.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    g.world.add(&balloon);
    // 30% chance: slow sway animator (only on inflated variant).
    if !is_deflated && rnd() < 0.30 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 8.0 + rnd() * 6.0;
        let bp = body.pos();
        let (base_x, base_z) = (bp.x, bp.z);
        let b = body.clone();
        g.animate(move |_g, _dt, t| {
            let wobble = (t * (PI * 2.0 / sway_period) + sway_phase).sin();
            {
                let mut p = b.position();
                p.x = base_x + wobble * 0.020;
                p.z = base_z + (t * (PI * 2.0 / sway_period) + sway_phase).cos() * 0.015;
            }
            b.rotation().z = wobble * 0.05;
        });
    }
    // 25% chance: 1-2 sibling balloons at different heights.
    if rnd() < 0.25 {
        let sib_count = 1 + (rnd() * 2.0).floor() as i32;
        for _ in 0..sib_count {
            let sib_color = pick(&balloon_colors);
            let sib_mat = lambert!({ color: sib_color });
            let sib_balloon = group();
            let sib_body = mesh(pg("SmallBalloons__body", sphere_geo!(0.10, 12, 10)), &sib_mat);
            sib_body.scale().set(1, 1.15, 1);
            let sib_float_y = 0.5 + rnd() * 0.7;
            sib_body.position().y = sib_float_y;
            sib_balloon.add(&sib_body);
            let sib_tie = mesh(pg("SmallBalloons__knot", cone_geo!(0.015, 0.03, 5)), &sib_mat);
            sib_tie.rotation().x = PI;
            sib_tie.position().y = sib_float_y - 0.11;
            sib_balloon.add(&sib_tie);
            let sib_string_len = sib_float_y - 0.14;
            let sib_string = mesh(pg("SmallBalloons__string", cyl_geo!(0.002, 0.002, sib_string_len, 4)), &string_mat);
            sib_string.position().y = sib_float_y - 0.14 - sib_string_len / 2.0;
            sib_balloon.add(&sib_string);
            let s_ang = rnd() * PI * 2.0;
            let s_rad = 0.20 + rnd() * 0.20;
            let bp = balloon.pos();
            sib_balloon.position().set(bp.x + s_ang.cos() * s_rad, 0, bp.z + s_ang.sin() * s_rad);
            g.world.add(&sib_balloon);
        }
    }
}
