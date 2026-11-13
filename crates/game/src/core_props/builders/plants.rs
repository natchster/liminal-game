//! The plants: the potted plants of a planted zone and its office
//! Dracaena (batch `r05`), its leaves, planter and soil painted.
use super::*;

/// A curved leaf (a flat shape geometry
/// pushed out in z by a sine of its height) with its pivot at the base.
pub fn make_leaf_shape(len: f64, wid: f64, color: u32, curl: f64) -> Obj {
    // 8-point ellipse-ish leaf shape
    let mut shape = Shape::new();
    shape.move_to(0, 0);
    shape.bezier_curve_to(wid, len * 0.25, wid * 0.6, len * 0.75, 0, len);
    shape.bezier_curve_to(-wid * 0.6, len * 0.75, -wid, len * 0.25, 0, 0);
    let geo = shape_geo!(shape, 12);
    // add gentle curl by pushing z per y
    {
        let mut gg = geo.g();
        let n = gg.vertex_count();
        for i in 0..n {
            let y = gg.position[i * 3 + 1] as f64;
            gg.position[i * 3 + 2] = (((y / len) * PI).sin() * curl * len) as f32;
        }
        gg.touch();
    }
    geo.compute_vertex_normals();
    mesh(geo, lambert!({ color: color, side: DoubleSide }))
}

/// Pot + soil + fronds
pub fn build_potted_plant(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    let pp = group();
    // terracotta pot with rim + drainage saucer
    let saucer_mat = lambert!({ color: 0x6a3a22 });
    let saucer = mesh(pg("Props__pottedPlant_saucer", cyl_geo!(0.19, 0.17, 0.02, 14)), &saucer_mat);
    saucer.position().y = 0.01;
    pp.add(&saucer);
    let pot_mat = lambert!({ color: 0x8a4a2a });
    let rim_mat = lambert!({ color: 0x9a5230 });
    let pot = mesh(pg("Props__pottedPlant_pot", cyl_geo!(0.16, 0.12, 0.28, 14)), &pot_mat);
    pot.position().y = 0.16;
    pp.add(&pot);
    // vertical seam lines on pot (hand-thrown look)
    for s in 0..6 {
        let seam = mesh(box_geo!(0.005, 0.24, 0.005), lambert!({ color: 0x5a2f18 }));
        let a = s as f64 * PI / 3.0;
        // on the tapered wall (r 0.12 at the foot .. 0.16 at the top): radius at
        // mid-height, leaning out with the wall; vertical at r 0.155 it stood 3 cm off
        seam.set_rotation_order(EulerOrder::YXZ);
        seam.rotation().set(0, -a, -0.142);
        seam.position().set(a.cos() * 0.1415, 0.16, a.sin() * 0.1415);
        pp.add(&seam);
    }
    let rim = mesh(pg("Props__pottedPlant_rim", cyl_geo!(0.17, 0.17, 0.03, 14)), &rim_mat);
    rim.position().y = 0.30;
    pp.add(&rim);
    // soil + tiny pebbles
    let soil = mesh(pg("Props__pottedPlant_soil", cyl_geo!(0.16, 0.16, 0.02, 12)), lambert!({ color: 0x2a1a10 }));
    soil.position().y = 0.315;
    pp.add(&soil);
    for _i in 0..5 {
        let pebble = mesh(pg("Props__pottedPlant_pebble", sphere_geo!(rand2(0.008, 0.014), 5, 4)), lambert!({ color: 0x5a4030 }));
        let x = rand2(-0.1, 0.1);
        let z = rand2(-0.1, 0.1);
        pebble.position().set(x, 0.33, z);
        pp.add(&pebble);
    }
    // fronds — mix of long thin blades + broader curled leaves
    let leaf_colors = [0x2a5a2au32, 0x3a6a30, 0x25502a, 0x487a3a, 0x1a3a1a];
    let blade_count = irand(10, 16);
    for i in 0..blade_count {
        let a = (i as f64 / blade_count as f64) * PI * 2.0 + rand2(-0.15, 0.15);
        let height = rand2(0.28, 0.55);
        let blade = mesh(pg_own("Props__pottedPlant_blade", box_geo!(0.022, height, 0.005)), lambert!({ color: choice(&leaf_colors) }));
        let x = a.cos() * rand2(0.02, 0.09);
        let z = a.sin() * rand2(0.02, 0.09);
        blade.position().set(x, 0.32 + height / 2.0 - 0.02, z);
        // splay outward from center
        let rx = a.sin() * rand2(0.15, 0.4);
        let rz = -a.cos() * rand2(0.15, 0.4);
        blade.rotation().set(rx, a, rz);
        // taper by scaling top thinner
        blade.geo().translate(0, 0, 0);
        pp.add(&blade);
    }
    // curled broad leaves scattered on top for variety
    let broad_count = irand(3, 5);
    for _i in 0..broad_count {
        let a = rand2(0.0, PI * 2.0);
        let (len, wid, color, curl) = (rand2(0.12, 0.20), rand2(0.05, 0.08), choice(&leaf_colors), rand2(0.12, 0.25));
        let leaf = make_leaf_shape(len, wid, color, curl);
        let x = a.cos() * rand2(0.02, 0.08);
        let z = a.sin() * rand2(0.02, 0.08);
        leaf.position().set(x, 0.33, z);
        let rx = rand2(0.2, 0.6);
        let rz = rand2(-0.3, 0.3);
        leaf.rotation().set(rx, a, rz);
        pp.add(&leaf);
    }
    // dead brown leaves on saucer and floor
    // the bound `irand(2, 4)` is deliberately redrawn every pass
    let mut i = 0i64;
    while i < irand(2, 4) {
        let dead = mesh(pg("Props__plant_deadLeaf", box_geo!(rand2(0.04, 0.06), 0.002, rand2(0.03, 0.05))), lambert!({ color: 0x5a3a20 }));
        let x = rand2(-0.22, 0.22);
        let z = rand2(-0.22, 0.22);
        dead.position().set(x, 0.006, z);
        dead.rotation().y = rand2(0.0, 7.0);
        dead.rotation().x = rand2(-0.15, 0.15);
        pp.add(&dead);
        i += 1;
    }
    // occasional small yellow flower
    if rnd() < 0.25 {
        let f_center = mesh(pg("Props__pottedPlant_flowerCenter", sphere_geo!(0.015, 6, 5)), lambert!({ color: 0xf0c040 }));
        let (x, y, z) = (rand2(-0.05, 0.05), rand2(0.5, 0.7), rand2(-0.05, 0.05));
        f_center.position().set(x, y, z);
        pp.add(&f_center);
        let fc = f_center.pos();
        for p in 0..5 {
            let petal = mesh(pg("Props__pottedPlant_petal", box_geo!(0.02, 0.008, 0.03)), lambert!({ color: 0xf8e0a0 }));
            let pa = (p as f64 / 5.0) * PI * 2.0;
            petal.position().set(fc.x + pa.cos() * 0.02, fc.y, fc.z + pa.sin() * 0.02);
            petal.rotation().y = PI / 2.0 - pa; // long (z) axis radial, like its offset
            pp.add(&petal);
        }
    }
    Built::new(pp, 0.7)
}

