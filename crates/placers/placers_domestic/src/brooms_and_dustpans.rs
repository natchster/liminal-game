//! `place_brooms_and_dustpans`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Broom + dustpan pair (floor): 5% chance per level.
pub(crate) fn place_brooms_and_dustpans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Broom handle color pool.
    let handle_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x505050];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    // Bristle color pool — natural straw or synthetic.
    let bristle_colors = [0xa07040, 0xc0a060, 0x805030, 0x808080, 0x1a1a1a];
    let bristle_mat = lambert!({ color: pick(&bristle_colors) });
    // Dustpan color.
    let pan_colors = [0xa02020, 0x2060a0, 0x606060, 0xd0a040, 0x101010];
    let pan_mat = lambert!({ color: pick(&pan_colors) });
    let bd = group();
    // Broom lying on floor — handle horizontal + bristle head at end.
    let broom_len = 0.90;
    let broom_handle = mesh(pg("BroomsAndDustpans__handle", cyl_geo!(0.010, 0.010, broom_len, 6)), &handle_mat);
    broom_handle.rotation().z = PI / 2.0;
    // Centred on the head's height (0..0.02) so it enters the socket boss.
    broom_handle.position().set(0, 0.010, 0);
    bd.add(&broom_handle);
    // Broom head — flat rectangular base + many bristles below.
    let head_base = mesh(pg("BroomsAndDustpans__head", box_geo!(0.14, 0.020, 0.05)), &handle_mat);
    head_base.position().set(broom_len * 0.5 - 0.02, 0.010, 0);
    bd.add(&head_base);
    // Bristle strands — 16 thin cylinders angling out radially.
    let strand_count = 16;
    for i in 0..strand_count {
        let t = i as f64 / (strand_count - 1) as f64;
        let bristle_len = 0.06 + rnd() * 0.02;
        let bristle = mesh(cyl_geo!(0.001, 0.0008, bristle_len, 3), &bristle_mat);
        // Position along head-base + slight angular spread.
        bristle.position().set(broom_len * 0.5 - 0.02 + (t - 0.5) * 0.12, 0.010 + bristle_len * 0.35, (rnd() - 0.5) * 0.03);
        bristle.rotation().x = (rnd() - 0.5) * 0.15;
        bristle.rotation().z = 0.15 + rnd() * 0.15;
        bd.add(&bristle);
    }
    // Small metal grip band where head meets handle.
    let grip_band = mesh(pg("BroomsAndDustpans__band", torus_geo!(0.012, 0.002, 3, 8)), lambert!({ color: 0xa0a0a8 }));
    grip_band.rotation().y = PI / 2.0;
    grip_band.position().set(broom_len * 0.42, 0.010, 0);
    bd.add(&grip_band);
    // Dustpan — lying near broom head end.
    let pan = group();
    // Base (flat rectangle).
    let pan_base = mesh(pg("BroomsAndDustpans__panFloor", box_geo!(0.16, 0.006, 0.11)), &pan_mat);
    pan_base.position().y = 0.003;
    pan.add(&pan_base);
    // Raised back edge (front is open for sweeping into it).
    let pan_back = mesh(pg("BroomsAndDustpans__panBack", box_geo!(0.16, 0.030, 0.008)), &pan_mat);
    pan_back.position().set(0, 0.018, -0.051);
    pan.add(&pan_back);
    // Side walls (short low boxes).
    for side in [-1.0, 1.0] {
        let wall = mesh(pg("BroomsAndDustpans__panSide", box_geo!(0.006, 0.024, 0.11)), &pan_mat);
        wall.position().set(side * 0.077, 0.015, 0);
        wall.rotation().z = 0.0;
        pan.add(&wall);
    }
    // Angled front-lip flap: the modelled lip is already a wedge, flat on the
    // floor, thinned so its back meets the pan floor top.
    let lip = mesh(pg("BroomsAndDustpans__panLip", box_geo!(0.14, 0.014, 0.03)), &pan_mat);
    lip.scale().y = 0.45;
    lip.position().set(0, 0.0032, 0.062);
    pan.add(&lip);
    // Handle from back edge angling up: its foot is in the back wall's boss,
    // the hook loop hangs off its top end.
    let pan_handle_len = 0.10;
    let pan_handle_dir = v3(0, 0.5f64.dcos(), -0.5f64.dsin());
    let pan_handle_foot = v3(0, 0.020, -0.053);
    let pan_handle = mesh(pg("BroomsAndDustpans__panHandle", cyl_geo!(0.006, 0.006, pan_handle_len, 5)), &pan_mat);
    pan_handle.rotation().x = -0.5;
    pan_handle.position().copy(&pan_handle_foot).add_scaled_vector(&pan_handle_dir, pan_handle_len / 2.0);
    pan.add(&pan_handle);
    // Hook loop at handle end.
    let hook_loop = mesh(pg("BroomsAndDustpans__hookLoop", torus_geo!(0.006, 0.001, 3, 6)), &pan_mat);
    hook_loop.rotation().x = -0.5;
    hook_loop.position().copy(&pan_handle_foot).add_scaled_vector(&pan_handle_dir, pan_handle_len + 0.005);
    pan.add(&hook_loop);
    // 65% chance: dust pile inside pan.
    if rnd() < 0.65 {
        let dust_mat = lambert!({ color: 0x605040 });
        // Small elongated pile in the middle of the pan.
        let dust = mesh(pg("BroomsAndDustpans__dust", sphere_geo!(0.020, 6, 4)), &dust_mat);
        dust.scale().set(1.4, 0.4, 1.0);
        dust.position().set(-0.010 + (rnd() - 0.5) * 0.020, 0.011, 0);
        pan.add(&dust);
        // A couple tiny dust chunks nearby.
        for _i in 0..3 {
            let chunk = mesh(pg("BroomsAndDustpans__crumb", sphere_geo!(0.004, 4, 3)), &dust_mat);
            chunk.position().set(-0.02 + rnd() * 0.06, 0.008, -0.02 + rnd() * 0.04);
            pan.add(&chunk);
        }
    }
    pan.position().set(broom_len * 0.20, 0, 0.14);
    pan.rotation().y = -0.4;
    bd.add(&pan);
    bd.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    bd.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bd);
}
