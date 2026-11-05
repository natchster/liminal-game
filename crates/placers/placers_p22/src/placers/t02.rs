//! Batch `t02`'s prop in this crate, made real (`docs/PROPS.md` P3): the
//! devotional triptych. A small Flemish triptych of about 1510 as a real one
//! is made and painted (CC0, `visuals::pictures`: Jan Provoost's Virgin and
//! Child with St John and Mary Magdalene on its wings, Rijksmuseum
//! SK-A-2570, photographed open, its frames with it): the centre panel in its
//! engaged frame (a carved, painted and silvered band, a gilt cove), the
//! wings on iron hinges along its front edges, opened to the room and
//! angled forward; hung flush on its nail. Its varnish gone amber, cracked,
//! grimed.
use super::*;
use game::visuals::pictures::{self, Age};

/// The centre and a wing (frame and panel), the centre's depth (`t02.py`:
/// TR_*)
const CENTRE: (f64, f64) = (0.216, 0.280);
const WING: (f64, f64) = (0.109, 0.280);
const CENTRE_D: f64 = 0.028;
const WING_D: f64 = 0.018;

// R501 religious triptych: 4% chance per level.
pub(crate) fn place_religious_triptychs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let triptych = group();
    mount_on_wall(g, &triptych, &c, 0.014, 1.68, 0.6);
    let p = triptych.pos();
    let vary = pictures::vary_of(p.x, p.z, p.y);
    // on the wall: z = 0 its face, 14 mm behind the spot
    let body = group();
    body.position().z = -0.014;
    triptych.add(&body);
    // the panels' varnish: amber, cracked finely (oak panels, not canvas)
    let mut age = Age::oil(vary);
    age.weave = 0.0;
    age.crack = (age.crack.0 * 0.6, age.crack.1 * 0.6);
    age.losses = age.losses.min(2);
    let panel = |name: &str, size: (f64, f64), salt: u32| {
        phong!({ map: &pictures::print_tex(name, size, 1400.0, age, vary ^ salt), specular: 0x1c1a16, shininess: 18 })
    };
    pictures::add_parts(&body, &[("Real__triptychCentre", panel("trip_c", CENTRE, 0))]);
    // the wings, opened forward on their hinges at the centre's front edges
    let open = 0.30 + 0.30 * f64::from((vary >> 5) % 101) / 100.0;
    if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__triptychWing") {
        for (side, name, salt) in [(-1.0, "trip_l", 0x11), (1.0, "trip_r", 0x22)] {
            let pivot = group();
            pivot.position().set(side * CENTRE.0 / 2.0, 0, CENTRE_D);
            pivot.rotation().y = -side * open;
            body.add(&pivot);
            let wing = mesh(geo.clone(), panel(name, WING, salt));
            wing.position().set(side * WING.0 / 2.0, 0, -WING_D);
            pivot.add(&wing);
        }
    }
    if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__triptychHinge") {
        let iron = phong!({ color: 0x2a2724, specular: 0x34322e, shininess: 24 });
        for side in [-1.0, 1.0] {
            for y in [-0.105, 0.105] {
                let h = mesh(geo.clone(), &iron);
                h.position().set(side * CENTRE.0 / 2.0, y, CENTRE_D - 0.0015);
                body.add(&h);
            }
        }
    }
    g.world.add(&triptych);
}