/// Large floor plant: a neglected office dracaena (Blender batch `r05`)
pub fn build_floor_plant(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    // A Dracaena fragrans on three staggered canes in a 16 in moulded plastic
    // planter on its saucer, left for years: the canes' shoots grown leggy,
    // the old leaves yellowing and hanging, a skirt of dead ones under each
    // crown, brown dry tips, the soil shrunk from the pot, leaves dropped
    // round it (docs/design/PROPS.md). The parts as modelled, the pot centred.
    use crate::assets::props::prop_part_as_modelled;
    let fp = group();
    // the plant turns about its pot (the old first leaf's spin)
    let plant = group();
    fp.add(&plant);
    let put = |name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            plant.add(&mesh(geo, mat));
        }
    };
    // the leaves' dust and blotches laid on at their real scale (a repeat every
    // 0.2 m, from each leaf's own facing), not stretched over the part's box
    let put_leaf = |name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            plant.add(&mesh(drac_leaf_uv(&geo), mat));
        }
    };
    // the old moss patches' draws (four: a size and a place each)
    for _m in 0..4 {
        let _ = (rand2(0.04, 0.08), rand2(-0.15, 0.15), rand2(-0.15, 0.15));
    }
    // the old eight broad leaves' draws: the first one's spin turns the plant
    let big_leaf_colors = [0x2a5a28u32, 0x357a30, 0x3a6a30, 0x225022];
    let mut spin = 0.0;
    for i in 0..8 {
        let _ = (rand2(0.22, 0.32), rand2(0.10, 0.14), choice(&big_leaf_colors), rand2(0.18, 0.32));
        let _ = (rand2(-0.05, 0.05), rand2(-0.05, 0.05));
        let (_, b, _) = (rand2(-0.4, -0.1), rand2(0.0, PI * 2.0), rand2(-0.3, 0.3));
        if i == 0 {
            spin = b;
        }
    }
    plant.rotation().y = spin;
    // the old two yellowing leaves' draws
    for _i in 0..2 {
        let _ = (rand2(0.16, 0.22), rand2(0.08, 0.11), rand2(0.25, 0.45));
        let _ = (rand2(-0.12, 0.12), rand2(0.6, 1.2), rand2(-0.12, 0.12));
        let _ = (rand2(-1.2, -0.8), rand2(0.0, PI * 2.0), rand2(-0.3, 0.3));
    }
    // the planter: charcoal moulded plastic gone grey with dust, the white
    // crust of hard water where the saucer stood full and where it was
    // over-watered down its side; the saucer black plastic, dry
    put("Real__dracSaucer", &phong!({ color: 0x2a2826, specular: 0x1a1a1a, shininess: 36 }));
    put("Real__dracPot", &phong!({ map: &drac_pot_tex(), specular: 0x1a1a1a, shininess: 24 }));
    put("Real__dracSoil", &lambert!({ map: &drac_soil_tex() }));
    put("Real__dracCanes", &lambert!({ color: 0x5f5546 }));
    put("Real__dracWax", &lambert!({ color: 0x5c2b21 }));
    put("Real__dracShoots", &lambert!({ color: 0x5b5338 }));
    // the leaves: a dark green filmed with dust, their tips dry, the oldest
    // yellow, the dead papery; each blotched as it went
    let tips = lambert!({ map: &drac_leaf_tex([116, 88, 58], 0.2), side: DoubleSide });
    let dead = lambert!({ map: &drac_leaf_tex([112, 88, 62], 0.15), side: DoubleSide });
    put_leaf("Real__dracLeaves", &phong!({ map: &drac_leaf_tex([51, 69, 43], 0.35), specular: 0x14180f, shininess: 16, side: DoubleSide }));
    put_leaf("Real__dracTips", &tips);
    put_leaf("Real__dracYellow", &lambert!({ map: &drac_leaf_tex([126, 116, 70], 0.25), side: DoubleSide }));
    put_leaf("Real__dracYellowTips", &tips);
    put_leaf("Real__dracDead", &dead);
    put_leaf("Real__dracLitter", &dead);
    // dropped leaves on the floor round it, curled and dry (out past the saucer)
    let fallen = prop_part_as_modelled("Real__dracFallen");
    let browns = [0x8a7150u32, 0x9a8458, 0x7a6244];
    let fallen_tex = drac_leaf_tex([255, 255, 255], 0.0);
    for _i in 0..4 {
        let _ = (rand2(0.08, 0.14), rand2(0.05, 0.08));
        let shade = browns[irand(0, 2) as usize];
        let x = rand2(-0.35, 0.35);
        let z = rand2(-0.35, 0.35);
        let turn = rand2(0.0, 7.0);
        let _ = rand2(-0.2, 0.2);
        if let Some(geo) = &fallen {
            let r = (x * x + z * z).sqrt().max(1e-6);
            let out = 0.25 + 0.42 * (r / 0.495).min(1.0);
            let leaf = mesh(drac_leaf_uv(geo), lambert!({ color: shade, map: &fallen_tex, side: DoubleSide }));
            leaf.position().set(x / r * out, 0.0, z / r * out);
            leaf.rotation().y = turn;
            fp.add(&leaf);
        }
    }
    // now and then spider mites' webbing between the top crown's leaves
    if rnd() < 0.4 {
        let _ = (rand2(-0.5, 0.5), rand2(0.0, 7.0), rand2(-0.5, 0.5));
        put("Real__dracWeb", &lambert!({ color: 0xb4b0a6, transparent: true, opacity: 0.45, depthWrite: false }));
    }
    Built::new(fp, 1.9)
}

