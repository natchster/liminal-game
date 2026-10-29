//! What the vending machine does (`docs/VENDING.md`, the rules).
use super::model::*;
use crate::prelude::*;

/// How lit a powered machine's tile is at least, in the light map (its
/// glass and the glow on the floor in front of it)
pub const VEND_LIGHT: f64 = 0.5;
/// Its coils, key legends and price strips are not drawn beyond this
/// (a 5 mm wire is under a pixel there)
pub const DETAIL_FAR: f64 = 8.0;

/// What a machine is doing
#[derive(Clone, Debug, Default)]
pub struct State {
    /// its fine detail is drawn (within `DETAIL_FAR`); None: not yet set
    /// for the meshes it has now (a new merge draws everything)
    pub near: Option<bool>,
}

impl Game {
    /// Registers machine `i`'s uses and its animator, once it stands where it is
    pub fn vending_register(&mut self, i: usize) {
        self.animate(move |g, _dt, _t| g.vending_frame(i));
    }

    /// The machines stand where they will stay (the level's settling is
    /// done): the places of the hums for the ambience
    pub fn vending_settled(&mut self) {
        self.pp.vending_positions = self.vending.iter().filter(|m| m.powered).map(|m| m.world(0.0, 0.9, 0.0)).collect();
    }

    /// Machine `i`, each frame
    pub fn vending_frame(&mut self, i: usize) {
        let cp = self.camera.pos();
        let Some(m) = self.vending.get_mut(i) else { return };
        let p = m.pos();
        let near = hypot2(cp.x - p.x, cp.z - p.z) < DETAIL_FAR;
        if m.state.near != Some(near) {
            m.state.near = Some(near);
            for (s, o) in &m.body.meshes {
                if matches!(s, Slot::Coil | Slot::Legend | Slot::Keys | Slot::Labels) {
                    o.set_visible(near);
                }
            }
        }
    }

    /// Whether a body of radius `r` at (x, z) is inside a vending machine
    /// (they are solid, as the walls are)
    pub fn hits_vending(&self, x: f64, z: f64, r: f64) -> bool {
        self.vending.iter().any(|m| {
            let p = m.local(&Vec3::new(x, 0.0, z));
            let (cx, cz) = (p.x.clamp(-W / 2.0, W / 2.0), p.z.clamp(-D / 2.0, D / 2.0));
            (p.x - cx).powi(2) + (p.z - cz).powi(2) < r * r
        })
    }

    /// The light the powered machines give the light map: (tile, at least)
    pub fn vending_light(&self) -> Vec<(i32, i32, f64)> {
        self.vending
            .iter()
            .filter(|m| m.powered)
            .map(|m| {
                let f = m.world(0.0, 0.0, D / 2.0 + 0.6);
                (self.tx(f.x), self.tz(f.z), VEND_LIGHT)
            })
            .collect()
    }
}
