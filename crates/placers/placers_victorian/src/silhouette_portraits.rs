//! `place_silhouette_portraits`
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use placers_kit::place::ri;

// Silhouette portrait (wall): 3% chance per level. A cut-paper
// silhouette of the 1830s-60s in its oval frame, 18 x 22 cm, hung by its
// ring a little askew (the old tilt's draw, now about its hook).
pub(crate) fn place_silhouette_portraits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let mut wall_tiles: Vec<(i32, i32, i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        wall_tiles.push((x, z, dx, dz));
                        break;
                    }
                }
            }
        }
    }
    if wall_tiles.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wall_tiles[ri(wall_tiles.len())];
    // the old frame colour's pick: the frame's finish
    let frame_colors: [u32; 4] = [0x604020, 0xd0a040, 0x2a1810, 0xc0a060];
    let frame = frame_colors[ri(frame_colors.len())];
    let sp = group();
    sp.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    sp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    let tilt = (rnd() - 0.5) * 0.03;
    let p = sp.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let (finish, tint) = match frame {
        0x604020 => (Finish::Wood([0x4a, 0x2e, 0x1c]), 0xffffff),
        0xd0a040 => (Finish::Gilt, 0xffffff),
        0x2a1810 => (Finish::Ebony, 0xffffff),
        _ => (Finish::Gilt, 0xb0a48a),
    };
    let body = pictures::hang(&sp, 1.60, 0.015, APEX, FRAME.1, 0.004, tilt);
    let brass = phong!({ color: 0x6a5632, specular: 0x5a4a2c, shininess: 40 });
    let parts: [(&str, Mat); 5] = [
        (
            "Real__silhouetteFrame",
            phong!({ map: &pictures::moulding_tex(FRAME, &FACE, Outline::Oval, finish, 1400.0, vary), color: tint, specular: 0x2a2418, shininess: 24 }),
        ),
        ("Real__silhouetteSlip", brass.clone()),
        ("Real__silhouetteRing", brass),
        ("Real__silhouetteCard", lambert!({ map: &silhouette_tex(vary) })),
        ("Real__silhouetteGlass", pictures::glass_mat(vary, 0.2 + 0.25 * (vary % 53) as f32 / 53.0)),
    ];
    pictures::add_parts(&body, &parts);
    g.world.add(&sp);
}

/// The frame's outside, the card's (`t19.py`: SIL_A, SIL_B, the card's
/// bounds) and the ring's inside top above the frame's middle (where the
/// hook bears)
const FRAME: (f64, f64) = (0.180, 0.220);

const CARD: (f64, f64) = (0.144, 0.184);

const APEX: f64 = 0.1207;

/// The moulding's face, outer edge to sight (d, z), as `SIL_SECTION`
static FACE: [(f64, f64); 10] = [
    (0.0, 0.0100),
    (0.0015, 0.0135),
    (0.0040, 0.0152),
    (0.0070, 0.0156),
    (0.0100, 0.0148),
    (0.0130, 0.0126),
    (0.0155, 0.0129),
    (0.0175, 0.0140),
    (0.0195, 0.0131),
    (0.0210, 0.0110),
];

/// The three sitters cut in profile (mm on the card, y down, facing +x): a
/// gentleman in a high stock and coat, a lady with her hair in a knot and a
/// lace collar, a child in a frilled collar; each bust cut off below in a
/// curve, as the cutters did
static GENTLEMAN: [(f64, f64); 42] = [
    (-18.0, -2.0),
    (-23.0, -10.0),
    (-27.0, -22.0),
    (-27.0, -38.0),
    (-20.0, -53.0),
    (-6.0, -61.0),
    (8.0, -60.0),
    (16.0, -55.0),
    (18.0, -49.0),
    (17.0, -43.0),
    (18.0, -37.0),
    (20.0, -34.0),
    (18.0, -31.0),
    (21.0, -26.0),
    (26.0, -20.0),
    (25.0, -18.0),
    (20.0, -17.0),
    (20.0, -15.0),
    (18.0, -13.0),
    (20.0, -11.0),
    (18.0, -9.0),
    (19.0, -5.0),
    (20.0, -1.0),
    (15.0, 3.0),
    (9.0, 6.0),
    (9.0, 9.0),
    (15.0, 12.0),
    (17.0, 17.0),
    (16.0, 22.0),
    (20.0, 27.0),
    (16.0, 32.0),
    (19.0, 38.0),
    (22.0, 46.0),
    (24.0, 52.0),
    (10.0, 56.0),
    (-10.0, 57.0),
    (-30.0, 55.0),
    (-36.0, 48.0),
    (-36.0, 36.0),
    (-33.0, 24.0),
    (-25.0, 14.0),
    (-19.0, 6.0),
];

