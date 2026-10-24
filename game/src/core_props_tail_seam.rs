//! A test seam for `place_props_tail` (core_props_tail.rs).
//!
//! The second half of `placeProps` calls functions other modules own: the
//! prop builders (core_props_builders.rs), `altPathExists` (world.rs) and the
//! lore tables (hud.rs). `wallAdjSpot` goes through core_props'
//! `props_wall_adj_spot` (and its `PropsHooks`), `take` / `hasWallN` are
//! core_props' own. The differential tests (game/tests/core_props_tail.rs)
//! replay a hand-off state recorded from the JS and need these calls to
//! behave like the JS harness's, so each goes through one of the `s_*`
//! functions here: if a test installed an override with [`set_tail_seam`],
//! it answers; otherwise the real function is called. Nothing is installed
//! in the game.
use crate::core_props::*;
use crate::prelude::*;
use crate::records::WallSpot;

/// The builders the second half calls (JS `buildX()`), with their arguments.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TailBuild {
    FilingCabinet, BookStack, CardboardStack, TrafficCone, Wheelchair, BabyDoll,
    PottedPlant, FloorPlant, Window, PaperPile, FloorHole,
    /// `buildBrokenMirror(yaw)`
    BrokenMirror(f64),
    /// `buildElevator(functional)`
    Elevator(bool),
}
impl TailBuild {
    /// the JS name (`buildFilingCabinet`, ...)
    pub fn js_name(&self) -> &'static str {
        match self {
            TailBuild::FilingCabinet => "buildFilingCabinet", TailBuild::BookStack => "buildBookStack",
            TailBuild::CardboardStack => "buildCardboardStack", TailBuild::TrafficCone => "buildTrafficCone",
            TailBuild::Wheelchair => "buildWheelchair", TailBuild::BabyDoll => "buildBabyDoll",
            TailBuild::PottedPlant => "buildPottedPlant", TailBuild::FloorPlant => "buildFloorPlant",
            TailBuild::Window => "buildWindow", TailBuild::PaperPile => "buildPaperPile",
            TailBuild::FloorHole => "buildFloorHole", TailBuild::BrokenMirror(_) => "buildBrokenMirror",
            TailBuild::Elevator(_) => "buildElevator",
        }
    }
    /// the real builder (core_props.rs)
    pub fn call(&self, g: &mut Game, cx: &mut PropsCtx) -> Built {
        match *self {
            TailBuild::FilingCabinet => build_filing_cabinet(g, cx), TailBuild::BookStack => build_book_stack(g, cx),
            TailBuild::CardboardStack => build_cardboard_stack(g, cx), TailBuild::TrafficCone => build_traffic_cone(g, cx),
            TailBuild::Wheelchair => build_wheelchair(g, cx), TailBuild::BabyDoll => build_baby_doll(g, cx),
            TailBuild::PottedPlant => build_potted_plant(g, cx), TailBuild::FloorPlant => build_floor_plant(g, cx),
            TailBuild::Window => build_window(g, cx), TailBuild::PaperPile => build_paper_pile(g, cx),
            TailBuild::FloorHole => build_floor_hole(g, cx), TailBuild::BrokenMirror(yaw) => build_broken_mirror(g, cx, yaw),
            TailBuild::Elevator(f) => build_elevator(g, cx, f),
        }
    }
}

/// Lore tables for tests: `LORE_ENTRIES[i].scp` per id and `LORE_ARCS` as (ids, colour).
#[derive(Clone, Debug, Default)]
pub struct TailLore { pub scp: Vec<Option<String>>, pub arcs: Vec<(Vec<usize>, u32)> }

/// Overrides; `None` fields fall through to the real functions.
#[derive(Default)]
pub struct TailSeam {
    /// `altPathExists(sx, sz, ex, ez, extraBlocked)`
    pub alt_path_exists: Option<Box<dyn FnMut(&Game, i32, i32, i32, i32, &[usize]) -> bool>>,
    /// every `buildX(...)` call
    pub build: Option<Box<dyn FnMut(&mut Game, &mut PropsCtx, &TailBuild) -> Built>>,
    /// `LORE_ENTRIES` / `LORE_ARCS`
    pub lore: Option<TailLore>,
}

