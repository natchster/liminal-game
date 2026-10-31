//! `place_props(dist)`, first half: the candidate tiles, the exit door, the
//! pickups, chairs, TV, clock, balloon, payphone, mannequin, every prop
//! builder, `scatter` and the scatter calls. The second half is
//! `place_props_tail` in `core_props/tail.rs`.
//!
//! The state both halves share lives in `PropsCtx`; the builders are free
//! functions in `core_props/builders.rs`, re-exported here.
pub mod builders;
pub mod seam;
pub mod tail;
pub mod vending;
pub mod weather_fx;
pub mod window;

use crate::base::records::*;
use crate::prelude::*;
use crate::visuals::textures::{handwritten_tex, text_tex, HandOpts, TextTexOpts};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub use crate::core_props::builders::*;
pub use crate::core_props::vending::build_vending;

/// a candidate tile (x, z) and `d`, its distance from spawn
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cand {
    pub x: i32,
    pub z: i32,
    pub d: i32,
}

/// what a builder returns: the group and its top height (`drawers` only for the filing cabinet)
#[derive(Clone, Debug)]
pub struct Built {
    pub g: Obj,
    pub top: f64,
    pub drawers: Vec<Rc<RefCell<Drawer>>>,
}
impl Built {
    pub fn new(g: Obj, top: f64) -> Built {
        Built { g, top, drawers: vec![] }
    }
}

/// `place_props` state shared by both halves
#[derive(Debug, Default)]
pub struct PropsCtx {
    /// (shuffled)
    pub cands: Vec<Cand>,
    pub taken: Vec<Cand>,
    pub exit_tile: Option<Cand>,
}
impl PropsCtx {
    /// The first candidate (in shuffled order) that passes the
    /// filter and is at least 4 tiles (Manhattan) from every taken tile. It is
    /// removed from `cands` and pushed to `taken`.
    pub fn take(&mut self, filter: Option<&dyn Fn(&Cand) -> bool>) -> Option<Cand> {
        for i in 0..self.cands.len() {
            let c = self.cands[i];
            if let Some(f) = filter {
                if !f(&c) {
                    continue;
                }
            }
            if self.taken.iter().any(|t| (t.x - c.x).abs() + (t.z - c.z).abs() < 4) {
                continue;
            }
            self.cands.remove(i);
            self.taken.push(c);
            return Some(c);
        }
        None
    }
}
pub fn has_wall_n(g: &Game, c: &Cand) -> bool {
    [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dz)| g.is_wall_tile(c.x + dx, c.z + dz))
}

/// A builder: `(g, cx) -> Built`; builders take no other arguments.
pub type Builder = fn(&mut Game, &mut PropsCtx) -> Built;

// ---------------------------------------------------------------- test seam
/// Stand-ins for the functions of other modules that `place_props` calls
/// (`wall_adj_spot`, `level_has_stairwell`, `circle_hits_wall`). `None` (the
/// default) calls the real `Game` method; tests install overrides here that
/// answer as the recorded reference data does, so they do not depend on the
/// state of those modules.
#[derive(Clone, Copy)]
pub struct PropsHooks {
    pub wall_adj_spot: Option<fn(&Game, i32, i32) -> Option<WallSpot>>,
    pub level_has_stairwell: Option<fn(&Game, i32) -> bool>,
    pub circle_hits_wall: Option<fn(&Game, f64, f64, f64) -> bool>,
}
impl PropsHooks {
    pub const NONE: PropsHooks = PropsHooks { wall_adj_spot: None, level_has_stairwell: None, circle_hits_wall: None };
}
impl Default for PropsHooks {
    fn default() -> Self {
        PropsHooks::NONE
    }
}
thread_local! { static HOOKS: std::cell::Cell<PropsHooks> = const { std::cell::Cell::new(PropsHooks::NONE) }; }
/// Install (or with `PropsHooks::NONE`, clear) the test stand-ins.
pub fn set_props_hooks(h: PropsHooks) {
    HOOKS.with(|c| c.set(h));
}
fn hooks() -> PropsHooks {
    HOOKS.with(|c| c.get())
}
/// `wall_adj_spot(x, z)` as `place_props` calls it (one `choice` draw)
pub fn props_wall_adj_spot(g: &Game, x: i32, z: i32) -> Option<WallSpot> {
    match hooks().wall_adj_spot {
        Some(f) => f(g, x, z),
        None => g.wall_adj_spot(x, z),
    }
}
fn level_has_stairwell(g: &Game, n: i32) -> bool {
    match hooks().level_has_stairwell {
        Some(f) => f(g, n),
        None => g.level_has_stairwell(n),
    }
}
fn circle_hits_wall(g: &Game, px: f64, pz: f64, r: f64) -> bool {
    match hooks().circle_hits_wall {
        Some(f) => f(g, px, pz, r),
        None => g.circle_hits_wall(px, pz, r),
    }
}

/// where `scatter` put a prop (position, yaw, wall normal), handed to `on_placed`
#[derive(Clone, Copy, Debug)]
pub struct SpotInfo {
    pub px: f64,
    pub pz: f64,
    pub yaw: f64,
    pub normal: Option<Vec3>,
}

/// Where a kind of core prop belongs: zones of one of these archetypes, or
/// whose purpose has one of these themes (and with `skip_bare`, never a
/// zone left bare, the `empty` purpose). On a level with no such zone it
/// goes anywhere with chance `elsewhere`, and otherwise not at all.
#[derive(Clone, Copy, Debug)]
pub struct Home {
    pub arch: &'static [&'static str],
    pub themes: &'static [&'static str],
    pub elsewhere: f64,
    pub skip_bare: bool,
}

// where the core props belong
const OFFICE: Home = Home { arch: &["office", "lobby"], themes: &["office"], elsewhere: 1.0, skip_bare: false };
const LOCKER_ROOM: Home =
    Home { arch: &["office", "hall", "pool"], themes: &["office", "bathroom", "industrial"], elsewhere: 1.0, skip_bare: false };
const VENDING: Home = Home { arch: &["office", "hall", "pool"], themes: &["office", "party"], elsewhere: 0.0, skip_bare: true };
const STORAGE: Home = Home { arch: &["tunnels", "electrical", "hall"], themes: &["industrial"], elsewhere: 1.0, skip_bare: false };
const WET: Home = Home { arch: &["pool"], themes: &["bathroom"], elsewhere: 1.0, skip_bare: false };
const SERVICE: Home =
    Home { arch: &["hall", "tunnels", "electrical", "office"], themes: &["industrial", "office"], elsewhere: 1.0, skip_bare: false };
const PUBLIC: Home =
    Home { arch: &["lobby", "office", "hall", "pool"], themes: &["office", "party", "domestic"], elsewhere: 1.0, skip_bare: false };
const BATH_TOYS: Home = Home { arch: &["pool"], themes: &["bathroom", "toys"], elsewhere: 0.0, skip_bare: false };
const WORKS: Home = Home { arch: &["tunnels", "electrical", "hall"], themes: &["industrial"], elsewhere: 0.5, skip_bare: false };
const HOME_CARE: Home = Home { arch: &[], themes: &["domestic", "victorian"], elsewhere: 0.25, skip_bare: false };
const NURSERY: Home = Home { arch: &[], themes: &["toys", "domestic", "victorian"], elsewhere: 0.25, skip_bare: false };
const LUGGAGE: Home = Home { arch: &["lobby", "hall"], themes: &["domestic", "victorian"], elsewhere: 1.0, skip_bare: false };
const READING: Home = Home { arch: &["office"], themes: &["office", "victorian", "domestic"], elsewhere: 1.0, skip_bare: false };
const CLOAKROOM: Home = Home { arch: &["office", "lobby"], themes: &["office", "domestic", "victorian"], elsewhere: 1.0, skip_bare: false };
const LIVING: Home = Home { arch: &[], themes: &["domestic", "office"], elsewhere: 1.0, skip_bare: false };
const GREENERY: Home = Home { arch: &["lobby", "office"], themes: &["nature"], elsewhere: 1.0, skip_bare: false };
const SEATING: Home = Home { arch: &["office", "lobby"], themes: &["office", "domestic", "party"], elsewhere: 1.0, skip_bare: false };
const CLOCK_WALL: Home =
    Home { arch: &["office", "lobby", "hall"], themes: &["office", "domestic", "victorian"], elsewhere: 1.0, skip_bare: false };

