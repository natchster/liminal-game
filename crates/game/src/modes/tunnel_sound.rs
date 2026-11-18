//! The way in's sound (`docs/design/WAY_IN.md`, *The rules*): every sound
//! from the place. The bars' hum from the nearest bar and your steps on the
//! paint are `modes::tunnel`'s; here, the cars on the road behind the glass
//! (one every few seconds at first, then fewer, then none: the voice's
//! `quiet`, `tunnel_voice::CARS_STOP`) and, once the door has changed, the
//! air breathing out of it, from where it stands. No draw from any stream:
//! when each car comes is a function of its count.
use super::tunnel_voice::CARS_STOP;
use crate::prelude::*;

/// Where a car passes you, across the glass: the near lane, at the height
/// of its wheels and body.
const LANE_X: f64 = 3.2;
const CAR_Y: f64 = 0.8;
/// The open door's breath: how often, and from how far it is heard.
const AIR_EVERY: f64 = 7.0;
const AIR_FROM: f64 = 60.0;

/// The way in's sounds' own clocks.
#[derive(Clone, Copy, Debug, Default)]
pub struct TunnelSound {
    /// the way in's clock when the next car passes, and how many have
    next_car: f64,
    pub cars: u32,
    next_air: f64,
}

/// Seconds from car `k` to the next at the way in's clock `t`: about five
/// at first, stretching to fourteen as the road empties.
pub fn car_gap(k: u32, t: f64) -> f64 {
    let base = if t < 50.0 { 5.0 } else { 5.0 + (t - 50.0).min(40.0) * 0.22 };
    // (a fixed scatter, the golden ratio's: no two gaps alike, no draw)
    let f = (k as f64 * 0.618_034).fract();
    base * (0.7 + 0.6 * f)
}

impl Game {
    /// Once a frame on the way in: a car if one is due, the door's breath.
    pub(crate) fn tunnel_sound(&mut self, clock: f64, feet: [f64; 3], open_door: Option<f64>) {
        if self.ac.is_none() {
            return;
        }
        let Some(mut s) = self.tunnel.as_ref().map(|t| t.sound) else { return };
        if s.next_car == 0.0 {
            s.next_car = 2.0;
        }
        if clock >= s.next_car && clock < CARS_STOP {
            s.cars += 1;
            s.next_car = clock + car_gap(s.cars, clock);
            // fewer as it goes on, and quieter; passing a little ahead of you
            let k = 1.0 - (clock / CARS_STOP).min(1.0) * 0.4;
            let z = feet[2] - 4.0 - (s.cars % 3) as f64 * 3.0;
            self.play_from(LANE_X, CAR_Y, z, 8.0, 0.9 * k, 0.0, |g| {
                // through the glass: the tyres' hiss and the body's rush, low,
                // swelling as it comes and falling away
                g.noise_burst(2.8, 520.0, 0.7, 0.16, "lowpass", Some(260.0), 1.1, 0.0);
                g.noise_burst(1.6, 1400.0, 1.2, 0.035, "bandpass", Some(700.0), 0.7, 0.4);
                g.blip(52.0, 2.2, "sine", 0.05, Some(41.0), 0.5, 0.0);
            });
        }
        if let Some(z) = open_door
            && clock >= s.next_air
            && (z - feet[2]).abs() < AIR_FROM
        {
            s.next_air = clock + AIR_EVERY;
            // air drawn out of the passage, cold, from behind the open leaf
            self.play_from(jahamun::DOOR_X - 0.6, 1.0, z, 3.0, 0.9, 0.0, |g| {
                g.noise_burst(2.6, 240.0, 0.6, 0.12, "lowpass", Some(140.0), 0.9, 0.0);
                g.noise_burst(1.8, 900.0, 2.0, 0.02, "bandpass", Some(600.0), 0.6, 0.5);
            });
        }
        if let Some(t) = self.tunnel.as_mut() {
            t.sound = s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cars_thin_out_and_stop() {
        let (mut t, mut k, mut gaps) = (2.0, 0u32, vec![]);
        while t < CARS_STOP {
            k += 1;
            let g = car_gap(k, t);
            gaps.push((t, g));
            t += g;
        }
        let early: f64 =
            gaps.iter().filter(|(t, _)| *t < 40.0).map(|(_, g)| g).sum::<f64>() / gaps.iter().filter(|(t, _)| *t < 40.0).count() as f64;
        let late: f64 = gaps.iter().filter(|(t, _)| *t > 75.0).map(|(_, g)| g).sum::<f64>()
            / gaps.iter().filter(|(t, _)| *t > 75.0).count().max(1) as f64;
        assert!(early < 6.0 && late > 9.0, "early {early}, late {late}");
        assert!(gaps.len() > 10 && gaps.len() < 25, "{} cars", gaps.len());
    }
}