/// A leaf part with its map laid on at the leaf's own scale: each point's
/// texture place from the two axes across its facing, a repeat every 0.2 m
fn drac_leaf_uv(geo: &Geo) -> Geo {
    let g = geo.duplicate();
    {
        let mut gg = g.g();
        let n = gg.position.len() / 3;
        let mut uv = Vec::with_capacity(n * 2);
        for i in 0..n {
            let (x, y, z) = (gg.position[i * 3], gg.position[i * 3 + 1], gg.position[i * 3 + 2]);
            let (nx, ny, nz) = (gg.normal[i * 3].abs(), gg.normal[i * 3 + 1].abs(), gg.normal[i * 3 + 2].abs());
            let (u, v) = if ny >= nx && ny >= nz {
                (x, z)
            } else if nx >= nz {
                (z, y)
            } else {
                (x, y)
            };
            uv.extend_from_slice(&[u / 0.2, v / 0.2]);
        }
        gg.uv = uv;
        gg.touch();
    }
    g
}

/// A dracaena leaf's surface, repeating: its colour (`base`, sRGB) a little
/// uneven, darker and lighter streaks, brown blotches where it was hurt, and
/// dust (`dust`: how much) settled on it in a grey film and specks
fn drac_leaf_tex(base: [u8; 3], dust: f64) -> Tex {
    canvas_tex(256, 256, move |ctx, w, h| {
        let [r, g, b] = base;
        ctx.set_fill_style(format!("rgb({r},{g},{b})"));
        ctx.fill_rect(0, 0, w, h);
        let shade = |k: f64| -> String {
            let f = |c: u8| ((c as f64 * k).round() as i64).clamp(0, 255);
            format!("{},{},{}", f(r), f(g), f(b))
        };
        // uneven colour: soft patches lighter and darker
        for _ in 0..40 {
            let (x, y, rr) = (rnd() * w, rnd() * h, 10.0 + rnd() * 40.0);
            let k = 0.82 + 0.36 * rnd();
            let stops = [(0.0, format!("rgba({},0.5)", shade(k))), (1.0, format!("rgba({},0)", shade(k)))];
            mirror_blob(ctx, x, y, rr, 0.3 + 0.7 * rnd(), rnd() * 3.0, &stops);
        }
        // fine streaks, the veins running the leaf's length
        ctx.set_line_width(0.8);
        for i in 0..60 {
            let x = (i as f64 + rnd()) / 60.0 * w;
            ctx.set_stroke_style(format!("rgba({},0.35)", shade(if i % 2 == 0 { 0.86 } else { 1.12 })));
            ctx.begin_path();
            ctx.move_to(x, 0);
            ctx.line_to(x + rnd() * 4.0 - 2.0, h);
            ctx.stroke();
        }
        // brown blotches and pin spots where it was hurt or dried
        for _ in 0..7 {
            let (x, y, rr) = (rnd() * w, rnd() * h, 3.0 + rnd() * 9.0);
            let stops = [(0.0, "rgba(92,66,40,0.75)".to_string()), (0.6, "rgba(104,78,48,0.5)".into()), (1.0, "rgba(104,78,48,0)".into())];
            mirror_blob(ctx, x, y, rr, 0.5 + 0.5 * rnd(), rnd() * 3.0, &stops);
        }
        // dust: a grey film, thicker in drifts, and specks
        if dust > 0.0 {
            for _ in 0..24 {
                let (x, y, rr) = (rnd() * w, rnd() * h, 16.0 + rnd() * 50.0);
                let a = dust * (0.4 + 0.6 * rnd());
                let stops = [(0.0, format!("rgba(150,146,134,{a:.3})")), (1.0, "rgba(150,146,134,0)".into())];
                mirror_blob(ctx, x, y, rr, 0.5 + 0.5 * rnd(), rnd() * 3.0, &stops);
            }
            for _ in 0..(900.0 * dust) as usize {
                let (x, y) = (rnd() * w, rnd() * h);
                ctx.set_fill_style(format!("rgba(170,166,154,{:.2})", 0.3 + 0.5 * rnd()));
                ctx.begin_path();
                ctx.arc(x, y, 0.4 + rnd() * rnd() * 1.4, 0, 7);
                ctx.fill();
            }
        }
    })
}