/// Per zone, whether it is `home` to a prop. None without a layout or zone
/// purposes (a synthetic level): the prop goes anywhere.
pub fn home_zones(g: &Game, home: &Home) -> Option<Vec<bool>> {
    let l = g.layout.as_ref()?;
    if g.zone_purposes.len() != l.zones.len() {
        return None;
    }
    Some(
        l.zones
            .iter()
            .zip(&g.zone_purposes)
            .map(|(z, p)| (home.arch.contains(&z.arch) || p.has_any(home.themes)) && !(home.skip_bare && p.primary.is_none()))
            .collect(),
    )
}

/// The zones a prop of `home` may use on this level: None for anywhere, an
/// empty list when it is left out (one draw when the level has no home for it)
fn home_for(g: &Game, cx: &PropsCtx, home: Option<&Home>) -> Option<Vec<bool>> {
    let h = home?;
    let zones = home_zones(g, h)?;
    let l = g.layout.as_ref()?;
    if cx.cands.iter().any(|c| zones[l.zone_of[g.t_idx(c.x, c.z)] as usize]) {
        return Some(zones);
    }
    if rnd() < h.elsewhere {
        None
    } else {
        Some(vec![])
    }
}

/// `take` restricted to the `home` zones (and to tiles beside a wall with `wall`)
fn take_home(g: &Game, cx: &mut PropsCtx, home: Option<&[bool]>, wall: bool) -> Option<Cand> {
    match home {
        None if !wall => cx.take(None),
        None => cx.take(Some(&|c: &Cand| has_wall_n(g, c))),
        Some([]) => None,
        Some(zones) => {
            let l = g.layout.as_ref()?;
            cx.take(Some(&|c: &Cand| (!wall || has_wall_n(g, c)) && zones[l.zone_of[g.t_idx(c.x, c.z)] as usize]))
        }
    }
}

/// `scatter` options
#[derive(Default)]
struct ScatterOpts<'a> {
    wall: bool,
    /// distance out from the wall face (None → 0.32)
    wall_offset: Option<f64>,
    can_stack: bool,
    /// height above the floor (default 0)
    y: f64,
    on_placed: Option<&'a dyn Fn(&mut Game, &Built, &SpotInfo)>,
    /// where it belongs (None: anywhere)
    home: Option<Home>,
    /// up to this many stand side by side, `spacing` metres apart: a row
    /// against the wall, or a line on the floor (only where it is at home)
    row: i32,
    spacing: f64,
}

/// The row `scatter` is laying: its tile, how many more go in it, and where
/// the first one stands (x, z, yaw, wall normal)
struct Row {
    c: Cand,
    left: i32,
    k: i32,
    first: (f64, f64, f64, Option<Vec3>),
}

/// Place `count` props on free tiles
/// (against a wall with `wall`), sometimes with a small prop stacked on top.
fn scatter(g: &mut Game, cx: &mut PropsCtx, builder: Builder, count: i32, opts: &ScatterOpts) {
    let home = home_for(g, cx, opts.home.as_ref());
    let mut row: Option<Row> = None;
    for _i in 0..count {
        let in_row = row.as_ref().map(|r| r.left > 0).unwrap_or(false);
        let c = if in_row { row.as_ref().map(|r| r.c) } else { take_home(g, cx, home.as_deref(), opts.wall) };
        let Some(c) = c else { return };
        let built = builder(g, cx);
        let bg = built.g.clone();
        let mut px = g.wx(c.x) + rand2(-1.0, 1.0);
        let mut pz = g.wz(c.z) + rand2(-1.0, 1.0);
        let mut yaw = rand1(PI * 2.0);
        let mut normal: Option<Vec3> = None;
        if let Some(r) = row.as_mut().filter(|_| in_row) {
            // the next one in the row: beside the first along its wall, or on along its line
            r.left -= 1;
            r.k += 1;
            let (fx, fz, fyaw, fnormal) = r.first;
            let step = opts.spacing * ((r.k + 1) / 2) as f64 * if r.k % 2 == 1 { 1.0 } else { -1.0 };
            let (tx, tz) = match fnormal {
                Some(nv) => (-nv.z, nv.x),
                None => (fyaw.cos(), -fyaw.sin()),
            };
            px = fx + tx * step;
            pz = fz + tz * step;
            yaw = if fnormal.is_some() { fyaw } else { fyaw + (yaw - PI) * 0.08 };
            normal = fnormal;
        } else {
            if opts.wall {
                if let Some(spot) = props_wall_adj_spot(g, c.x, c.z) {
                    let off = opts.wall_offset.unwrap_or(0.32);
                    px = spot.pos.x + spot.normal.x * off;
                    pz = spot.pos.z + spot.normal.z * off;
                    yaw = spot.yaw;
                    normal = Some(spot.normal);
                }
            }
            // where it is at home a few may stand together
            let more = if home.is_some() && opts.row > 1 { irand(0, opts.row as i64 - 1) as i32 } else { 0 };
            row = Some(Row { c, left: more, k: 0, first: (px, pz, yaw, normal) });
        }
        bg.position().set(px, opts.y, pz);
        bg.rotation().y = yaw;
        g.world.add(&bg);
        if let Some(f) = opts.on_placed {
            f(g, &built, &SpotInfo { px, pz, yaw, normal });
        }
        // tall props once got a second small prop stacked on top (a duck, a
        // cone, a doll...); it read as a gag and is no longer placed, but its
        // draws stay so the rest of the level is built from the same sequence
        if opts.can_stack && built.top > 0.5 && rnd() < 0.5 {
            let tops: [Builder; 5] = [build_rubber_duck, build_traffic_cone, build_baby_doll, build_book_stack, build_fluorescent_tube];
            let b = choice(&tops);
            let _unplaced = b(g, cx);
            let _ = (rand2(-0.15, 0.15), rand2(-0.15, 0.15), rand1(PI * 2.0));
        }
    }
}

