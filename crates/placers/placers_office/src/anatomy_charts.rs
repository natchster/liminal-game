//! `place_anatomy_charts`
use game::base::rng::SplitMix64;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use placers_kit::place::face_wall;
use placers_kit::place::wall_tiles;

pub(crate) fn place_anatomy_charts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    // the rails (the old frame colour's draw): walnut, beech, black paint, weathered grey
    let rail = match choice(&[0x604020, 0x8b5a2b, 0x202020, 0x504030]) {
        0x604020 => phong!({ color: 0x4e3420, specular: 0x1e1a14, shininess: 20 }),
        0x8b5a2b => phong!({ color: 0x8a6a46, specular: 0x1e1a14, shininess: 20 }),
        0x202020 => phong!({ color: 0x1e1c1a, specular: 0x262626, shininess: 26 }),
        _ => lambert!({ color: 0x5e564a }),
    };
    let ac = group();
    let mount_y = 1.60;
    ac.rotation().y = face_wall(dx, dz);
    ac.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    // swung on its cord (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.03;
    let p = ac.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    // the nail, and the chart hanging from it
    let nail = group();
    nail.position().set(0, mount_y + AC_NAIL, -0.015);
    nail.rotation().z = tilt;
    ac.add(&nail);
    let body = group();
    body.position().y = -AC_NAIL;
    nail.add(&body);
    let pic = ["anat_front", "anat_back"][(vary % 2) as usize];
    // varnished, as wall charts were: gone amber, darkest where hands rolled it
    let mut age = Age::engraving(vary);
    age.tone = [age.tone[0], age.tone[1] - 0.03, age.tone[2] - 0.08];
    age.grime += 0.06;
    pictures::add_parts(
        &body,
        &[
            (
                "Real__anatomyChartSheet",
                phong!({ map: &pictures::print_tex(pic, AC_SHEET, 1330.0, age, vary), specular: 0x24221c, shininess: 16 }),
            ),
            ("Real__anatomyChartCord", lambert!({ color: 0x6a604e })),
            ("Real__anatomyChartNail", phong!({ color: 0x3a3936, specular: 0x404040, shininess: 30 })),
        ],
    );
    if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__anatomyChartRail") {
        for y in [AC_RAIL.0, -AC_RAIL.0] {
            let r = mesh(geo.clone(), &rail);
            r.position().set(0, y, AC_RAIL.1);
            body.add(&r);
        }
    }
    g.world.add(&ac);
}

// ------------------------------------------------------ the anatomy chart ---
/// The plate, the rails' axes (above and below its middle, out from the
/// wall) and the nail above its middle (`office/maps_charts_prints_and_posters.py`: AC_*)
const AC_SHEET: (f64, f64) = (0.215, 0.300);

const AC_RAIL: (f64, f64) = (0.154, 0.0075);

const AC_NAIL: f64 = 0.305;
