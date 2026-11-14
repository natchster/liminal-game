//! `place_ornate_wall_mirrors`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use placers_kit::geometry::put as t10_put;
use placers_kit::noise::fbm as t10_fbm;
use placers_kit::noise::hash as t10_hash;
use placers_kit::noise::sstep as t10_ss;
use placers_kit::noise::vn as t10_vn;
use placers_kit::paint::fill as t10_fill;
use placers_kit::place::pick;
use placers_kit::place::WallCand;

// Ornate wall mirror: 5% chance per level. A small Italian giltwood
// mirror in the rococo revival (Blender batch `t10`): a moulded frame of
// gesso over carved wood, water gilt, a pierced rocaille crest of shell,
// C-scrolls and acanthus, leaves breaking out at its corners and a little
// shell for an apron; the gold dulled brown and rubbed through to its red
// bole and white gesso, dust in its hollows. Its glass gives back the
// room's light (`Patch::Glass`), its silver going: black creeping in
// from the edges, foxed in spots, a bloom where damp got behind it.
pub(crate) fn place_ornate_wall_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    let mirror = group();
    // Position on wall.
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.014);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.014);
    let center_y = 1.60;
    if c.dx.abs() == 1 {
        mirror.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        mirror.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        mirror.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        mirror.position().set(wall_x + (rnd() - 0.5) * 0.6, center_y, wall_z);
    }
    let p = mirror.pos();
    let vary = pictures::vary_of(p.x, p.z, center_y);
    // the parts are modelled with the wall at z = 0: 14 mm behind the spot
    let body = group();
    body.position().z = -0.014;
    mirror.add(&body);
    t10_put(
        &body,
        "Real__ornateMirrorFrame",
        &phong!({ map: &pictures::moulding_tex(OM_OUTER, &OM_FACE, Outline::Rect, Finish::Gilt, 1100.0, vary), specular: 0x3a3020, shininess: 22 }),
    );
    t10_put(&body, "Real__ornateMirrorCrest", &phong!({ map: &om_gilt_tex(vary), specular: 0x3a3020, shininess: 22 }));
    // the black paint on the glass's back, seen where the silver has gone
    let back = mesh(plane_geo!(OM_GLASS[0] + 0.012, OM_GLASS[1] + 0.012), lambert!({ color: 0x141312 }));
    back.position().z = OM_GLASS[2] - 0.0035;
    body.add(&back);
    let glass = mesh(plane_geo!(OM_GLASS[0] + OM_UNDER, OM_GLASS[1] + OM_UNDER), om_glass_mat(om_silver_tex(vary)));
    glass.position().z = OM_GLASS[2] - 0.0001;
    glass.set_render_order(2);
    body.add(&glass);
    let film = mesh(
        plane_geo!(OM_GLASS[0], OM_GLASS[1]),
        lambert!({ map: &om_film_tex(vary), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    film.position().z = OM_GLASS[2] + 0.0003;
    film.set_render_order(3);
    body.add(&film);
    g.world.add(&mirror);
}

/// Every (open tile, wall direction) pair whose neighbour is a wall or off the grid.
fn wall_candidates(g: &Game) -> Vec<WallCand> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if g.is_wall_tile(nx, nz) {
                    v.push(WallCand { x, z, dx, dz });
                }
            }
        }
    }
    v
}

fn t10_mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// The mirror (t10.py `OM_*`): its moulding's outer size (w, h), the
/// moulding's face from its outer edge to the sight (d, z), the glass in
/// sight (w, h), its face's distance from the wall, and how much more of it
/// runs under the lip (both sides)
const OM_OUTER: (f64, f64) = (0.300, 0.400);

const OM_FACE: [(f64, f64); 16] = [
    (0.0, 0.0120),
    (0.0015, 0.0155),
    (0.0040, 0.0170),
    (0.0065, 0.0165),
    (0.0085, 0.0140),
    (0.0120, 0.0150),
    (0.0170, 0.0200),
    (0.0230, 0.0260),
    (0.0290, 0.0290),
    (0.0330, 0.0285),
    (0.0370, 0.0250),
    (0.0405, 0.0205),
    (0.0430, 0.0195),
    (0.0455, 0.0215),
    (0.0480, 0.0205),
    (0.0500, 0.0170),
];

const OM_GLASS: [f64; 3] = [0.200, 0.300, 0.0125];

const OM_UNDER: f64 = 0.010;