/// One `[E] OPEN DRAWER` per drawer
/// (the drawer group slides out along the cabinet's +z and reveals its
/// content), then registers the cabinet in `g.drawer_props`.
fn register_cabinet_drawers(g: &mut Game, built: &Built, si: &SpotInfo, bat_tip_geo: &Geo, bat_tip_mat: &Mat) {
    // only the filing cabinet (the one builder with drawers) registers, and it always has its three
    let SpotInfo { px, pz, yaw, normal } = *si;
    let fwd_x = match normal {
        Some(n) => n.x,
        None => yaw.sin(),
    };
    let fwd_z = match normal {
        Some(n) => n.z,
        None => yaw.cos(),
    };
    for (idx, d) in built.drawers.iter().enumerate() {
        let drawer_center_y = 0.28 + idx as f64 * 0.45;
        let drawer_floor_y = drawer_center_y - 0.144; // ~top of interior floor plane
        let use_pos = v3(px + fwd_x * 0.6, EYE, pz + fwd_z * 0.6);
        d.borrow_mut().use_pos = use_pos;
        let d_en = d.clone();
        let d_use = d.clone();
        let cabinet = built.g.clone();
        let tip_geo = bat_tip_geo.clone();
        let tip_mat = bat_tip_mat.clone();
        g.interact(
            use_pos,
            1.0,
            "[E] OPEN DRAWER",
            move |_g| !d_en.borrow().opened,
            move |g| {
                d_use.borrow_mut().opened = true;
                // slide drawer out along cabinet's local +z (world = normal direction)
                let slide_out = 0.36;
                let (dmesh, content_type) = {
                    let d = d_use.borrow();
                    (d.mesh.clone(), d.content_type.clone())
                };
                dmesh.position().z += slide_out;
                g.sfx_clank();
                // content sits inside the drawer interior on the drawer floor,
                // roughly at the middle of the visible open area
                let content_local_z = slide_out + 0.10;
                let cp = cabinet.pos();
                let content_x = cp.x + fwd_x * content_local_z;
                let content_z = cp.z + fwd_z * content_local_z;
                if content_type == "battery" {
                    let bg = group();
                    bg.position().set(content_x, drawer_floor_y + 0.06, content_z);
                    let body =
                        mesh(pg("Props__battery_body", cyl_geo!(0.06, 0.06, 0.18, 10)), lambert!({ color: 0xc79b3d, emissive: 0x2a1f08 }));
                    body.rotation().z = PI / 2.0;
                    bg.add(&body);
                    // + terminal on the body's +Y end (at -x once laid down), shared tip geometry
                    let tip = mesh(&tip_geo, &tip_mat);
                    tip.rotation().z = PI / 2.0;
                    tip.position().x = -0.105;
                    bg.add(&tip);
                    g.world.add(&bg);
                    g.battery_items.push(Pickup { g: bg, taken: false, base: drawer_floor_y + 0.06, phase: rand1(7.0) });
                    g.show_msg("A BATTERY INSIDE", 1.6);
                } else if content_type == "almond" {
                    let ag = group();
                    ag.position().set(content_x, drawer_floor_y + 0.15, content_z);
                    let b_mat = lambert!({ color: 0x9fc4d8, emissive: 0x16323e });
                    let b = mesh(pg("Props__almond_bottle", cyl_geo!(0.09, 0.09, 0.3, 10)), &b_mat);
                    ag.add(&b);
                    let cap = mesh(pg("Props__almond_cap", cyl_geo!(0.05, 0.05, 0.06, 8)), &g.mats.metal);
                    cap.position().y = 0.18;
                    ag.add(&cap);
                    g.world.add(&ag);
                    g.almonds.push(Pickup { g: ag, taken: false, base: drawer_floor_y + 0.15, phase: rand1(7.0) });
                    g.show_msg("ALMOND WATER INSIDE", 1.6);
                } else if content_type == "paper" {
                    let mut dummy = PropsCtx::default();
                    let p = build_paper_pile(g, &mut dummy);
                    p.g.position().set(content_x, drawer_floor_y + 0.02, content_z);
                    g.world.add(&p.g);
                    g.papers.push(PaperPile { g: p.g.clone(), taken: false });
                    g.show_msg("A STACK OF PAPER", 1.6);
                } else if let Some(i) = content_type.strip_prefix("secret:").and_then(|i| i.parse::<usize>().ok()) {
                    // something taped under it (`secrets`)
                    g.open_secret(i);
                } else {
                    g.show_msg("EMPTY.", 1.2);
                }
            },
        );
    }
    g.drawer_props.push(DrawerProp { cabinet: built.g.clone(), drawers: built.drawers.clone() });
}

// the exit sign face (ISO 7010 green, running person + arrow + EXIT)
fn exit_sign_tex() -> Tex {
    canvas_tex_opt(512, 192, false, |ctx, w, h| {
        // ISO 7010 green (E002)
        ctx.set_fill_style("#0a8a3c");
        ctx.fill_rect(0, 0, w, h);
        // thin darker border
        ctx.set_stroke_style("rgba(0,0,0,0.35)");
        ctx.set_line_width(4);
        ctx.stroke_rect(2, 2, w - 4.0, h - 4.0);
        // running-person pictogram (left half)
        ctx.set_fill_style("#ffffff");
        let (cx, cy) = (96.0, 96.0);
        // head
        ctx.begin_path();
        ctx.arc(cx - 6.0, cy - 46.0, 14, 0, PI * 2.0);
        ctx.fill();
        // torso (leaning forward)
        ctx.save();
        ctx.translate(cx, cy);
        ctx.rotate(-0.18);
        ctx.fill_rect(-12, -34, 20, 44);
        ctx.restore();
        // front leg (kicked forward)
        ctx.save();
        ctx.translate(cx + 4.0, cy + 4.0);
        ctx.rotate(-0.5);
        ctx.fill_rect(-6, 0, 12, 42);
        ctx.restore();
        // back leg (pushing off)
        ctx.save();
        ctx.translate(cx - 8.0, cy + 10.0);
        ctx.rotate(0.55);
        ctx.fill_rect(-6, 0, 12, 40);
        ctx.restore();
        // front arm swinging forward
        ctx.save();
        ctx.translate(cx + 12.0, cy - 20.0);
        ctx.rotate(-0.9);
        ctx.fill_rect(-5, 0, 10, 36);
        ctx.restore();
        // back arm swinging back
        ctx.save();
        ctx.translate(cx - 18.0, cy - 20.0);
        ctx.rotate(0.8);
        ctx.fill_rect(-5, 0, 10, 36);
        ctx.restore();
        // "door frame" pictogram the person is running toward
        ctx.set_stroke_style("#ffffff");
        ctx.set_line_width(6);
        ctx.stroke_rect(150, 42, 32, 108);
        // arrow pointing right (below the person, ISO 7010-style)
        ctx.begin_path();
        ctx.move_to(56, 168);
        ctx.line_to(148, 168);
        ctx.line_to(148, 158);
        ctx.line_to(178, 174);
        ctx.line_to(148, 190);
        ctx.line_to(148, 180);
        ctx.line_to(56, 180);
        ctx.close_path();
        ctx.fill();
        // EXIT text (right half) — bold sans, white
        ctx.set_fill_style("#ffffff");
        ctx.set_font("bold 120px Helvetica, Arial, sans-serif");
        ctx.set_text_baseline("middle");
        ctx.fill_text("EXIT", 220, 96);
        // subtle inner glow around border for the emissive feel
        let grad = ctx.create_linear_gradient(0, 0, 0, h);
        grad.add_color_stop(0, "rgba(255,255,255,0.10)");
        grad.add_color_stop(0.5, "rgba(255,255,255,0)");
        grad.add_color_stop(1, "rgba(0,0,0,0.15)");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
    })
}

impl Game {
    /// The first half here, then `place_props_tail`.
    pub fn place_props(&mut self, dist: &[i32]) {
        let mut cx = self.place_props_head(dist);
        self.place_props_tail(&mut cx, dist);
    }