thread_local! { static SEAM: RefCell<Option<TailSeam>> = const { RefCell::new(None) }; }

/// Install (Some) or remove (None) the overrides for this thread.
pub fn set_tail_seam(s: Option<TailSeam>) { SEAM.with(|c| *c.borrow_mut() = s); }

// take a closure out of the seam while it runs (it may re-enter the game), then put it back
macro_rules! with_hook {
    ($field:ident, |$f:ident| $call:expr) => {{
        let hook = SEAM.with(|s| s.borrow_mut().as_mut().and_then(|s| s.$field.take()));
        match hook {
            Some(mut $f) => {
                let r = $call;
                SEAM.with(|s| if let Some(s) = s.borrow_mut().as_mut() { s.$field = Some($f); });
                Some(r)
            }
            None => None,
        }
    }};
}

/// `take()` / `take(hasWallN)`
pub(crate) fn s_take(g: &Game, cx: &mut PropsCtx, wall: bool) -> Option<Cand> {
    if wall { cx.take(Some(&|c: &Cand| has_wall_n(g, c))) } else { cx.take(None) }
}
/// `wallAdjSpot(x, z)` (one `choice` draw) — core_props' hookable version
pub(crate) fn s_wall_adj_spot(g: &Game, x: i32, z: i32) -> Option<WallSpot> { props_wall_adj_spot(g, x, z) }
/// `altPathExists(...)`
pub(crate) fn s_alt_path_exists(g: &Game, sx: i32, sz: i32, ex: i32, ez: i32, blocked: &[usize]) -> bool {
    if let Some(r) = with_hook!(alt_path_exists, |f| f(g, sx, sz, ex, ez, blocked)) { return r; }
    g.alt_path_exists(sx, sz, ex, ez, blocked)
}
/// `buildX(...)`
pub(crate) fn s_build(g: &mut Game, cx: &mut PropsCtx, b: TailBuild) -> Built {
    if let Some(r) = with_hook!(build, |f| f(g, cx, &b)) { return r; }
    b.call(g, cx)
}
fn lore_seam<R>(f: impl FnOnce(&TailLore) -> R) -> Option<R> { SEAM.with(|s| s.borrow().as_ref().and_then(|s| s.lore.as_ref()).map(f)) }
/// `LORE_ENTRIES.length`
pub(crate) fn s_lore_count(g: &Game) -> usize { lore_seam(|l| l.scp.len()).unwrap_or_else(|| g.lore_entry_count()) }
/// `LORE_ENTRIES[id].scp`
pub(crate) fn s_lore_scp(g: &Game, id: usize) -> Option<String> {
    lore_seam(|l| l.scp.get(id).cloned().flatten()).unwrap_or_else(|| g.lore_entry_scp(id))
}
/// `loreArcFor(id)` (index into LORE_ARCS)
pub(crate) fn s_lore_arc_for(g: &Game, id: usize) -> Option<usize> {
    lore_seam(|l| l.arcs.iter().position(|a| a.0.contains(&id))).unwrap_or_else(|| g.lore_arc_for(id))
}
/// `LORE_ARCS[arc].color`
pub(crate) fn s_lore_arc_color(g: &Game, arc: usize) -> u32 { lore_seam(|l| l.arcs[arc].1).unwrap_or_else(|| g.lore_arc_color(arc)) }
/// `loreNoteBoosted(id)`
pub(crate) fn s_lore_note_boosted(g: &Game, id: usize) -> bool {
    lore_seam(|l| {
        for (ids, _) in &l.arcs {
            let idx = match ids.iter().position(|&i| i == id) { Some(i) if i > 0 => i, _ => continue };
            for k in 0..idx { if g.lore_found.contains(&ids[k]) { return true; } }
        }
        false
    }).unwrap_or_else(|| g.lore_note_boosted(id))
}
