//! `place_miniature_portrait_clusters`
use game::assets::props::has_prop_part;
use game::assets::props::prop_part_as_modelled;
use game::assets::props::props_enabled;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use placers_kit::place::mount_on_wall;
use placers_kit::place::wall_candidates;

// Cluster of 3-4 small oval portrait miniatures (wall): 3% chance per level.
pub(crate) fn place_miniature_portrait_clusters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // the cases: gilt, gilt gone brown, gilt gone olive, ebonised wood
    let frame_colors = [0xc0a060, 0x8b6f30, 0x907040, 0x2a1810];
    let mp = group();
    let silh_colors = [0x181810, 0x201810, 0x302010, 0x281810];
    let mini_count = 3 + (rnd() * 2.0).floor() as i32; // 3 or 4 miniatures
    let arrangements: Vec<Vec<(f64, f64)>> = vec![
        // 3-miniature triangle.
        vec![(-0.10, 0.06), (0.10, 0.06), (0.0, -0.08)],
        // 3-miniature horizontal row.
        vec![(-0.14, 0.0), (0.0, 0.0), (0.14, 0.0)],
        // 3-miniature vertical.
        vec![(0.0, 0.12), (0.0, 0.0), (0.0, -0.12)],
        // 4-miniature diamond.
        vec![(0.0, 0.12), (-0.12, 0.0), (0.12, 0.0), (0.0, -0.12)],
        // 4-miniature 2x2 grid.
        vec![(-0.08, 0.08), (0.08, 0.08), (-0.08, -0.08), (0.08, -0.08)],
    ];
    let pick = if mini_count == 3 { (rnd() * 3.0).floor() as usize } else { 3 + (rnd() * 2.0).floor() as usize };
    let arr_idx = (arrangements.len() - 1).min(pick);
    let arrangement = &arrangements[arr_idx];
    let use_count = (mini_count as usize).min(arrangement.len());
    // what the old cluster built of fallback rings (a frame each, and a hook
    // each where one hung), whose count its frame-tilt animator drew from:
    // none when the old parts are there (props on), so neither here
    let ring_frames = !props_enabled() || !has_prop_part("MiniaturePortraitClusters__frame");
    let ring_hooks = !props_enabled() || !has_prop_part("MiniaturePortraitClusters__hook");
    let mut ringed: Vec<usize> = vec![];
    let mut minis: Vec<(Obj, u32, u32, bool)> = vec![];
    for i in 0..use_count {
        let (ox, oy) = arrangement[i];
        let is_shared_frame = rnd() < 0.7;
        let frame_color = if is_shared_frame { frame_colors[0] } else { choice(&frame_colors) };
        // its size (the old ring's radius): the whole miniature, 0.9 to 1.1
        let mini_r = 0.045 + rnd() * 0.010;
        // the old silhouette's shade: kept as a salt for this one's painting
        let shade: u32 = choice(&silh_colors);
        // 60%: hung from a pin through its ring, else by a ribbon
        let pinned = rnd() < 0.6;
        let mini = group();
        mini.position().set(ox, oy, -0.004);
        mini.scale().set_scalar(mini_r / 0.05);
        mp.add(&mini);
        if ring_frames {
            ringed.push(i);
        }
        if pinned && ring_hooks {
            ringed.push(i);
        }
        minis.push((mini, frame_color, shade, pinned));
    }
    // 40%: a ribbon bow over the group (four of them)
    let bow = rnd() < 0.4 && use_count >= 4;
    mount_on_wall(g, &mp, &c, 0.004, 1.55, 0.4);
    mp.rotation().z = (rnd() - 0.5) * 0.010;
    let p = mp.pos();
    let vary = pictures::vary_of(p.x, p.z, mp.rot().z);
    let start = (vary % 4) as usize;
    for (i, (mini, frame_color, shade, pinned)) in minis.iter().enumerate() {
        let v = vary ^ shade.wrapping_mul(0x9e37_79b9) ^ (i as u32).wrapping_mul(0x85eb_ca6b);
        let (finish, tint) = match *frame_color {
            0x2a1810 => (Finish::Ebony, 0xffffff),
            0x8b6f30 => (Finish::Gilt, 0xb0a48a),
            0x907040 => (Finish::Gilt, 0xb4ae8c),
            _ => (Finish::Gilt, 0xffffff),
        };
        let case_mat = phong!({ map: &pictures::moulding_tex((2.0 * CASE.0, 2.0 * CASE.1), &FACE, Outline::Oval, finish, 3000.0, v), color: tint, specular: 0x3a3020, shininess: 30 });
        let brass = phong!({ color: 0x6a5632, specular: 0x5a4a2c, shininess: 40 });
        let mut age = Age::photo(v);
        age.fade = 0.04;
        age.tone = [1.0, 0.95, 0.86];
        age.silver = 0.0;
        age.spots = (v % 4) as usize;
        age.crease = false;
        age.stain = 0.0;
        let pic = MINIATURES[(start + i) % 4];
        let ivory_mat = lambert!({ map: &pictures::print_tex(pic, (2.0 * IVORY.0, 2.0 * IVORY.1), 2600.0, age, v) });
        let parts: [(&str, &Mat); 4] = [
            ("Real__miniatureFrame", &case_mat),
            ("Real__miniatureRing", if let Finish::Ebony = finish { &brass } else { &case_mat }),
            ("Real__miniatureIvory", &ivory_mat),
            ("Real__miniatureGlass", &pictures::glass_mat(v, 0.15 + 0.2 * (v % 71) as f32 / 71.0)),
        ];
        for (name, mat) in parts {
            if let Some(geo) = prop_part_as_modelled(name) {
                let m = mesh(geo, mat);
                if name.ends_with("Glass") {
                    m.set_render_order(2);
                }
                mini.add(&m);
            }
        }
        // hung from a brass pin through its ring, or by a ribbon from a pin above
        let pin_y = if *pinned { RING_TOP } else { RING_TOP + 0.060 };
        if let Some(geo) = prop_part_as_modelled("Real__miniaturePin") {
            let m = mesh(geo, &brass);
            m.position().y = pin_y;
            mini.add(&m);
        }
        if !*pinned && let Some(geo) = prop_part_as_modelled("Real__miniatureRibbon") {
            let ribbon = [0x6e5450, 0x4e5a52, 0x5e5468][(v / 3 % 3) as usize];
            mini.add(&mesh(geo, lambert!({ color: ribbon, side: DoubleSide })));
        }
    }
    if bow && let Some(geo) = prop_part_as_modelled("Real__miniatureBow") {
        let top = arrangement.iter().take(use_count).map(|a| a.1).fold(f64::MIN, f64::max);
        let m = mesh(geo, lambert!({ color: 0x6a504c }));
        m.position().set(0, top + 0.105, -0.004);
        mp.add(&m);
    }
    g.world.add(&mp);
    // Frame-tilt animator: one miniature drifts slightly askew then
    // returns. The old cluster drew it only from its fallback rings (props
    // off); so does this one
    if !ringed.is_empty() {
        let target = minis[ringed[(rnd() * ringed.len() as f64).floor() as usize]].0.clone();
        let base_rot_z = target.rot().z;
        let tilt_freq = (PI * 2.0) / (20.0 + rnd() * 20.0); // 20-40 s cycle
        let tilt_phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            let raw = (t * tilt_freq + tilt_phase).sin();
            let envelope = sign_of(raw) * raw.abs().powf(2.5);
            target.rotation().z = base_rot_z + envelope * 0.05; // ±3 deg peak
        });
    }
}

/// The case's outside (semi-axes), the painted oval's (`domestic/pictures_on_the_walls.py`: MN_*)
const CASE: (f64, f64) = (0.045, 0.058);

const IVORY: (f64, f64) = (0.0366, 0.0496);

/// How far the ring's top is above the case's middle (where the pin goes)
const RING_TOP: f64 = 0.058 + 0.0068 + 0.0035;

/// The case's face, outer edge to sight (d, z), as `mini_frame`
static FACE: [(f64, f64); 11] = [
    (0.0, 0.0058),
    (0.0010, 0.0080),
    (0.0025, 0.0096),
    (0.0042, 0.0104),
    (0.0058, 0.0100),
    (0.0068, 0.0092),
    (0.0078, 0.0094),
    (0.0090, 0.0097),
    (0.0100, 0.0093),
    (0.0107, 0.0084),
    (0.011, 0.0074),
];

const MINIATURES: [&str; 4] = ["mini_a", "mini_b", "mini_c", "mini_d"];