/// The planter's plastic, by height (the part's box: v 0 its foot, 1 its
/// rim): charcoal gone grey with dust, more of it up by the rim; the white
/// crust of hard water in a band at its foot where the saucer stood full,
/// runs of it down from the rim where it was over-watered; scuffs
fn drac_pot_tex() -> Tex {
    canvas_tex(256, 256, |ctx, w, h| {
        ctx.set_fill_style("rgb(44,43,41)");
        ctx.fill_rect(0, 0, w, h);
        let gr = ctx.create_linear_gradient(0, 0, 0, h);
        gr.add_color_stop(0, "rgba(140,136,126,0.38)");
        gr.add_color_stop(0.3, "rgba(140,136,126,0.1)");
        gr.add_color_stop(1, "rgba(140,136,126,0.03)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(0, 0, w, h);
        // the dust unevenly, in soft patches
        for _ in 0..14 {
            let (x, y, rr) = (rnd() * w, h * rnd(), 12.0 + rnd() * 40.0);
            let stops = [(0.0, format!("rgba(130,126,116,{:.3})", 0.08 + 0.12 * rnd())), (1.0, "rgba(130,126,116,0)".into())];
            mirror_blob(ctx, x, y, rr, 0.4 + 0.6 * rnd(), rnd() * 3.0, &stops);
        }
        // the crust where the saucer's water stood: a band at the foot, its top edge wandering
        let top = h * 0.93;
        ctx.set_fill_style("rgba(196,192,180,0.55)");
        ctx.begin_path();
        ctx.move_to(0, h);
        let ph = rnd() * 6.3;
        for k in 0..=64 {
            let x = k as f64 / 64.0 * w;
            let y = top - 3.0 * (x / w * 6.28 * 3.0 + ph).sin() - rnd() * 3.0;
            ctx.line_to(x, y);
        }
        ctx.line_to(w, h);
        ctx.close_path();
        ctx.fill();
        // its tide line, whiter
        ctx.set_stroke_style("rgba(214,210,198,0.7)");
        ctx.set_line_width(1.5);
        ctx.begin_path();
        for k in 0..=64 {
            let x = k as f64 / 64.0 * w;
            let y = top - 3.0 * (x / w * 6.28 * 3.0 + ph).sin() - rnd() * 1.5;
            if k == 0 {
                ctx.move_to(x, y);
            } else {
                ctx.line_to(x, y);
            }
        }
        ctx.stroke();
        // runs down from the rim
        ctx.set_line_cap("round");
        for _ in 0..9 {
            let x = rnd() * w;
            let len = h * (0.2 + 0.55 * rnd());
            let y0 = h * (0.04 + 0.04 * rnd());
            ctx.set_stroke_style(format!("rgba(200,196,184,{:.2})", 0.2 + 0.3 * rnd()));
            ctx.set_line_width(1.5 + rnd() * 3.5);
            ctx.begin_path();
            ctx.move_to(x, y0);
            ctx.quadratic_curve_to(x + rnd() * 6.0 - 3.0, y0 + len * 0.5, x + rnd() * 8.0 - 4.0, y0 + len);
            ctx.stroke();
        }
        // scuffs, lighter
        ctx.set_line_width(0.8);
        for _ in 0..40 {
            let (x, y) = (rnd() * w, h * (0.3 + 0.7 * rnd()));
            let a = rnd() * 0.6 - 0.3;
            let l = 4.0 + rnd() * 16.0;
            ctx.set_stroke_style(format!("rgba(120,118,112,{:.2})", 0.3 + 0.4 * rnd()));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        // dust specks
        for _ in 0..500 {
            let (x, y) = (rnd() * w, h * rnd().powf(1.6));
            ctx.set_fill_style(format!("rgba(160,156,146,{:.2})", 0.25 + 0.4 * rnd()));
            ctx.begin_path();
            ctx.arc(x, y, 0.4 + rnd() * 0.9, 0, 7);
            ctx.fill();
        }
    })
}

/// The potting soil seen from above, dried out: dark peat, crumbs lighter
/// and darker, the white grains of perlite, a few bits of dead leaf
fn drac_soil_tex() -> Tex {
    canvas_tex(256, 256, |ctx, w, h| {
        ctx.set_fill_style("rgb(62,48,36)");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..900 {
            let (x, y) = (rnd() * w, rnd() * h);
            let v = if rnd() < 0.5 { (42, 32, 24) } else { (96, 78, 58) };
            ctx.set_fill_style(format!("rgba({},{},{},{:.2})", v.0, v.1, v.2, 0.4 + 0.5 * rnd()));
            ctx.begin_path();
            ctx.arc(x, y, 0.6 + rnd() * rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        // perlite
        for _ in 0..140 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_fill_style(format!("rgba(206,202,192,{:.2})", 0.7 + 0.3 * rnd()));
            ctx.begin_path();
            ctx.arc(x, y, 0.8 + rnd() * 1.6, 0, 7);
            ctx.fill();
        }
        // bits of dead leaf
        ctx.set_line_cap("round");
        for _ in 0..10 {
            let (x, y, a, l) = (rnd() * w, rnd() * h, rnd() * 6.3, 6.0 + rnd() * 18.0);
            ctx.set_stroke_style("rgba(128,100,66,0.8)");
            ctx.set_line_width(2.0 + rnd() * 3.0);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
}
