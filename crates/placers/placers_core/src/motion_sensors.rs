//! `place_motion_sensors`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::shuffle;

// Motion sensors: 30% chance per level, 1-2 ceiling-mounted domes with LED.
pub(crate) fn place_motion_sensors(g: &mut Game, _dist: &[i32]) {
    g.pp.motion_sensor_positions.clear(); // reset per level
    if rnd() < 0.70 {
        return;
    } // 30% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let dome_mat = lambert!({ color: 0xe0d8c8 });
    // Per-LED material created inline for the blink animation.
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        let sensor = group();
        // Base disc (fits flush against ceiling).
        let base = mesh(pg("MotionSensors__base", cyl_geo!(0.08, 0.08, 0.02, 10)), &dome_mat);
        base.position().y = 0.0;
        sensor.add(&base);
        // Dome (hemisphere).
        let dome = mesh(pg("MotionSensors__lens", sphere_geo!(0.06, 10, 6, 0, PI * 2.0, 0, PI / 2.0)), &dome_mat);
        dome.position().y = -0.01;
        dome.rotation().x = PI; // flip so it points down
        sensor.add(&dome);
        // Blinking LED, colour per weather.
        let led_color = led_color_by_weather(g.weather_key());
        let led_mat_own = basic_mat!({ color: led_color, transparent: true, opacity: 1 });
        let led = mesh(box_geo!(0.008, 0.008, 0.008), &led_mat_own);
        led.position().set(0.04, -0.06, 0);
        sensor.add(&led);
        let blink_phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            // 2 s period sin wave: opacity = 0.7 + 0.3 * sin.
            led_mat_own.m().opacity = 0.7 + 0.3 * (t * PI + blink_phase).sin();
        });
        let ox = (rnd() - 0.5) * 1.4;
        let oz = (rnd() - 0.5) * 1.4;
        sensor.position().set(g.wx(x) + ox, WALL_H - 0.01, g.wz(z) + oz); // base top on the ceiling
        g.world.add(&sensor);
        // Track for proximity-click audio events.
        let p = sensor.pos();
        g.pp.motion_sensor_positions.push(v3(p.x, p.y, p.z));
        i += 1;
    }
}