    /// The first half of `place_props`: everything before the hand-off to
    /// the second half (returns the shared state).
    pub fn place_props_head(&mut self, dist: &[i32]) -> PropsCtx {
        let g = self;
        let mut cx = PropsCtx::default();
        // candidate open tiles, far from spawn, spread apart
        for z in 1..g.h - 1 {
            for x in 1..g.w - 1 {
                let d = dist[g.t_idx(x, z)];
                if g.tiles[g.t_idx(x, z)] == 0 && d > 8 {
                    cx.cands.push(Cand { x, z, d });
                }
            }
        }
        let mut i = cx.cands.len() as i64 - 1;
        while i > 0 {
            let j = (rnd() * (i + 1) as f64).floor() as usize;
            cx.cands.swap(i as usize, j);
            i -= 1;
        }

        g.props_exit_door(&mut cx);
        let (bat_tip_geo, bat_tip_mat) = g.props_pickups(&mut cx);
        g.props_chairs(&mut cx);
        g.props_crt_tv(&mut cx);
        g.props_wall_clock(&mut cx);
        g.props_balloon(&mut cx);
        g.props_payphone(&mut cx);
        g.props_mannequin(&mut cx);

        // place each prop type with random counts, sometimes stacked absurdly;
        // each kind keeps to the places it belongs in, a few side by side
        let reg = move |g: &mut Game, built: &Built, si: &SpotInfo| register_cabinet_drawers(g, built, si, &bat_tip_geo, &bat_tip_mat);
        let wall = |off: f64, home: Home, row: i32, spacing: f64| ScatterOpts {
            wall: true,
            wall_offset: Some(off),
            home: Some(home),
            row,
            spacing,
            ..Default::default()
        };
        let floor = |home: Home, row: i32, spacing: f64| ScatterOpts { home: Some(home), row, spacing, ..Default::default() };
        scatter(g, &mut cx, build_filing_cabinet, 3, &ScatterOpts { can_stack: true, on_placed: Some(&reg), ..wall(0.3, OFFICE, 3, 0.66) });
        scatter(g, &mut cx, build_locker, 2, &ScatterOpts { can_stack: true, ..wall(0.28, LOCKER_ROOM, 2, 1.42) });
        // no machine on the Lobby (level 4: its emptiness is the point), nor
        // in the Showroom or the Borrowed House, whose light is their own;
        // elsewhere only where an office, a hall or the Poolrooms would have
        // put one, never in a zone left bare (docs/VENDING.md, where it stands)
        if g.level_num != 4 && !matches!(g.level_scp.as_deref(), Some("store" | "house")) {
            scatter(
                g,
                &mut cx,
                vending::build_vending,
                1,
                &ScatterOpts { on_placed: Some(&vending::placed), ..wall(vending::model::D / 2.0 + 0.05, VENDING, 1, 0.0) },
            );
        }
        scatter(g, &mut cx, build_cardboard_stack, 4, &floor(STORAGE, 3, 0.75));
        scatter(g, &mut cx, build_wet_floor_sign, 2, &floor(WET, 1, 0.0));
        scatter(g, &mut cx, build_fire_extinguisher, 2, &ScatterOpts { y: 0.0, ..wall(0.16, SERVICE, 1, 0.0) });
        scatter(g, &mut cx, build_trash_can, 3, &floor(PUBLIC, 1, 0.0));
        scatter(g, &mut cx, build_rubber_duck, 3, &floor(BATH_TOYS, 3, 0.3));
        scatter(g, &mut cx, build_traffic_cone, 3, &floor(WORKS, 3, 0.9));
        scatter(g, &mut cx, build_wheelchair, 1, &floor(HOME_CARE, 1, 0.0));
        scatter(g, &mut cx, build_baby_doll, 1, &floor(NURSERY, 1, 0.0));
        scatter(g, &mut cx, build_suitcase, 2, &floor(LUGGAGE, 2, 0.7));
        scatter(g, &mut cx, build_book_stack, 3, &floor(READING, 2, 0.55));
        scatter(g, &mut cx, build_coat_rack, 2, &floor(CLOAKROOM, 1, 0.0));
        scatter(g, &mut cx, build_fluorescent_tube, 3, &floor(WORKS, 3, 0.35));
        scatter(g, &mut cx, build_tv_on_floor, 1, &floor(LIVING, 1, 0.0));
        scatter(g, &mut cx, build_pile, 2, &floor(STORAGE, 1, 0.0));
        // scattered single plants — signs someone cared once
        scatter(g, &mut cx, build_potted_plant, 3, &floor(GREENERY, 1, 0.0));
        scatter(g, &mut cx, build_floor_plant, 1, &floor(GREENERY, 1, 0.0));
        cx
    }

    /// the exit door: the farthest wall-adjacent tile (sets `exit_data`)
    fn props_exit_door(&mut self, cx: &mut PropsCtx) {
        let g = self;
        let mut best: Option<Cand> = None;
        for c in &cx.cands {
            if has_wall_n(g, c) && (best.is_none() || c.d > best.unwrap().d) {
                best = Some(*c);
            }
        }
        // the last fallback has no distance; it gets `d` = 0
        let exit_tile = match best {
            Some(b) => b,
            None => match cx.take(None) {
                Some(c) => c,
                None => Cand { x: g.w - 2, z: g.h - 2, d: 0 },
            },
        };
        cx.exit_tile = Some(exit_tile);
        cx.taken.push(exit_tile);

        let spot = props_wall_adj_spot(g, exit_tile.x, exit_tile.z).unwrap_or(WallSpot {
            pos: v3(g.wx(exit_tile.x), 0, g.wz(exit_tile.z)),
            normal: v3(0, 0, 1),
            yaw: 0.0,
        });
        g.build_exit_door(&spot);
    }

    /// The way down at `spot` (on the floor, against its wall): the door,
    /// its EXIT sign and lights, `exit_data` and its use (the Stairwell's
    /// steel door, the Loop's plain office door).
    pub(crate) fn build_exit_door(&mut self, spot: &WallSpot) {
        let g = self;
        let dg = group();
        dg.position().copy(&spot.pos);
        dg.rotation().y = spot.yaw;
        let frame = mesh(pg("Props__exitDoor_frame", box_geo!(1.5, 2.5, 0.18)), &g.mats.wood);
        frame.position().y = 1.25;
        dg.add(&frame);
        let door_mat = lambert!({ color: 0x6b5636, emissive: 0x1a1408 });
        let door = mesh(pg("Props__exitDoor_leaf", box_geo!(1.1, 2.2, 0.1)), &door_mat);
        door.position().set(0, 1.1, 0.06);
        dg.add(&door);
        let gap = mesh(plane_geo!(0.1, 2.2), basic_mat!({ color: 0xfff6d8 }));
        gap.position().set(0.5, 1.1, 0.12);
        dg.add(&gap);
        // ---- proper green office EXIT sign: running person + arrow + EXIT text ----
        let exit_sign = exit_sign_tex();
        // The sign is an unlit (basic) material so it self-illuminates even in the dark.
        let sign_mat = basic_mat!({ map: &exit_sign });
        let sign = mesh(plane_geo!(1.30, 0.48), &sign_mat);
        sign.position().set(0, 2.68, 0.14);
        dg.add(&sign);
        // recessed housing behind the sign so it reads as a mounted fixture
        let sign_back = mesh(pg("Props__exitSign_housing", box_geo!(1.36, 0.54, 0.04)), lambert!({ color: 0x1a1a1a }));
        sign_back.position().set(0, 2.68, 0.12);
        dg.add(&sign_back);
        // tiny mounting hooks at top corners
        for s in [-1.0, 1.0] {
            let hook = mesh(cyl_geo!(0.015, 0.015, 0.06, 6), &g.mats.chrome);
            // standoff from the wall face (z -0.06) to the housing's back (z 0.10);
            // it was buried inside the housing
            hook.rotation().x = PI / 2.0;
            hook.scale().y = 2.67;
            hook.position().set(s * 0.6, 2.92, 0.02);
            dg.add(&hook);
        }
        g.world.add(&dg);
        // greenish emergency-exit light (weaker + tinted so it colors the sign area)
        let light = point_light!(0x9ce8b0, 3.5, 6, 1.6);
        light.position().copy(&spot.pos).add_scaled_vector(&spot.normal, 0.6);
        light.position().y = spot.pos.y + 2.5;
        g.world.add(&light);
        // second warmer light for the door itself
        let door_light = point_light!(0xffe9b8, 4, 5, 1.6);
        door_light.position().copy(&spot.pos).add_scaled_vector(&spot.normal, 1.0);
        door_light.position().y = spot.pos.y + 1.6;
        g.world.add(&door_light);
        let mut use_pos = spot.pos;
        use_pos.add_scaled_vector(&spot.normal, 0.9);
        use_pos.y = spot.pos.y + EYE;
        g.exit_data = Some(ExitData { pos: use_pos, normal: spot.normal, door: None });
        dg.user().gameplay = true;
        let to_stairs = level_has_stairwell(g, g.level_num);
        if to_stairs {
            // a steel door made up to look like a janitor's closet
            door_mat.m().color.set_hex(0x59606a);
            door_mat.m().emissive.set_hex(0x0a0c0e);
            let plaque = mesh(
                plane_geo!(0.34, 0.1),
                basic_mat!({
                    map: text_tex("STAIRS", &TextTexOpts { w: 256.0, h: 76.0, font: "bold 44px Helvetica, Arial, sans-serif".into(), color: "#e8e2d0".into(), glow: 2.0, ..Default::default() }),
                    transparent: true,
                }),
            );
            plaque.position().set(0, 1.62, 0.115);
            dg.add(&plaque);
        }
        if g.level_scp.as_deref() == Some("loop") {
            // the Loop's way in is an ordinary office door: no EXIT sign over it and
            // no green light (`modes::loop_rooms`)
            for c in dg.children() {
                if c.pos().y > 2.5 {
                    c.set_visible(false);
                }
            } // the sign, its housing, its hooks
            g.world.remove(&light);
            door_mat.m().color.set_hex(0x8a7a5c);
            door_mat.m().emissive.set_hex(0x120e08);
        }
        g.interact(
            use_pos,
            1.7,
            if to_stairs { "[E] OPEN THE STEEL DOOR" } else { "[E] ENTER THE DOOR" },
            |_g| true,
            move |g| g.use_exit_door(to_stairs),
        );
    }

