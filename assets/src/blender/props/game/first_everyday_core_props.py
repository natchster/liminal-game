"""first_everyday_core_props (was r01): the props remodelled at their real sizes (docs/design/PROPS.md), first the
everyday ones of the core props.

No map file: the game builds each prop from these parts by name, placed as
modelled (a scale of one, `prop_part_as_modelled`). Each part's frame is
given with it.

The wet floor sign: an A-frame of two moulded polypropylene panels hinged
at the top, at the size of the common 2-sided "caution wet floor" sign
(Rubbermaid FG611277, 11 x 12 x 26 in: 0.28 x 0.30 x 0.66 m standing). One
panel is modelled standing straight up, x across its face (its right seen
from the front), y up from the floor, z out of its printed face; its hinge
line runs along x at the top (y = PANEL_H), at z = 0 on the panel's back.
The game turns two of them about that line, splayed.

wetSignPanel  a panel: the face (3 mm, its print panel a shade proud of it),
              the moulded rim on its back, two ribs, the handle slot
              through the head, the two feet at its foot
wetSignHinge  the hinge pin and the four knuckles along it (both panels'),
              x across, centred at the hinge line
"""
from propkit import *
import propkit

# ---------------------------------------------------------------- sizes ---
PANEL_W0, PANEL_W1, PANEL_H = 0.290, 0.262, 0.655   # foot width, head width, length
FACE_T = 0.0035                                     # the face's wall
RIM_D, RIM_W = 0.016, 0.005                         # the rim on its back: depth, wall
SLOT = (0.112, 0.030, 0.592)                        # the handle slot: w, h, centre y


def panel_outline(inset=0.0):
    """The panel's outline (x, y), a little narrower at its head, its head
    rounded; inset by `inset` all round."""
    w0, w1, h = PANEL_W0 / 2 - inset, PANEL_W1 / 2 - inset, PANEL_H - inset
    y0 = inset
    pts = [(-w0, y0), (w0, y0), (w1, h - 0.045), (w1 - 0.035, h), (-w1 + 0.035, h), (-w1, h - 0.045)]
    return round_outline(pts, 0.012, seg=3)


def wet_sign_panel():
    face = prism(panel_outline(), FACE_T, bevel_r=0.0012)
    face.move(0, 0, -FACE_T / 2)                                # its front at z = 0... then turned below
    # the rim: the outline less itself inset, the depth of the moulding
    rim = cut(prism(panel_outline(), RIM_D), prism(panel_outline(RIM_W), RIM_D * 1.5))
    rim.move(0, 0, -FACE_T - RIM_D / 2)
    # two ribs down the back, and one across under the slot
    ribs = [box(0.004, PANEL_H - 0.12, RIM_D * 0.8).move(sx * 0.055, PANEL_H / 2 - 0.02, -FACE_T - RIM_D * 0.4) for sx in (-1, 1)]
    ribs.append(box(PANEL_W1 - 0.02, 0.004, RIM_D * 0.8).move(0, SLOT[2] - 0.04, -FACE_T - RIM_D * 0.4))
    # the print panel: a shallow raised field where the label goes
    field = prism(round_outline([(-0.115, 0.055), (0.115, 0.055), (0.108, 0.53), (-0.108, 0.53)], 0.01, seg=2), 0.0008)
    field.move(0, 0, 0.0004)
    s = face + rim + sum(ribs) + field
    sw, sh, sy = SLOT
    slot = rbox(sw, sh, 0.1, r=sh / 2 - 0.001, seg=4).move(0, sy, -0.02)
    s = cut(s, slot)
    # the feet: two pads moulded on the foot, 4 mm proud of the face's line
    feet = [rbox(0.05, 0.012, RIM_D + FACE_T + 0.004, r=0.003, seg=1).move(sx * 0.105, 0.006, -(RIM_D + FACE_T) / 2) for sx in (-1, 1)]
    s = s + sum(feet)
    # its back face to z = 0 at the hinge: the panel's printed face is +z
    return s.move(0, 0, RIM_D + FACE_T)


def wet_sign_hinge():
    pin = cyl(0.0035, h=PANEL_W1 - 0.07, seg=6, bevel_r=0).rot(0, 0, math.pi / 2)
    knuckles = [cyl(0.008, h=0.03, seg=8, bevel_r=0.0008, bevel_seg=1).rot(0, 0, math.pi / 2).move(x, 0, 0) for x in (-0.09, -0.03, 0.03, 0.09)]
    return pin + sum(knuckles)


# ---------------------------------------------------------------- parts ---
def auto(name, build, budget):
    """Register a part whose frame is its own bounds: the game places it as
    built, at a scale of one."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, f_bounds(tuple(lo), tuple(hi), 'box'), lambda: shape, budget=budget)


auto('Real__wetSignPanel', wet_sign_panel, 700)
auto('Real__wetSignHinge', wet_sign_hinge, 240)