static LADY: [(f64, f64); 44] = [
    (-14.0, -2.0),
    (-22.0, -8.0),
    (-30.0, -14.0),
    (-33.0, -24.0),
    (-28.0, -34.0),
    (-20.0, -38.0),
    (-17.0, -46.0),
    (-8.0, -54.0),
    (4.0, -56.0),
    (13.0, -51.0),
    (16.0, -44.0),
    (16.0, -38.0),
    (17.0, -35.0),
    (16.0, -32.0),
    (19.0, -28.0),
    (23.0, -23.0),
    (22.0, -21.0),
    (17.0, -20.0),
    (18.0, -17.0),
    (16.0, -15.0),
    (18.0, -13.0),
    (16.0, -10.0),
    (16.0, -6.0),
    (13.0, -2.0),
    (7.0, 1.0),
    (6.0, 8.0),
    (7.0, 16.0),
    (9.0, 22.0),
    (14.0, 27.0),
    (17.0, 26.0),
    (19.0, 30.0),
    (17.0, 34.0),
    (21.0, 38.0),
    (19.0, 42.0),
    (23.0, 46.0),
    (24.0, 52.0),
    (8.0, 56.0),
    (-12.0, 57.0),
    (-30.0, 55.0),
    (-34.0, 46.0),
    (-30.0, 34.0),
    (-22.0, 22.0),
    (-15.0, 12.0),
    (-13.0, 4.0),
];

static CHILD: [(f64, f64); 31] = [
    (-16.0, 6.0),
    (-24.0, -6.0),
    (-27.0, -22.0),
    (-22.0, -38.0),
    (-10.0, -48.0),
    (4.0, -50.0),
    (15.0, -44.0),
    (19.0, -34.0),
    (19.0, -28.0),
    (23.0, -22.0),
    (22.0, -19.0),
    (18.0, -18.0),
    (19.0, -15.0),
    (17.0, -12.0),
    (18.0, -8.0),
    (15.0, -4.0),
    (9.0, -1.0),
    (8.0, 5.0),
    (13.0, 8.0),
    (17.0, 11.0),
    (13.0, 14.0),
    (16.0, 18.0),
    (15.0, 26.0),
    (18.0, 36.0),
    (19.0, 44.0),
    (6.0, 48.0),
    (-14.0, 49.0),
    (-28.0, 46.0),
    (-31.0, 36.0),
    (-27.0, 24.0),
    (-20.0, 14.0),
];

/// What was pencilled under each, in an old hand (invented sitters)
const NAMES: [[&str; 2]; 3] =
    [["Mr. Jas. Holt, 1834", "Wm. Ashby, aet. 41"], ["Eliza Ann, aet. 23", "Mrs. Pell, 1839"], ["Little Mary", "Thos., aet. 6"]];

/// The card (its whole face, 14.4 x 18.4 cm; the slip's window shows its
/// middle 12.7 x 16.7): cream wove card, the bust cut from black paper
/// laid on it facing left or right, the sitter's name pencilled under it;
/// aged as a print is (the card gone tan, foxed, the black gone a little
/// brown and grey where the light reached it)
fn silhouette_tex(vary: u32) -> Tex {
    let ppm = 1600.0;
    let mut age = Age::engraving(vary);
    age.crease = false;
    age.stain = 0.0;
    age.fade = 0.02 + 0.04 * (vary % 7) as f64 / 7.0;
    pictures::sheet_tex(CARD, ppm, age, vary, move |ctx, w, h| {
        let mm = ppm / 1000.0;
        ctx.set_fill_style("#e4dbc4");
        ctx.fill_rect(0, 0, w, h);
        let sitter = (vary / 3 % 3) as usize;
        let pts: &[(f64, f64)] = match sitter {
            0 => &GENTLEMAN,
            1 => &LADY,
            _ => &CHILD,
        };
        let flip = if vary % 2 == 0 { 1.0 } else { -1.0 };
        let at = |p: (f64, f64)| (w / 2.0 + flip * (p.0 + 5.0) * mm, h / 2.0 + (p.1 - 4.0) * mm);
        // a closed Catmull-Rom curve through the cutter's points
        let n = pts.len();
        ctx.set_fill_style("#141210");
        ctx.begin_path();
        let p0 = at(pts[0]);
        ctx.move_to(p0.0, p0.1);
        for i in 0..n {
            let (a, b, c, d) = (at(pts[(i + n - 1) % n]), at(pts[i]), at(pts[(i + 1) % n]), at(pts[(i + 2) % n]));
            let c1 = (b.0 + (c.0 - a.0) / 6.0, b.1 + (c.1 - a.1) / 6.0);
            let c2 = (c.0 - (d.0 - b.0) / 6.0, c.1 - (d.1 - b.1) / 6.0);
            ctx.bezier_curve_to(c1.0, c1.1, c2.0, c2.1, c.0, c.1);
        }
        ctx.close_path();
        ctx.fill();
        // pencil under the bust, in an old hand
        ctx.set_fill_style("rgba(90,86,80,0.55)");
        ctx.set_font("italic 13px cursive");
        ctx.set_text_align("center");
        ctx.fill_text(NAMES[sitter][(vary / 9 % 2) as usize], w / 2.0, h / 2.0 + 66.0 * mm);
    })
}