    /// almond water (restores stamina) and batteries (recharge the flashlight).
    /// Returns the battery tip geometry/material the drawer contents reuse.
    fn props_pickups(&mut self, cx: &mut PropsCtx) -> (Geo, Mat) {
        let g = self;
        // --- almond water (restores stamina) ---
        let bottle_geo = pg("Props__almond_bottle", cyl_geo!(0.09, 0.09, 0.3, 10));
        let bottle_mat = lambert!({ color: 0x9fc4d8, emissive: 0x16323e });
        let cap_geo = pg("Props__almond_cap", cyl_geo!(0.05, 0.05, 0.06, 8));
        for _i in 0..3 {
            let Some(c) = cx.take(None) else { break };
            let ag = group();
            let px = g.wx(c.x) + rand2(-1.0, 1.0);
            let pz = g.wz(c.z) + rand2(-1.0, 1.0);
            ag.position().set(px, 0.15, pz);
            let b = mesh(&bottle_geo, &bottle_mat);
            ag.add(&b);
            let cap = mesh(&cap_geo, &g.mats.metal);
            cap.position().y = 0.18;
            ag.add(&cap);
            g.world.add(&ag);
            g.almonds.push(Pickup { g: ag, taken: false, base: 0.15, phase: rand1(7.0) });
        }
        // --- batteries (recharge flashlight) ---
        let bat_body_geo = pg("Props__battery_body", cyl_geo!(0.06, 0.06, 0.18, 10));
        let bat_tip_geo = pg("Props__battery_tip", cyl_geo!(0.025, 0.025, 0.03, 8));
        let bat_body_mat = lambert!({ color: 0xc79b3d, emissive: 0x2a1f08 });
        let bat_tip_mat = lambert!({ color: 0xdad3c2 });
        for _i in 0..3 {
            let Some(c) = cx.take(None) else { break };
            let bg = group();
            let px = g.wx(c.x) + rand2(-1.0, 1.0);
            let pz = g.wz(c.z) + rand2(-1.0, 1.0);
            bg.position().set(px, 0.11, pz);
            let body = mesh(&bat_body_geo, &bat_body_mat);
            bg.add(&body);
            let tip = mesh(&bat_tip_geo, &bat_tip_mat);
            tip.position().y = 0.105;
            bg.add(&tip);
            g.world.add(&bg);
            g.battery_items.push(Pickup { g: bg, taken: false, base: 0.11, phase: rand1(7.0) });
        }
        (bat_tip_geo, bat_tip_mat)
    }

    /// chairs (office / dining variants) — one is knocked over
    fn props_chairs(&mut self, cx: &mut PropsCtx) {
        let g = self;
        let home = home_for(g, cx, Some(&SEATING));
        for i in 0..3 {
            let Some(c) = take_home(g, cx, home.as_deref(), false) else { break };
            let ch = build_detailed_chair(g);
            let px = g.wx(c.x) + rand2(-0.8, 0.8);
            let pz = g.wz(c.z) + rand2(-0.8, 0.8);
            ch.position().set(px, 0, pz);
            ch.rotation().y = rand1(PI * 2.0);
            // one chair per level is knocked over on its side
            if i == 2 {
                ch.rotation().z = PI / 2.0 - 0.12;
                ch.position().y = 0.28;
            }
            g.world.add(&ch);
        }
    }

    /// CRT television with live static (sets `tv_data`)
    fn props_crt_tv(&mut self, cx: &mut PropsCtx) {
        let g = self;
        let home = home_for(g, cx, Some(&LIVING));
        let Some(c) = take_home(g, cx, home.as_deref(), true) else { return };
        let spot = props_wall_adj_spot(g, c.x, c.z);
        let tg = group();
        let px = match spot {
            Some(s) => s.pos.x + s.normal.x * 0.55,
            None => g.wx(c.x),
        };
        let pz = match spot {
            Some(s) => s.pos.z + s.normal.z * 0.55,
            None => g.wz(c.z),
        };
        tg.position().set(px, 0, pz);
        tg.rotation().y = match spot {
            Some(s) => s.yaw,
            None => rand1(PI * 2.0),
        };
        let stand = mesh(pg("Props__crtTv_stand", box_geo!(0.9, 0.5, 0.5)), &g.mats.wood);
        stand.position().y = 0.25;
        tg.add(&stand);
        let body = mesh(pg("Props__crtTv_body", box_geo!(0.85, 0.62, 0.55)), &g.mats.dark);
        body.position().y = 0.85;
        tg.add(&body);
        let screen = mesh(plane_geo!(0.66, 0.46), basic_mat!({ map: &g.tex.static_tex }));
        screen.position().set(0, 0.85, 0.281);
        tg.add(&screen);
        let light = point_light!(0xa8b8d8, 0, 5.5, 1.8);
        light.position().set(0, 1.0, 0.8);
        tg.add(&light);
        g.world.add(&tg);
        g.tv_data = Some(TvData {
            light,
            world: v3(px, 1, pz),
            acc: 0.0,
            normal: match spot {
                Some(s) => s.normal,
                None => v3(0, 0, 1),
            },
        });
    }