/// The carved ornament's gilding (its crest, corners and apron, as their
/// fronts are mapped): leaf laid in squares, dulled toward brown, burnished
/// and rubbed through to the red bole and the white gesso where it stands
/// out, grime and dust in its hollows
fn om_gilt_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0x9117;
        t10_fill(ctx, w, h, |u, v| {
            let sq = t10_hash(seed, (u * 14.0) as i32, (v * 14.0) as i32);
            let mut o = [0.6 + 0.06 * sq, 0.47 + 0.05 * sq, 0.25 + 0.03 * sq];
            o = t10_mix(o, [0.40, 0.32, 0.19], 0.45 + 0.25 * t10_fbm(seed ^ 3, u * 9.0, v * 9.0, 3));
            let n = t10_fbm(seed ^ 5, u * 30.0, v * 30.0, 3);
            o = t10_mix(o, [0.46, 0.22, 0.13], 0.8 * t10_ss(0.62, 0.74, n));
            o = t10_mix(o, [0.78, 0.74, 0.64], t10_ss(0.76, 0.84, n));
            let hollow = t10_ss(0.55, 0.8, t10_fbm(seed ^ 7, u * 60.0, v * 60.0, 2));
            o = o.map(|c| c * (1.0 - 0.35 * hollow));
            let dust = 0.10 + 0.15 * t10_fbm(seed ^ 9, u * 12.0, v * 12.0, 3) + 0.12 * v;
            let o = t10_mix(o, [0.52, 0.49, 0.44], dust.min(0.45));
            [o[0], o[1], o[2], 1.0]
        });
    })
}

/// The glass: silver behind it (`Patch::Glass`, nearly all of the light
/// back), its map where the silver has gone to the black paint behind
fn om_glass_mat(silver: Tex) -> Mat {
    // the lamp seen in it as a lamp is in a mirror: a small hot spot the
    // size of its lens, not a disc of glare round it (at 4000 the highlight
    // read as bloom: M8)
    let m = phong!({ color: 0xffffff, specular: 0x2a2a2a, shininess: MIRROR_SHINE, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(silver);
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "mirror_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it (the light where it hangs, off what it faces), nothing behind it
        f[0] = 0.13;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // the map's grey is the silver gone, to the black paint behind (linear)
        f[8] = 1.0;
        f[9] = 0.016;
        f[10] = 0.015;
        f[11] = 0.013;
        f[12] = 0.0;
        // silvered: most of the light comes back at any angle
        f[13] = 0.86;
    }
    m
}

/// Where the silver has gone (grey: how much): black feathering in from the
/// edges under the lip and beyond it, worst at the foot where damp stood;
/// foxing in clusters of spots; a cloudy bloom
fn om_silver_tex(vary: u32) -> Tex {
    canvas_tex_opt(160, 224, false, move |ctx, w, h| {
        let seed = vary ^ 0x5117;
        let (gw, gh) = (OM_GLASS[0] + OM_UNDER, OM_GLASS[1] + OM_UNDER);
        let bloom = (0.2 + 0.6 * t10_hash(seed, 1, 1), 0.15 + 0.5 * t10_hash(seed, 2, 2));
        t10_fill(ctx, w, h, |u, v| {
            let (x, y) = ((u - 0.5) * gw, (v - 0.5) * gh);
            // in from the nearest edge, the foot's counted double
            let edge = (gw / 2.0 - x.abs()).min(gh / 2.0 - y.abs()).min((y + gh / 2.0) * 0.5);
            let wave = t10_fbm(seed, x * 30.0, y * 30.0, 4);
            let creep = 1.0 - t10_ss(0.004, 0.012 + 0.016 * wave, edge);
            // tendrils reaching further in
            let tendril = (1.0 - t10_ss(0.010, 0.030, edge)) * t10_ss(0.62, 0.7, t10_fbm(seed ^ 3, x * 80.0, y * 80.0, 3));
            // foxing: spots in clusters
            let cl = t10_ss(0.6, 0.75, t10_fbm(seed ^ 5, x * 12.0, y * 12.0, 3));
            let spot = if t10_hash(seed ^ 7, (u * 160.0) as i32, (v * 224.0) as i32) < 0.03 * cl + 0.002 { 0.85 } else { 0.0 };
            let soft = cl * 0.25 * t10_vn(seed ^ 9, x * 300.0, y * 300.0);
            // the bloom where damp stood behind it
            let (bx, by) = ((u - bloom.0) * gw, (v - bloom.1) * gh);
            let b = (1.0 - t10_ss(0.02, 0.07, (bx * bx + by * by).sqrt())) * 0.4 * t10_fbm(seed ^ 11, x * 40.0, y * 40.0, 3);
            let k = (creep + tendril + spot + soft + b).min(1.0);
            [k, k, k, 1.0]
        });
    })
}

/// What lies on the glass: a grey dust thicker toward its top edge, fly
/// specks, an old wipe's arc
fn om_film_tex(vary: u32) -> Tex {
    canvas_tex_opt(160, 224, false, move |ctx, w, h| {
        let seed = vary ^ 0xf117;
        t10_fill(ctx, w, h, |u, v| {
            let n = t10_fbm(seed, u * 6.0, v * 8.0, 4);
            let wipe = ((u - 0.5).powi(2) + (v - 0.35).powi(2)).sqrt();
            let wiped = t10_ss(0.30, 0.36, wipe);
            let mut a = (0.06 + 0.16 * n + 0.10 * v) * (0.4 + 0.6 * wiped);
            let mut c = [0.60, 0.57, 0.52];
            if t10_hash(seed ^ 3, (u * 160.0) as i32, (v * 224.0) as i32) > 0.996 {
                a = 0.85;
                c = [0.10, 0.08, 0.06];
            }
            [c[0], c[1], c[2], a.min(0.85)]
        });
    })
}
