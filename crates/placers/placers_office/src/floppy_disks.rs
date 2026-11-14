//! `place_floppy_disks`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Floppy disk 3.5": 7% chance per level. Shell w/ metal shutter + label strip.
pub(crate) fn place_floppy_disks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let shell_colors = [0x202020, 0x4060c0, 0xc02020, 0x606060, 0xf0e8d0, 0x408060];
    let shell_mat = lambert!({ color: choice(&shell_colors) });
    let shutter_mat = lambert!({ color: 0xa0a0a0 });
    let label_mat = lambert!({ color: 0xf8f4e8 });
    let floppy = group();
    let shell = mesh(pg("FloppyDisks__shell", box_geo!(0.085, 0.005, 0.088)), &shell_mat);
    shell.position().y = 0.0025;
    floppy.add(&shell);
    let shutter = mesh(pg("FloppyDisks__shutter", box_geo!(0.045, 0.002, 0.030)), &shutter_mat);
    shutter.position().set(0, 0.006, -0.028);
    floppy.add(&shutter);
    let notch = mesh(box_geo!(0.015, 0.001, 0.008), lambert!({ color: 0x201818 }));
    notch.position().set(0.015, 0.007, -0.028);
    floppy.add(&notch);
    let label = mesh(plane_geo!(0.055, 0.028), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().set(0, 0.006, 0.012);
    floppy.add(&label);
    let tab_hole = mesh(box_geo!(0.005, 0.001, 0.005), lambert!({ color: 0x100808 }));
    tab_hole.position().set(-0.038, 0.006, 0.038);
    floppy.add(&tab_hole);
    let (px, pz) = jitter(g, x, z, 2.4);
    floppy.position().set(px, 0, pz);
    floppy.rotation().y = rnd() * PI * 2.0;
    g.world.add(&floppy);
}