    /// wall clock, wrong on purpose (sets `clock_data`)
    fn props_wall_clock(&mut self, cx: &mut PropsCtx) {
        let g = self;
        let home = home_for(g, cx, Some(&CLOCK_WALL));
        let Some(c) = take_home(g, cx, home.as_deref(), true) else { return };
        let Some(spot) = props_wall_adj_spot(g, c.x, c.z) else { return };
        let cg = group();
        cg.position().copy(&spot.pos).add_scaled_vector(&spot.normal, 0.05);
        cg.position().y = 2.1;
        cg.rotation().y = spot.yaw;
        let face_c = mesh(circle_geo!(0.3, 24), lambert!({ color: 0xd9d4c2 }));
        cg.add(&face_c);
        let rim = mesh(pg("Props__clock_rim", torus_geo!(0.3, 0.025, 8, 24)), &g.mats.metal);
        cg.add(&rim);
        let hand_m = basic_mat!({ color: 0x141210 });
        let hour_h = mesh(pg("Props__clock_hourHand", box_geo!(0.03, 0.16, 0.01)), &hand_m);
        hour_h.position().y = 0.06;
        hour_h.rotation().z = 2.4;
        let hour_pivot = group();
        hour_pivot.add(&hour_h);
        hour_pivot.position().z = 0.012;
        cg.add(&hour_pivot);
        let sec_h = mesh(pg("Props__clock_secondHand", box_geo!(0.012, 0.24, 0.008)), basic_mat!({ color: 0x8a1a12 }));
        sec_h.position().y = 0.09;
        let sec_pivot = group();
        sec_pivot.add(&sec_h);
        sec_pivot.position().z = 0.02;
        cg.add(&sec_pivot);
        g.world.add(&cg);
        g.clock_data = Some(ClockData { sec_pivot, world: cg.pos(), acc: 0.0, tock: false, normal: spot.normal });
    }

    /// red balloon (bobs in place)
    fn props_balloon(&mut self, cx: &mut PropsCtx) {
        let g = self;
        let Some(c) = cx.take(None) else { return };
        let bg = group();
        let px = g.wx(c.x) + rand2(-0.5, 0.5);
        let pz = g.wz(c.z) + rand2(-0.5, 0.5);
        bg.position().set(px, 0, pz);
        let bal = mesh(pg("Props__balloon_body", sphere_geo!(0.3, 18, 14)), lambert!({ color: 0xb01a1a, emissive: 0x2a0505 }));
        bal.position().y = 2.35;
        bal.scale().y = 1.12;
        bg.add(&bal);
        bg.user().floats = true; // a helium balloon: the placement pass leaves it be
        let s_geo = Geo::from_points(&[v3(0, 2.05, 0), v3(0.03, 1.2, 0.02)]);
        bg.add(&line(s_geo, &line_mat!({ color: 0x777066 })));
        g.world.add(&bg);
        let (cxf, czf) = (c.x as f64, c.z as f64);
        g.animate(move |_g, _dt, t| {
            bal.position().y = 2.35 + (t * 0.9 + cxf).sin() * 0.07;
            bal.position().x = (t * 0.5 + czf).sin() * 0.05;
        });
    }

