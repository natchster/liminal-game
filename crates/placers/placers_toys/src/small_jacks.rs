//! `place_small_jacks`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small metal jacks: 8% chance per level. 4-8 six-point
// metal jack toys scattered with small rubber ball. Games family.
pub(crate) fn place_small_jacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open(g) else { return };
    let jack_colors = [0xc8c8c8, 0xd8b040, 0xa02040, 0x2050d8];
    let ball_mat = lambert!({ color: 0xd82040 });
    let grp = group();
    let n_jacks = 4 + (rnd() * 5.0).floor() as i32;
    for i in 0..n_jacks {
        let jack_mat = lambert!({ color: pick_color(&jack_colors) });
        let jack = group();
        // 3 perpendicular pins with round tips (6 tips total = jack).
        for axis in 0..3 {
            let pin = mesh(pg("SmallJacks__pin", cyl_geo!(0.0018, 0.0018, 0.026, 4)), &jack_mat);
            if axis == 1 {
                // Y-axis: no rotation.
            } else if axis == 0 {
                pin.rotation().z = PI / 2.0;
            } else {
                pin.rotation().x = PI / 2.0;
            }
            jack.add(&pin);
            // Ball tips.
            for s in [-1.0, 1.0] {
                let tip = mesh(pg("SmallJacks__knob", sphere_geo!(0.003, 5, 4)), &jack_mat);
                if axis == 1 {
                    tip.position().y = s * 0.013;
                } else if axis == 0 {
                    tip.position().x = s * 0.013;
                } else {
                    tip.position().z = s * 0.013;
                }
                jack.add(&tip);
            }
        }
        // Keep clear of the jacks already down and of the ball (reach 1.6 cm);
        // widen the scatter a little on each retry when it gets crowded.
        let (mut jx, mut jz) = (0.0, 0.0);
        for tries in 0..30 {
            let spread = 1.0 + tries as f64 * 0.06;
            jx = (rnd() - 0.5) * 0.15 * spread;
            jz = (rnd() - 0.5) * 0.12 * spread;
            if hypot2(jx - 0.06, jz - 0.04) >= 0.028
                && grp.children().iter().all(|o| {
                    let p = o.pos();
                    hypot2(p.x - jx, p.z - jz) >= 0.034
                })
            {
                break;
            }
        }
        jack.position().set(jx, 0.013, jz);
        jack.rotation().set(rnd() * PI, rnd() * PI, rnd() * PI);
        grp.add(&jack);
    }
    // Small red rubber ball nearby.
    let ball = mesh(pg("SmallJacks__ball", sphere_geo!(0.010, 8, 6)), &ball_mat);
    ball.position().set(0.06, 0.010, 0.04);
    grp.add(&ball);
    scatter_pos(g, &grp, x, z, 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}
