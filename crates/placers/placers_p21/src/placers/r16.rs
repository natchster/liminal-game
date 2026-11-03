//! Batch `r16`'s props in this crate, made real (`docs/PROPS.md` P2): its
//! placers and what only they use.
use super::*;

// R514 a broken bottle (Blender batch `r16`): a 12 oz longneck dropped and
// burst on the floor, green, amber or clear glass (the old colours): its
// base standing where it hit, its wall broken off in teeth; the neck and
// shoulder in one piece, rolled away on their side; curved pieces of the
// wall lying about, convex up, on their bellies, one on its edge; a glitter
// of chips; all of it dulled by dust and dried mud. 6% chance per level.
pub(crate) fn place_broken_bottles(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // green, amber, clear (the old green, brown and blue)
    let glass_colors = [0x306040, 0x603820, 0x304060];
    let glass = pick_hex(&glass_colors);
    let kind = glass_colors.iter().position(|&c| c == glass).unwrap_or(0);
    // the old base's four tabs: an angle's jitter and a height each
    let mut tabs = [(0.0f64, 0.0f64); 4];
    for t in tabs.iter_mut() {
        t.0 = rnd() * 0.4;
        t.1 = rnd() * 0.008;
    }
    // the old six shards: a size, two shape draws, where (x, z), its turn
    let mut shards = [[0.0f64; 6]; 6];
    for s in shards.iter_mut() {
        s[0] = 0.014 + rnd() * 0.016;
        s[1] = rnd() - 0.5;
        s[2] = 0.5 + rnd() * 0.5;
        s[3] = (rnd() - 0.5) * 0.20;
        s[4] = (rnd() - 0.5) * 0.20;
        s[5] = rnd() * PI * 2.0;
    }
    let salt = tabs.iter().fold(0x0b07_71e5u32, |h, t| h.rotate_left(9) ^ (t.0 * 1.0e7) as u32 ^ (t.1 * 1.0e9) as u32);
    let mat = bottle_glass_mat(kind, &bottle_grime_tex(kind, salt));
    let grp = group();
    let put = |name: &str, x: f64, z: f64, turn: f64| {
        if let Some(geo) = prop_part_as_modelled(name) {
            let m = mesh(geo, &mat);
            m.position().set(x, 0, z);
            m.rotation().y = turn;
            grp.add(&m);
        }
    };
    // the base where it hit, the neck rolled off (the old neck's place)
    put("Real__bottleBase", 0.0, 0.0, tabs[0].0 * 15.7);
    put("Real__bottleNeck", 0.085, 0.055, 0.6 + tabs[1].0 * 4.0 + tabs[2].1 * 120.0);
    put("Real__bottleChips", 0.0, 0.0, tabs[3].0 * 15.7);
    // the pieces of its wall where the old shards lay, kept off the base
    // and the neck
    for s in &shards {
        let which = (((s[0] - 0.014) / 0.016 * 3.0) as usize + if s[1] > 0.0 { 2 } else { 0 }).min(4);
        let (mut sx, mut sz) = (s[3], s[4]);
        let d = (sx * sx + sz * sz).sqrt();
        if d < 0.055 {
            let k = 0.055 / d.max(1e-3);
            sx *= k;
            sz *= k;
            if d < 1e-3 {
                sx = 0.055 * (s[2] * 7.0).cos();
                sz = 0.055 * (s[2] * 7.0).sin();
            }
        }
        if ((sx - 0.085).powi(2) + (sz - 0.055).powi(2)).sqrt() < 0.06 {
            sx -= 0.05;
            sz -= 0.10;
        }
        put(
            ["Real__bottleShardA", "Real__bottleShardB", "Real__bottleShardC", "Real__bottleShardD", "Real__bottleShardE"][which],
            sx,
            sz,
            s[5],
        );
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}

/// Bottle glass: its body colour (linear) and how much of it the light
/// meets (thick green and amber glass, thin clear), the rest the room seen
/// in it and its highlights (`Patch::Glass`, as the windows' panes)
const BOTTLE_GLASS: [([f32; 3], f32); 3] = [([0.05, 0.15, 0.06], 0.66), ([0.2, 0.09, 0.022], 0.7), ([0.26, 0.28, 0.25], 0.44)];

/// The glass of the bottle: the room seen in it by Fresnel (bright at its
/// edges and the breaks, where it is seen at a graze), its colour and dust
/// a film over it (the map's grey: how much)
fn bottle_glass_mat(kind: usize, map: &Tex) -> Mat {
    let (tint, body) = BOTTLE_GLASS[kind];
    let m = phong!({ color: 0xffffff, specular: 0xb0b0b0, shininess: 220, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "bottle_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it, nothing lit behind it
        f[0] = 0.55;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // its body: how much of the map's grey, its colour
        f[8] = body;
        f[9] = tint[0];
        f[10] = tint[1];
        f[11] = tint[2];
        f[12] = 0.0;
        // two faces of glass
        f[13] = 0.10;
    }
    m
}

fn bottle_hash(seed: u32, x: i32, y: i32) -> f64 {
    let mut h = seed ^ (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h = (h ^ (h >> 13)).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    h as f64 / 4_294_967_296.0
}

fn bottle_noise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (fx, fy) = (x - xf, y - yf);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xf as i32, yf as i32);
    let (a, b) = (bottle_hash(seed, xi, yi), bottle_hash(seed, xi + 1, yi));
    let (c, d) = (bottle_hash(seed, xi, yi + 1), bottle_hash(seed, xi + 1, yi + 1));
    a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}

/// Dust and dried mud on the glass (grey: how much of the glass's film is
/// there: its body where it is clean, more where dust and mud dried on it)
fn bottle_grime_tex(kind: usize, salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0xb071;
        let film = if kind == 2 { 0.9 } else { 0.4 };
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = (i as f64, j as f64);
                let d = bottle_noise(seed, x / 18.0, y / 18.0) * 0.6 + bottle_noise(seed ^ 1, x / 5.0, y / 5.0) * 0.4;
                let mud = ((d - 0.5) * 2.5).clamp(0.0, 1.0) * film;
                let speck = (bottle_noise(seed ^ 2, x / 1.3, y / 1.3) - 0.85).max(0.0) * 4.0;
                let v = ((0.72 + 0.28 * (mud + speck).min(1.0)) * 255.0) as u8;
                out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}