    /// payphone that rings when you come close (heavily detailed); sets `phone_data`
    fn props_payphone(&mut self, cx: &mut PropsCtx) {
        let g = self;
        let Some(c) = cx.take(Some(&|c: &Cand| has_wall_n(g, c))) else { return };
        let Some(spot) = props_wall_adj_spot(g, c.x, c.z) else { return };
        let pgp = group();
        pgp.position().copy(&spot.pos).add_scaled_vector(&spot.normal, 0.16);
        pgp.position().y = 1.35;
        pgp.rotation().y = spot.yaw;
        // ---- main body: enclosure + darker recessed inner face ----
        let body_mat = lambert!({ color: 0x5a5f65 });
        let bx = mesh(pg("Props__payphone_housing", box_geo!(0.46, 0.70, 0.22)), &body_mat);
        pgp.add(&bx);
        // top cornice / hood
        let hood = mesh(pg("Props__payphone_hood", box_geo!(0.48, 0.05, 0.24)), lambert!({ color: 0x3a3f45 }));
        hood.position().set(0, 0.375, 0.01);
        pgp.add(&hood);
        // dark inner recessed panel
        let face = mesh(pg("Props__payphone_face", box_geo!(0.40, 0.60, 0.02)), lambert!({ color: 0x1a1c20 }));
        face.position().set(0.02, 0, 0.108);
        pgp.add(&face);
        // brand / instructions plate at top
        let brand_tex = canvas_tex_opt(256, 96, false, |ctx, w, h| {
            ctx.set_fill_style("#e0dabf");
            ctx.fill_rect(0, 0, w, h);
            ctx.set_stroke_style("#1a1408");
            ctx.set_line_width(3);
            ctx.stroke_rect(3, 3, w - 6.0, h - 6.0);
            ctx.set_fill_style("#7a1010");
            ctx.set_font("bold 42px Impact, sans-serif");
            ctx.set_text_align("center");
            ctx.fill_text("PHONE", 128, 44);
            ctx.set_fill_style("#1a1408");
            ctx.set_font("bold 18px monospace");
            ctx.fill_text("LOCAL & LONG DISTANCE", 128, 74);
        });
        let brand = mesh(plane_geo!(0.32, 0.10), basic_mat!({ map: &brand_tex, transparent: true }));
        brand.position().set(0.02, 0.24, 0.121);
        pgp.add(&brand);
        // ---- 4x3 keypad ----
        let digits = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "*", "0", "#"];
        let key_mat = lambert!({ color: 0xd0c8b0 });
        let key_tex = |label: &str| {
            let label = label.to_string();
            canvas_tex_opt(64, 64, false, move |ctx, w, h| {
                ctx.set_fill_style("#d0c8b0");
                ctx.fill_rect(0, 0, w, h);
                ctx.set_fill_style("#1a1a1a");
                ctx.set_font("bold 34px sans-serif");
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.fill_text(&label, 32, 34);
            })
        };
        for r in 0..4 {
            for ccol in 0..3 {
                let idx = r * 3 + ccol;
                let (rf, cf) = (r as f64, ccol as f64);
                let key = mesh(pg("Props__payphone_key", box_geo!(0.058, 0.038, 0.02)), &key_mat);
                key.position().set(-0.09 + cf * 0.09, 0.10 - rf * 0.055, 0.118);
                pgp.add(&key);
                let label = mesh(plane_geo!(0.05, 0.035), basic_mat!({ map: key_tex(digits[idx]), transparent: true }));
                label.position().set(-0.09 + cf * 0.09, 0.10 - rf * 0.055, 0.13);
                pgp.add(&label);
            }
        }
        // ---- coin slot area (top-right of face) ----
        let coin_slot_housing = mesh(pg("Props__payphone_coinPlate", box_geo!(0.10, 0.06, 0.02)), lambert!({ color: 0x2a2e34 }));
        coin_slot_housing.position().set(0.14, 0.14, 0.12);
        pgp.add(&coin_slot_housing);
        let coin_slot = mesh(pg("Props__payphone_coinSlot", box_geo!(0.06, 0.008, 0.005)), &g.mats.dark);
        coin_slot.position().set(0.14, 0.14, 0.132);
        pgp.add(&coin_slot);
        // coin return cup at bottom
        let coin_cup = mesh(pg("Props__payphone_coinCup", box_geo!(0.11, 0.05, 0.03)), lambert!({ color: 0x2a2e34 }));
        coin_cup.position().set(0.14, -0.24, 0.115);
        pgp.add(&coin_cup);
        let coin_cup_slot = mesh(pg("Props__payphone_coinCupSlot", box_geo!(0.09, 0.025, 0.02)), &g.mats.dark);
        coin_cup_slot.position().set(0.14, -0.25, 0.13);
        pgp.add(&coin_cup_slot);
        // ---- small display screen ----
        let screen = mesh(plane_geo!(0.18, 0.05), basic_mat!({ color: 0x306040 }));
        screen.position().set(0.02, 0.18, 0.122);
        pgp.add(&screen);
        // ---- handset cradle (left side) ----
        let cradle = mesh(pg("Props__payphone_cradle", box_geo!(0.10, 0.04, 0.06)), lambert!({ color: 0x3a3f45 }));
        cradle.position().set(-0.18, 0.14, 0.12);
        pgp.add(&cradle);
        let cradle2 = cradle.duplicate();
        cradle2.position().y = -0.16;
        pgp.add(&cradle2);
        // handset (upright, hanging on cradle)
        let handset = mesh(pg("Props__payphone_handset", box_geo!(0.08, 0.34, 0.09)), &g.mats.dark);
        handset.position().set(-0.18, -0.02, 0.14);
        pgp.add(&handset);
        // earpiece + mouthpiece caps
        let ear = mesh(pg("Props__payphone_earCap", cyl_geo!(0.035, 0.035, 0.02, 10)), &key_mat);
        // oval like the handset's cups (outer half squashed), inside their walls;
        // round and centred on the cups they stuck 1.5 cm past the handset ends
        ear.scale().z = 0.66;
        ear.rotation().x = PI / 2.0;
        ear.position().set(-0.18, 0.121, 0.14);
        pgp.add(&ear);
        let mouth = ear.duplicate();
        mouth.position().set(-0.18, -0.161, 0.14);
        pgp.add(&mouth);
        // little holes on earpiece (draw dots)
        let dots_mat = basic_mat!({ color: 0x1a1a1a });
        for r in 0..6 {
            let dot = mesh(circle_geo!(0.004, 6), &dots_mat);
            let a = (r as f64 / 6.0) * PI * 2.0;
            dot.position().set(-0.18 + a.cos() * 0.02, 0.121 + a.sin() * 0.013, 0.151);
            pgp.add(&dot);
        }
        // coiled handset cord — a series of small torus rings
        let cord_mat = lambert!({ color: 0x1a1a1a });
        for i in 0..8 {
            let ring = mesh(pg("Props__payphone_cordCoil", torus_geo!(0.018, 0.005, 5, 10)), &cord_mat);
            ring.rotation().x = PI / 2.0; // coil turns stacked along the hanging cord (a Z turn left them facing out)
            let fi = i as f64;
            ring.position().set(-0.18, -0.22 - fi * 0.02, 0.16 + (fi * 0.8).sin() * 0.008);
            pgp.add(&ring);
        }
        // small info sticker
        let info = mesh(plane_geo!(0.08, 0.05), lambert!({ color: 0xf0e0b0 }));
        info.position().set(0.14, -0.10, 0.121);
        pgp.add(&info);
        let info_text = mesh(box_geo!(0.06, 0.004, 0.002), &g.mats.dark);
        info_text.position().set(0.14, -0.09, 0.124);
        pgp.add(&info_text);
        // graffiti on the side, sometimes
        if rnd() < 0.5 {
            let gtex = handwritten_tex(
                "CALL ME",
                &HandOpts {
                    color: "rgba(10,10,10,0.9)".into(),
                    thickness: 4.0,
                    jitter: 1.6,
                    wobble: 1.2,
                    w: 256.0,
                    h: 64.0,
                    ..Default::default()
                },
            );
            let grafitti = mesh(plane_geo!(0.20, 0.05), basic_mat!({ map: &gtex, transparent: true }));
            grafitti.position().set(0, -0.32, 0.122);
            pgp.add(&grafitti);
        }
        g.world.add(&pgp);
        let mut use_pos = pgp.pos();
        use_pos.add_scaled_vector(&spot.normal, 0.8);
        use_pos.y = EYE;
        g.phone_data = Some(PhoneData {
            world: pgp.pos(),
            ringing: false,
            answered: false,
            ring_t: 0.0,
            normal: spot.normal,
            handset: Some(handset.clone()),
            f: HashMap::new(),
        });
        // subtle jitter of handset while ringing
        let hs = handset.clone();
        g.animate(move |g, _dt, t| {
            if g.phone_data.as_ref().is_some_and(|p| p.ringing) {
                hs.position().x = -0.18 + (t * 40.0).sin() * 0.008;
                hs.position().y = -0.02 + (t * 40.0).cos() * 0.006;
            } else {
                hs.position().x = -0.18;
                hs.position().y = -0.02;
            }
        });
        g.interact(
            use_pos,
            1.6,
            "[E] ANSWER",
            |g| g.phone_data.as_ref().is_some_and(|p| p.ringing && !p.answered),
            |g| g.answer_payphone(),
        );
    }

    /// answering the payphone — R769 three-outcome phone answer:
    ///  40% whisper — sanity drain (~15 pts).
    ///  35% exit hint — the place shows the way to the door
    ///      (`show_the_way`, from `update_hud` while `phone_data.f["hintUntil"]`).
    ///  25% alert — distant scream, all entities snap to hunt.
    pub fn answer_payphone(&mut self) {
        let g = self;
        if let Some(p) = g.phone_data.as_mut() {
            p.answered = true;
            p.ringing = false;
        }
        let roll = rnd();
        if roll < 0.40 {
            g.sfx_whisper();
            g.player.sanity = clamp(g.player.sanity - 15.0, 0.0, SAN_MAX);
            let lines = [
                "\"...it is behind you.\"",
                "\"stop running. it is tired too.\"",
                "\"the door only opens when you stop looking.\"",
                "\"we are still here. we have always been here.\"",
            ];
            let l = lines[(rnd() * lines.len() as f64).floor() as usize];
            g.show_msg(l, 2.8);
        } else if roll < 0.75 {
            // Exit hint — the place shows the way (`update_hud` calls `show_the_way`).
            if let Some(answer) = g.way_answer_here() {
                let until = g.perf_t + 8.0; // (the hint is answered within this while)
                if let Some(p) = g.phone_data.as_mut() {
                    p.f.insert("hintUntil".into(), until);
                }
                if g.ac.is_some() {
                    // Static-y click + a low tone (mysterious dial-tone hint).
                    g.noise_burst(0.10, 900.0, 0.7, 0.06, "bandpass", None, 0.0, 0.0);
                    g.blip(320.0, 0.30, "sine", 0.10, None, 0.0, 0.0);
                }
                // it says only where to look; the place shows the rest
                let line = match answer {
                    WayAnswer::Lights => "\"...the lights know the way.\"",
                    WayAnswer::Prints => "\"...look at the floor.\"",
                    WayAnswer::Call => "\"...listen for the door.\"",
                };
                g.show_msg(line, 2.5);
            } else {
                // No way to the door from here — fall back to whisper hint.
                g.sfx_whisper();
                g.player.sanity = clamp(g.player.sanity - 8.0, 0.0, SAN_MAX);
                g.show_msg("\"...you are not looking hard enough.\"", 2.5);
            }
        } else {
            // Alert — distant scream + all entities snap to hunt.
            if g.ac.is_some() {
                g.noise_burst(0.60, 300.0, 2.5, 0.25, "lowpass", None, 0.0, 0.0);
                g.blip(180.0, 0.50, "sawtooth", 0.15, Some(90.0), 0.0, 0.0);
                g.blip(240.0, 0.35, "sine", 0.10, Some(140.0), 0.2, 0.0);
            }
            for ent in g.entities.iter_mut() {
                if ent.kind == "watcher" {
                    continue;
                }
                if ent.state != "hunt" {
                    ent.state = "hunt".into();
                    ent.lose_t = 0.0;
                }
            }
            g.player.sanity = clamp(g.player.sanity - 25.0, 0.0, SAN_MAX);
            g.ui.flash.set_opacity(0.6);
            g.schedule(250.0, |g| {
                g.ui.flash.set_opacity(0.0);
            });
            g.show_msg("SOMEONE SCREAMED. THEY HEARD.", 2.8);
        }
    }

    /// mannequin. do not blink (sets `man_data`; its timer is `f["timer"]`)
    fn props_mannequin(&mut self, cx: &mut PropsCtx) {
        let g = self;
        let Some(c) = cx.take(Some(&|cc: &Cand| cc.d > 14)) else { return };
        let mg = group();
        let px = g.wx(c.x) + rand2(-0.7, 0.7);
        let pz = g.wz(c.z) + rand2(-0.7, 0.7);
        mg.position().set(px, 0, pz);
        let skin = lambert!({ color: 0xcfc8ba });
        let torso = mesh(pg("Props__mannequin_torso", capsule_geo!(0.21, 0.62, 4, 10)), &skin);
        torso.position().y = 1.12;
        mg.add(&torso);
        let head = mesh(pg("Props__mannequin_head", sphere_geo!(0.155, 14, 10)), &skin);
        head.position().y = 1.72;
        head.scale().y = 1.25;
        mg.add(&head);
        for s in [-1.0, 1.0] {
            let leg = mesh(pg("Props__mannequin_leg", capsule_geo!(0.075, 0.62, 4, 8)), &skin);
            leg.position().set(s * 0.11, 0.42, 0);
            mg.add(&leg);
            let arm = mesh(pg("Props__mannequin_arm", capsule_geo!(0.055, 0.52, 4, 8)), &skin);
            arm.position().set(s * 0.3, 1.1, 0);
            arm.rotation().z = s * 0.12;
            mg.add(&arm);
        }
        mg.rotation().y = rand1(PI * 2.0);
        g.world.add(&mg);
        let timer0 = rand2(3.0, 6.0);
        let mut f = HashMap::new();
        f.insert("timer".to_string(), timer0);
        g.man_data = Some(ManData { g: mg.clone(), f, b: HashMap::new() });
        let timer = Rc::new(std::cell::Cell::new(timer0));
        let man_g = mg;
        g.animate(move |g, dt, _t| {
            let mp = man_g.pos();
            let pp = g.player.pos;
            let d2 = mp.distance_to_squared(&pp);
            if d2 > 200.0 {
                return;
            }
            let mut to_man = mp;
            to_man.sub(&pp).normalize();
            let seen = g.player.fwd().dot(&to_man) > 0.1; // inside view cone
            timer.set(timer.get() - dt);
            if timer.get() <= 0.0 {
                timer.set(rand2(2.5, 6.0));
                if !seen {
                    man_g.rotation().y = (pp.x - mp.x).atan2(pp.z - mp.z);
                    if d2 > 20.0 && rnd() < 0.45 {
                        // a step closer while unwatched
                        let mut dir = pp;
                        dir.sub(&mp).set_y(0).normalize().multiply_scalar(0.9);
                        let nx = mp.x + dir.x;
                        let nz = mp.z + dir.z;
                        if !circle_hits_wall(g, nx, nz, 0.3) {
                            man_g.position().x = nx;
                            man_g.position().z = nz;
                        }
                    }
                }
            }
            // mirror the timer into `man_data` (while it is still this mannequin)
            if let Some(m) = g.man_data.as_mut() {
                if m.g.ptr_eq(&man_g) {
                    m.f.insert("timer".into(), timer.get());
                }
            }
        });
    }
}

