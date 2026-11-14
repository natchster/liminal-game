//! Glass that is not clear: frosted panes and smudged mirrors.
use game::prelude::*;

/// A pane of glass (`Patch::Glass`): `room` how bright the room reads in
/// it, `back` the light behind it (linear), `milk` how much of that light
/// its frosting scatters, `fresnel` its reflection square on (0.86 a
/// silvered one); its dirt from the map's grey, in `dirt`'s colour
pub fn glass_mat(map: Tex, name: &str, room: f32, back: [f32; 3], milk: f32, fresnel: f32, dirt: [f32; 4], shine: f64) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x2a2a2a, shininess: shine, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map);
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = name.into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = room;
        f[1] = back[0];
        f[2] = back[1];
        f[3] = back[2];
        // no breath's mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        f[8] = dirt[0];
        f[9] = dirt[1];
        f[10] = dirt[2];
        f[11] = dirt[3];
        f[12] = milk;
        f[13] = fresnel;
    }
    m
}
