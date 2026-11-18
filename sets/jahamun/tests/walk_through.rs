//! The whole walk on the set as built (`assets/set.bin`): from the street
//! above, down the stairs, along the plaza, through the portal and down the
//! walkway to the gate; and what must stop you (the glass, the tiled wall,
//! the gate, the stairs' edge). Moves only: no window, no frame.
use jahamun::data::SetData;
use jahamun::walk::Walk;

fn walk() -> (Walk, SetData) {
    let bytes = std::fs::read(jahamun::home().join("assets/set.bin")).expect("assets/set.bin (make it: pipeline/set.py)");
    let d = SetData::read(&bytes).expect("the set file");
    (Walk::new(d.walk.clone()), d)
}

/// Walk toward (x, z) in 5 cm steps until there or stuck; the feet.
fn go(w: &Walk, mut p: [f64; 3], x: f64, z: f64) -> [f64; 3] {
    for _ in 0..20_000 {
        let (dx, dz) = (x - p[0], z - p[2]);
        let d = (dx * dx + dz * dz).sqrt();
        if d < 0.03 {
            break;
        }
        let s = d.min(0.05);
        let q = w.step(p, dx / d * s, dz / d * s);
        if (q[0] - p[0]).abs() + (q[2] - p[2]).abs() < 1e-9 {
            break;
        }
        p = q;
    }
    p
}

fn near(p: [f64; 3], x: f64, y: f64, z: f64) -> bool {
    (p[0] - x).abs() < 0.1 && (p[1] - y).abs() < 0.02 && (p[2] - z).abs() < 0.1
}

#[test]
fn from_the_street_above_to_the_gate() {
    let (w, d) = walk();
    let top = d.spawns["top"];
    let mut p = [top[0], top[1], top[2]];
    assert!(w.fits(p[0], p[2], p[1]), "the top spawn stands: {p:?}");
    // to the stairs' head (across the coping's gap), then down both flights
    p = go(&w, p, -7.0, 20.2);
    p = go(&w, p, -4.75, 20.2);
    assert!(near(p, -4.75, 7.2, 20.2), "at the stairs' head: {p:?}");
    p = go(&w, p, -4.75, 5.0);
    assert!(near(p, -4.75, 0.2, 5.0), "down the stairs to the plaza: {p:?}");
    // across the plaza to the walkway's mouth, through the portal, to the gate
    p = go(&w, p, -0.9, 3.0);
    p = go(&w, p, -0.9, -500.0);
    assert!((p[1] - 0.2).abs() < 1e-9 && p[2] < -483.0 && p[2] > -485.0, "at the gate, not through it: {p:?}");
}

#[test]
fn the_glass_the_wall_and_the_edge_stop_you() {
    let (w, _) = walk();
    let inside = [-0.9, 0.2, -100.0];
    let p = go(&w, inside, 5.0, -100.0);
    assert!(p[0] < -0.06 - 0.2, "the barrier's foot stops you: {p:?}");
    let p = go(&w, inside, -5.0, -100.0);
    assert!(p[0] > -1.70 + 0.2, "the tiled wall stops you: {p:?}");
    // halfway down the upper flight, step off its open side toward the plaza
    let mut q = [-4.75, 7.2, 20.2];
    q = go(&w, q, -4.75, 16.0);
    let y = q[1];
    assert!(y > 4.0 && y < 7.0, "on the upper flight: {q:?}");
    let r = go(&w, q, 0.0, 16.0);
    assert!((r[1] - y).abs() < 0.2 && r[0] < -4.0, "no stepping off a flight: {r:?}");
    // the plaza does not climb onto the street above
    let s = go(&w, [-3.0, 0.2, 30.0], -12.0, 30.0);
    assert!((s[1] - 0.2).abs() < 1e-9 && s[0] > -5.5, "the retaining wall stops you: {s:?}");
}