/// (one of `chair_cushion_colors` for the cushion)
fn build_detailed_chair(g: &mut Game) -> Obj {
    let chair_cushion_colors = [0x6a2a2au32, 0x2a3a5a, 0x3a5a3a, 0x5a4a2a, 0x2a2a2a];
    let cg = group();
    let frame_mat = lambert!({ color: 0x2a2018 });
    let wood_tone = lambert!({ color: 0x4c3b22 });
    // seat frame (wooden square)
    let seat = mesh(pg("Props__chair_seat", box_geo!(0.50, 0.05, 0.50)), &wood_tone);
    seat.position().y = 0.48;
    cg.add(&seat);
    // padded cushion on top of seat
    let cushion_mat = lambert!({ color: choice(&chair_cushion_colors) });
    let cushion = mesh(pg("Props__chair_cushion", box_geo!(0.44, 0.06, 0.44)), &cushion_mat);
    cushion.position().y = 0.53;
    cg.add(&cushion);
    // cushion piping / edge trim
    for (dx, dz, rot) in [(0.0, 0.22, 0.0), (0.0, -0.22, 0.0), (0.22, 0.0, PI / 2.0), (-0.22, 0.0, PI / 2.0)] {
        let trim = mesh(pg("Props__chair_piping", cyl_geo!(0.008, 0.008, 0.45, 6)), lambert!({ color: 0x1a1008 }));
        trim.rotation().z = PI / 2.0;
        trim.rotation().y = rot;
        trim.position().set(dx, 0.535, dz);
        cg.add(&trim);
    }
    // button tuft in center of cushion
    let tuft = mesh(pg("Props__chair_tuft", sphere_geo!(0.02, 6, 5)), &cushion_mat);
    tuft.position().set(0, 0.55, 0);
    tuft.scale().set(1, 0.4, 1);
    cg.add(&tuft);
    // curved backrest — slats
    let _back_x = -0.22;
    // outer back frame
    for s in [-1.0, 1.0] {
        let post = mesh(pg("Props__chair_post", box_geo!(0.045, 0.62, 0.045)), &wood_tone);
        post.position().set(s * 0.22, 0.82, -0.22);
        cg.add(&post);
    }
    // horizontal back rails
    for y in [0.72, 1.05] {
        let rail = mesh(pg("Props__chair_rail", box_geo!(0.48, 0.04, 0.03)), &wood_tone);
        rail.position().set(0, y, -0.22);
        cg.add(&rail);
    }
    // vertical slats
    for sl in 0..5 {
        let slat = mesh(pg("Props__chair_slat", box_geo!(0.025, 0.32, 0.02)), &wood_tone);
        slat.position().set(-0.16 + sl as f64 * 0.08, 0.88, -0.22);
        cg.add(&slat);
    }
    // small backrest cushion pad
    let back_pad = mesh(pg("Props__chair_backPad", box_geo!(0.42, 0.28, 0.04)), &cushion_mat);
    back_pad.position().set(0, 0.90, -0.20);
    cg.add(&back_pad);
    // legs with taper (each leg = 2 boxes for taper illusion)
    for (lx, lz) in [(-0.20, -0.20), (0.20, -0.20), (-0.20, 0.20), (0.20, 0.20)] {
        let leg_top = mesh(pg("Props__chair_legTop", box_geo!(0.05, 0.30, 0.05)), &frame_mat);
        leg_top.position().set(lx, 0.32, lz);
        cg.add(&leg_top);
        let leg_bot = mesh(pg("Props__chair_legBottom", box_geo!(0.035, 0.18, 0.035)), &frame_mat);
        leg_bot.position().set(lx, 0.09, lz);
        cg.add(&leg_bot);
        // brass foot cap
        let cap = mesh(pg("Props__chair_footCap", cyl_geo!(0.022, 0.022, 0.02, 8)), &g.mats.chrome);
        cap.position().set(lx, 0.01, lz);
        cg.add(&cap);
    }
    // stretcher rails connecting legs (rear cross brace)
    let rear_brace = mesh(pg("Props__chair_braceX", box_geo!(0.42, 0.02, 0.02)), &frame_mat);
    rear_brace.position().set(0, 0.14, -0.20);
    cg.add(&rear_brace);
    let side_brace_l = mesh(pg("Props__chair_braceZ", box_geo!(0.02, 0.02, 0.42)), &frame_mat);
    side_brace_l.position().set(-0.20, 0.14, 0);
    cg.add(&side_brace_l);
    let side_brace_r = side_brace_l.duplicate();
    side_brace_r.position().x = 0.20;
    cg.add(&side_brace_r);
    cg
}
