"""doors: every door in a wall, made by the asset pipeline (docs/plan/DOORS.md;
their dossiers, assets/src/props/<door>/, hold every measurement's source).

Real doors at their real sizes, used as modelled (a scale of one), each
part in its door's own frame: x across its face (to the right, seen from
in front), y up from the floor, z out of the wall toward you. A part's
frame is its own bounds, a box's, so its surfaces map as a box's faces do
(`assets::props::uv_for`): the leaf's face takes its veneer or its paint
whole. The game puts them together (`core_props::doors`).

The exit door (assets/src/props/exit_door/): the door at the far end of an
office floor, to its stair, seen from the floor (it opens away from you:
its hinges are on the other side). Its origin is 6 cm in front of the wall
face it stands on (`World::wall_adj_spot`); nothing is behind it.

ExitDoor__leaf       a flush solid-core door faced in red-oak veneer,
                     908 x 2111 x 44 mm (a 3'0" x 7'0" opening, 3 mm at the
                     sides and 19 mm under it), its edges eased
ExitDoor__frame      its pressed-steel frame: a 51 mm face round the
                     opening, 65 mm proud of the wall, and the stop 16 mm
                     behind the face that the leaf shuts against
ExitDoor__lever      a cylindrical lever lockset: its 87 mm rose, the lever
                     returning to the leaf, the key cylinder in its hub, at
                     a 70 mm backset and 1024 mm up (the strike's line)
ExitDoor__closer     an overhead closer on the leaf's head, hinge side,
                     its parallel arm folded along the frame's head to its
                     shoe on the stop
ExitDoor__kickPlate  a stainless kick plate, 254 mm high, 1.27 mm thick,
                     its edges bevelled, eight screws
ExitSign__housing    the lit exit sign's box over the door: white
                     thermoplastic, 430 x 230 x 55 mm, its face recessed
                     for the lit panel (painted in the game)

The steel door (assets/src/props/steel_door/): a hollow-metal stair door
in the same opening and the same pressed-steel frame (ExitDoor__frame,
painted as it is), its lever, closer and kick plate the exit door's; seen
from the side that pushes (the floor's, the stair's bottom door) or the
side that pulls (the stair's top door, behind you).

SteelDoor__leaf      a flush hollow-metal leaf, 18-gauge skins: 908 x 2111
                     x 44 mm, its edges seamed and eased
SteelDoor__plaque    an engraved two-layer plastic plaque, 203 x 51 x 1.6 mm
                     (8" x 2" x 1/16"), at eye height on the leaf (its letters cut
                     through in the game)
SteelDoor__pushBar   a rim exit device across the push side: the mechanism
                     case at the latch edge, the touch bar, the end cap at
                     the hinge edge, its centreline 1011 mm up
HermeticDoor__*      the hermetic door (assets/src/props/hermetic_door/): a
                     Soviet shelter's ДУ-IV-2-sized steel door, its leaf in the
                     frame of its hinge (x from the hinge axis across it, y
                     up from the floor, z out of its outer face's side):
  __leaf             the steel leaf, 1240 x 2060 x 90 mm, its outer face plain
                     plate, its edges eased (its hatch, marks and tape lie on
                     that face: they are the game's)
  __inner            its inner face (a cell's, seen from inside): the rib grid
                     of channels, the gearbox the wheel turns, the screw rods
                     to the wedges at its head and foot
  __seal             the rubber seal round the inner face's edge
  __hinges           its two heavy hinges on the axis, their pins and straps
  __wheel            the hand wheel in its own frame (axis z, its hub's foot
                     at z = 0): a 420 mm rim, four spokes, the hub and nut
  __wheelFar         the same, lighter, for the storeys across the void
  __frame            a cell's frame in its opening (1.3 x 2.1 m through a
                     0.3 m wall; centred, z out of the walk side): the lining,
                     the face band on the walk side, the stop on the cell side
                     at the latch and the head, the threshold plate
  __faceFrame        a gallery door's frame on the block's face (no opening
                     behind it): its band 75 mm beyond the leaf, 35 mm deep
LiftDoors__*         the lift's landing doors (assets/src/props/lift_doors/), in
                     the lift's frame: x across, y up, z out, the wall's face
                     at z = -0.075 (`tail_elevators` stands it 7.5 cm off it):
  __frame            the landing entrance's stainless frame round a 1100 x
                     2100 mm opening (a 1000 kg car's, ISO 4190), 120 mm at
                     the jambs and 150 at the head, its face 16 cm out
  __panel            one of the two centre-opening panels, hairline stainless,
                     572 x 2094 x 32 mm, centred (the game sets each)
  __panelForced      the broken lift's panel that was forced: kicked in a
                     third of the way up, its foot pushed in toward the
                     shaft (it hangs from its top track; refs 7, 15)
  __sill             the extruded aluminium sill under them, its two grooves
  __callPlate        the hall call station's faceplate, 91 x 171 x 6 mm, on
                     the wall beside the frame, 1065 mm up
  __button           a 25 mm push button, centred (the game sets two)
InteriorDoor__*      the interior door (assets/src/props/interior_door/), in its
                     frame's frame: the wall's face at z = 0, x across it, y
                     up from the floor, z out into the room:
  __leaf             a hollow-core six-panel moulded door, 813 x 2032 x 35 mm
                     (2'8" x 6'8" x 1-3/8"), its panels moulded on both
                     faces as Masonite lays them out, its 54 mm bore at a
                     60 mm backset 914 mm up, its hinge mortises
  __frame            its jamb and colonial casing (57 x 17 mm), standing
                     on the wall's face round it (no opening behind it)
  __knob             a round knob on its rose, 54 mm across, 59 mm out
SteelDoor__hinges    three 4-1/2" butt hinges' knuckles at the hinge edge,
                     seen from the side that pulls (the hinge edge on the
                     right from there)
"""
from propkit import *
import propkit

# the wall's face, in the door's frame
WALL = -0.06


def auto(name, build, budget, smooth=35.0, flat=False):
    """A part as modelled, at a scale of one: its frame is its own bounds,
    a box's (its faces mapped as a box's are)."""
    shape = build()
    lo, hi = shape.bounds()
    part(name, ('box', tuple(lo), tuple(hi)), lambda: shape, smooth=smooth, budget=budget, flat=flat)


def ring_u(w_out, h_out, w_in, h_in, d, r=0.004, seg=1):
    """A door's frame seen from in front: a rectangle w_out x h_out round an
    opening w_in x h_in that runs out through its foot (its head h_out -
    h_in deep), d deep along z, centred, its edges rounded by r."""
    outer = box(w_out, h_out, d)
    top_band = h_out - h_in
    hole = box(w_in, h_in + 0.2, d + 0.1).move(y=h_out / 2 - top_band - (h_in + 0.2) / 2)
    s = cut(outer, hole)
    return bevel(s, r, seg=seg, angle=30) if r > 0 else s


# ------------------------------------------------------------- exit door --
# the opening (3'0" x 7'0"), the leaf in it, how far the frame stands proud
OPEN_W, OPEN_H = 0.914, 2.134
LEAF_W, LEAF_H, LEAF_T = 0.908, 2.111, 0.044
UNDERCUT = 0.019
FACE = 0.051
FRAME_Z = 0.005           # the frame's face
STOP_Z = -0.011           # the stop's face, 16 mm behind it
LEAF_Z = -0.015           # the leaf's face (its back 1 mm off the wall)
LEVER_X, LEVER_Y = LEAF_W / 2 - 0.070, 1.024


def exit_leaf():
    s = rbox(LEAF_W, LEAF_H, LEAF_T, r=0.002, seg=1)
    return s.move(y=UNDERCUT + LEAF_H / 2, z=LEAF_Z - LEAF_T / 2)


def exit_frame():
    # the frame's face and returns: a block 65 mm deep round the opening,
    # open at the floor, its edges rounded as pressed steel's are
    depth = FRAME_Z - WALL
    W, H = OPEN_W + 2 * FACE, OPEN_H + FACE
    face = ring_u(W, H, OPEN_W, OPEN_H, depth, r=0.0025, seg=1)
    face = face.move(y=H / 2, z=WALL + depth / 2)
    # the stop: a 13 mm lip inside the opening, its face 16 mm behind the
    # frame's (1 mm into the face block, so no two faces meet)
    sd = STOP_Z - WALL
    stop = ring_u(OPEN_W + 0.002, OPEN_H + 0.001, OPEN_W - 0.026, OPEN_H - 0.013, sd, r=0.0015, seg=1)
    stop = stop.move(y=(OPEN_H + 0.001) / 2, z=WALL + sd / 2)
    return face + stop


def exit_lever():
    x, y, z = LEVER_X, LEVER_Y, LEAF_Z
    rose = lathe([(0.0, 0.0), (0.0435, 0.0), (0.0435, 0.004), (0.039, 0.010), (0.018, 0.012), (0.0, 0.012)], seg=24)
    rose = rose.rot(x=math.pi / 2).move(x=x, y=y, z=z)
    hub = cyl(0.011, 0.011, 0.050, seg=16, bevel_r=0.002).rot(x=math.pi / 2).move(x=x, y=y, z=z + 0.012 + 0.025)
    # the lever: out from the hub, then toward the hinges, its end turned
    # back to the leaf (a return, so it catches nothing)
    lever = tube([(x, y, z + 0.052), (x - 0.010, y, z + 0.060), (x - 0.050, y, z + 0.062),
                  (x - 0.105, y, z + 0.060), (x - 0.122, y, z + 0.050), (x - 0.126, y, z + 0.030)],
                 0.0085, seg=10, smooth=2)
    # the key cylinder's face in the hub's end, its keyway a slot
    plug = cyl(0.0075, 0.0075, 0.003, seg=16).rot(x=math.pi / 2).move(x=x, y=y, z=z + 0.0635)
    return rose + hub + lever + plug


def exit_closer():
    # the closer's body on the leaf's head, hinge side (a rounded box with
    # end caps), its spindle, and the parallel arm folded along the frame's
    # head to the shoe on the stop
    top = UNDERCUT + LEAF_H
    bx0, bx1, by, bz = -LEAF_W / 2 + 0.030, -LEAF_W / 2 + 0.341, top - 0.060, LEAF_Z + 0.0285
    body = rbox(bx1 - bx0, 0.089, 0.057, r=0.012, seg=1).move(x=(bx0 + bx1) / 2, y=by, z=bz)
    caps = sum(cyl(0.040, 0.040, 0.006, seg=12).rot(z=math.pi / 2).move(x=xx, y=by, z=bz) for xx in (bx0 - 0.002, bx1 + 0.002))
    sx = bx0 + 0.070
    spindle = cyl(0.009, 0.009, 0.020, seg=12).rot(x=math.pi / 2).move(x=sx, y=by + 0.010, z=bz + 0.034)
    arm = rbox(0.300, 0.020, 0.008, r=0.003).move(x=sx + 0.140, y=by + 0.024, z=bz + 0.040)
    fore = rbox(0.200, 0.016, 0.007, r=0.003).rot(z=0.08).move(x=sx + 0.370, y=by + 0.040, z=bz + 0.030)
    shoe = rbox(0.060, 0.012, 0.030, r=0.002).move(x=sx + 0.470, y=OPEN_H - 0.006 - 0.001, z=STOP_Z + 0.012)
    return body + caps + spindle + arm + fore + shoe


def exit_kick_plate():
    w, h, t = 0.863, 0.254, 0.00127
    s = bevel(box(w, h, t), 0.001, seg=1, angle=30)
    s = s.move(y=UNDERCUT + 0.004 + h / 2, z=LEAF_Z + t / 2 + 0.0002)
    for sx in (-1, 0, 1):
        for sy in (-1, 1):
            if sx == 0 and sy == -1:
                continue
            head = cyl(0.0045, 0.0035, 0.0012, seg=6).rot(x=math.pi / 2)
            s = s + head.move(x=sx * (w / 2 - 0.020), y=UNDERCUT + 0.004 + h / 2 + sy * (h / 2 - 0.018),
                               z=LEAF_Z + t + 0.0007)
    return s


SIGN_W, SIGN_H, SIGN_D, SIGN_Y = 0.430, 0.230, 0.055, 2.470


def exit_sign_housing():
    s = rbox(SIGN_W, SIGN_H, SIGN_D, r=0.008, seg=2)
    s = cut(s, rbox(0.404, 0.204, 0.020, r=0.004, seg=1).move(z=SIGN_D / 2 + 0.004))
    return s.move(y=SIGN_Y, z=WALL + SIGN_D / 2)


auto('ExitDoor__leaf', exit_leaf, 200)
auto('ExitDoor__frame', exit_frame, 500, smooth=30.0)
auto('ExitDoor__lever', exit_lever, 1100, smooth=45.0)
auto('ExitDoor__closer', exit_closer, 900)
auto('ExitDoor__kickPlate', exit_kick_plate, 600)
auto('ExitSign__housing', exit_sign_housing, 500)


# ------------------------------------------------------------ steel door --
def steel_leaf():
    s = rbox(LEAF_W, LEAF_H, LEAF_T, r=0.0015, seg=1)
    return s.move(y=UNDERCUT + LEAF_H / 2, z=LEAF_Z - LEAF_T / 2)


PLAQUE_Y = 1.52


def steel_plaque():
    s = rbox(0.203, 0.051, 0.0016, r=0.0006, seg=1)
    return s.move(y=PLAQUE_Y, z=LEAF_Z + 0.0008)


def steel_push_bar():
    # a rim exit device (touch bar): the mechanism case at the latch edge,
    # the touch bar across toward the hinges, its end cap and bracket
    y, z = 1.011, LEAF_Z
    x1 = LEAF_W / 2 - 0.030
    case = rbox(0.203, 0.070, 0.060, r=0.010, seg=1).move(x=x1 - 0.1015, y=y, z=z + 0.030)
    bar = rbox(0.560, 0.048, 0.040, r=0.012, seg=1).move(x=x1 - 0.203 - 0.280, y=y - 0.003, z=z + 0.052)
    channel = rbox(0.600, 0.057, 0.030, r=0.006, seg=1).move(x=x1 - 0.203 - 0.300, y=y, z=z + 0.015)
    cap = rbox(0.050, 0.060, 0.055, r=0.008, seg=1).move(x=-LEAF_W / 2 + 0.060, y=y, z=z + 0.0275)
    return case + bar + channel + cap


def steel_hinges():
    out = None
    # (centrelines 10-3/8 in over the floor, 9-3/4 in under the head, the
    # third between: Steelcraft)
    for cy in (0.264, 1.075, 1.886):
        k = cyl(0.0079, 0.0079, 0.114, seg=10, bevel_r=0.0012)
        tips = cyl(0.0050, 0.0050, 0.130, seg=8)
        h = (k + tips).move(x=LEAF_W / 2 + 0.0015, y=cy, z=STOP_Z + 0.004)
        out = h if out is None else out + h
    return out


auto('SteelDoor__leaf', steel_leaf, 200)
auto('SteelDoor__plaque', steel_plaque, 200)
auto('SteelDoor__pushBar', steel_push_bar, 600)
auto('SteelDoor__hinges', steel_hinges, 700)


# --------------------------------------------------------- hermetic door --
# the leaf in its hinge's frame (the game's `door_leaf`): x from the hinge
# axis, y from the floor, z out of the outer face's side
H_LW, H_LH, H_T, H_Y0 = 1.24, 2.06, 0.09, 0.02
H_WHEEL_Y = 1.05


def hermetic_leaf():
    s = rbox(H_LW, H_LH, H_T, r=0.008, seg=1)
    return s.move(x=H_LW / 2, y=H_Y0 + H_LH / 2)


def hermetic_inner():
    zf = -H_T / 2                       # the inner face
    out = None
    def add(sh):
        nonlocal out
        out = sh if out is None else out + sh
    # the rib grid: channels round the edge, two across, two up
    c = 0.05
    d = 0.035
    for (w, h, x, y) in [(H_LW, c, H_LW / 2, H_Y0 + c / 2), (H_LW, c, H_LW / 2, H_Y0 + H_LH - c / 2),
                         (c, H_LH - 2 * c, c / 2, H_Y0 + H_LH / 2), (c, H_LH - 2 * c, H_LW - c / 2, H_Y0 + H_LH / 2),
                         (H_LW - 2 * c, c, H_LW / 2, 0.58), (H_LW - 2 * c, c, H_LW / 2, 1.58),
                         (c, H_LH - 2 * c, 0.31, H_Y0 + H_LH / 2), (c, H_LH - 2 * c, 0.93, H_Y0 + H_LH / 2)]:
        add(rbox(w, h, d, r=0.004, seg=1).move(x=x, y=y, z=zf - d / 2))
    # the gearbox the wheel turns, and its rods to the wedges at head and foot
    gz = zf - 0.03
    add(rbox(0.17, 0.17, 0.06, r=0.012, seg=1).move(x=H_LW / 2, y=H_WHEEL_Y, z=gz))
    for (y0, y1) in [(H_WHEEL_Y + 0.085, H_Y0 + H_LH - 0.09), (H_Y0 + 0.09, H_WHEEL_Y - 0.085)]:
        add(cyl(0.013, 0.013, y1 - y0, seg=8).move(x=H_LW / 2, y=(y0 + y1) / 2, z=gz))
    for y in (H_Y0 + H_LH - 0.06, H_Y0 + 0.06):
        add(rbox(0.13, 0.07, 0.05, r=0.006, seg=1).move(x=H_LW / 2, y=y, z=gz))
    return out


def hermetic_seal():
    w, h = H_LW - 0.004, H_LH - 0.004
    ring = ring_u(w, h, w - 0.06, h - 0.06, 0.012, r=0.0, seg=1)
    # (closed at the foot: a ring, not a casing)
    foot = box(w, 0.03, 0.012).move(y=-h / 2 + 0.015)
    return (ring + foot).move(x=H_LW / 2, y=H_Y0 + H_LH / 2, z=-H_T / 2 - 0.006)


def hermetic_hinges():
    out = None
    for yc in (0.45, 1.85):
        barrel = cyl(0.032, 0.032, 0.20, seg=12, bevel_r=0.004)
        pin = cyl(0.014, 0.014, 0.26, seg=8)
        strap = rbox(0.32, 0.07, 0.012, r=0.003, seg=1).move(x=0.16 + 0.02, z=-H_T / 2 - 0.006)
        h = (barrel + pin + strap).move(y=yc)
        out = h if out is None else out + h
    return out


def hermetic_wheel():
    R, r = 0.21 - 0.0125, 0.0125
    rim = torus(R, r, seg=20, tseg=5).move(z=0.048)
    hub = cyl(0.036, 0.036, 0.05, seg=10, bevel_r=0.004).rot(x=math.pi / 2).move(z=0.025)
    nut = cyl(0.016, 0.016, 0.012, seg=6).rot(x=math.pi / 2).move(z=0.056)
    spokes = None
    for k in range(4):
        a = k * math.pi / 2 + math.pi / 4
        sp = tube([(math.cos(a) * 0.03, math.sin(a) * 0.03, 0.040), (math.cos(a) * R, math.sin(a) * R, 0.048)], 0.010, seg=5)
        spokes = sp if spokes is None else spokes + sp
    return rim + hub + nut + spokes


# a cell's opening (`level::building`: CELL_DOOR x CELL_DOOR_H through
# CELL_FRONT), the frame centred in the wall, z out of the walk side
C_W, C_H, C_D = 1.3, 2.1, 0.3


def hermetic_frame():
    lining = ring_u(C_W + 0.01, C_H + 0.005, C_W - 0.01, C_H - 0.005, C_D, r=0.002, seg=1).move(y=(C_H + 0.005) / 2)
    band = ring_u(C_W + 0.15, C_H + 0.075, C_W - 0.01, C_H - 0.005, 0.008, r=0.002, seg=1)
    band = band.move(y=(C_H + 0.075) / 2, z=C_D / 2 + 0.004)
    stop_latch = rbox(0.05, C_H - 0.06, 0.02, r=0.003, seg=1).move(x=C_W / 2 - 0.03, y=(C_H - 0.06) / 2, z=-C_D / 2 + 0.01)
    stop_head = rbox(1.1, 0.05, 0.02, r=0.003, seg=1).move(x=0.08, y=C_H - 0.03, z=-C_D / 2 + 0.01)
    sill = box(C_W - 0.01, 0.006, C_D).move(y=0.003)
    return lining + band + stop_latch + stop_head + sill


def hermetic_face_frame():
    # round the leaf (the game sets its hinge 0.62 m left of the middle):
    # the band 75 mm beyond it, 35 mm off the face; the leaf laps 33 mm
    lw2 = H_LW / 2
    W, H = H_LW + 0.15, H_Y0 + H_LH + 0.075
    s = ring_u(W, H, H_LW - 0.066, H_LH - 0.066, 0.035, r=0.004, seg=1)
    foot = box(W, 0.075 + H_Y0, 0.035).move(y=-H / 2 + (0.075 + H_Y0) / 2)
    return (s + foot).move(y=H / 2, z=0.0175)


def hermetic_wheel_far():
    # the same wheel for the doors of the storeys above and below, across
    # the void: its rim coarser, its spokes bars, the hub a block
    R, r = 0.21 - 0.0125, 0.0125
    rim = torus(R, r, seg=14, tseg=4).move(z=0.048)
    hub = box(0.07, 0.07, 0.05).move(z=0.025)
    spokes = None
    for k in range(2):
        sp = box(2 * R, 0.02, 0.016).rot(z=k * math.pi / 2 + math.pi / 4).move(z=0.046)
        spokes = sp if spokes is None else spokes + sp
    return rim + hub + spokes


auto('HermeticDoor__leaf', hermetic_leaf, 200)
auto('HermeticDoor__inner', hermetic_inner, 900)
auto('HermeticDoor__seal', hermetic_seal, 200)
auto('HermeticDoor__hinges', hermetic_hinges, 600)
auto('HermeticDoor__wheel', hermetic_wheel, 480)
auto('HermeticDoor__wheelFar', hermetic_wheel_far, 200)
auto('HermeticDoor__frame', hermetic_frame, 600)
auto('HermeticDoor__faceFrame', hermetic_face_frame, 300)


# ------------------------------------------------------------ lift doors --
# the lift's frame: the wall's face at z = -0.075; the entrance's face 16 cm
# out, the panels just behind it (the shaft's look takes them there: its
# cutscene, assets/src/cutscenes/shaft.toml)
L_WALL, L_FACE = -0.075, 0.16
L_OW, L_OH = 1.10, 2.10
L_JAMB, L_HEAD = 0.12, 0.15
LIFT_CALL_X, LIFT_CALL_Y = 0.80, 1.065


def lift_frame():
    d = L_FACE - L_WALL
    W, H = L_OW + 2 * L_JAMB, L_OH + L_HEAD
    s = ring_u(W, H, L_OW, L_OH, d, r=0.003, seg=1)
    return s.move(y=H / 2, z=(L_FACE + L_WALL) / 2)


def lift_panel():
    return rbox(0.572, 2.094, 0.032, r=0.0015, seg=1)


def lift_panel_forced():
    # a grid sheet to bend: a kick's dent near the leading edge low, the
    # foot pushed back into the shaft (-z), the leading edge's foot most
    w, h, t = 0.572, 2.094, 0.032
    s = slab(w, h, t, nx=6, ny=14)

    def beat(p):
        u, v = p.x / (w / 2), p.y / (h / 2)
        p.z -= 0.014 * math.exp(-((u - 0.45) / 0.35) ** 2 - ((v + 0.45) / 0.25) ** 2)
        p.z -= 0.06 * max(0.0, -v - 0.6) ** 2 * (0.4 + 0.6 * max(0.0, u))
        return p
    return noisy(s.warp(beat), 0.0015, scale=6, seed=7)


def lift_sill():
    # the sill under the panels, flush with the floor: its nosing to the
    # landing, two grooves the panels' gibs run in
    z0, z1 = 0.110, 0.205
    out = None
    for (a, b) in [(z0, 0.120), (0.127, 0.150), (0.157, z1)]:
        st = box(L_OW, 0.008, b - a).move(y=0.004, z=(a + b) / 2)
        out = st if out is None else out + st
    under = box(L_OW, 0.004, z1 - z0).move(y=0.002, z=(z0 + z1) / 2)
    return out + under


def lift_call_plate():
    s = rbox(0.091, 0.171, 0.006, r=0.002, seg=1)
    return s.move(x=LIFT_CALL_X, y=LIFT_CALL_Y, z=L_WALL + 0.003)


def lift_button():
    return lathe([(0.0, 0.0), (0.0125, 0.0), (0.0125, 0.003), (0.0105, 0.006), (0.0, 0.0065)], seg=20).rot(x=math.pi / 2)


auto('LiftDoors__frame', lift_frame, 300)
auto('LiftDoors__panel', lift_panel, 100)
auto('LiftDoors__panelForced', lift_panel_forced, 600)
auto('LiftDoors__sill', lift_sill, 150)
auto('LiftDoors__callPlate', lift_call_plate, 150)
auto('LiftDoors__button', lift_button, 150)


# --------------------------------------------------------- interior door --
# in its frame's frame: the wall's face at z = 0, z out into the room
IN = 0.0254
I_W, I_H, I_T = 0.813, 2.032, 0.035
I_Y0, I_Z0 = 0.010, 0.005              # the leaf's foot and back
I_KNOB = (I_W / 2 - 0.060, 0.914)      # its bore: 60 mm backset, 914 mm up
I_JAMB, I_CASE_W, I_CASE_T = 0.045, 0.057, 0.017


def interior_leaf():
    s = rbox(I_W, I_H, I_T, r=0.0025, seg=1).move(y=I_H / 2)
    # the six panels (Masonite's layout at 32" wide: stiles 4.345", panels
    # 9.956", the middle stile 3.775"; from the top, rails 4.51", 3.71",
    # 6.78" and 8.76" round panels 8.54", 24.23", 24.23"), moulded on
    # both faces, their bevel soft (0.43" deep)
    sx = (4.345 + 9.956 / 2) * IN - I_W / 2
    pw = 9.956 * IN
    rows, y = [], I_H
    for (rail, ph) in [(4.51, 8.54), (3.71, 24.23), (6.78, 24.23)]:
        y -= rail * IN
        rows.append((y - ph * IN / 2, ph * IN))
        y -= ph * IN
    pockets = []
    for (cy, ph) in rows:
        for x in (sx, -sx):
            for face in (1, -1):
                pockets.append(rbox(pw, ph, 0.016, r=0.006, seg=2).move(x=x, y=cy, z=face * (I_T / 2 + 0.002)))
    s = cut(s, *pockets)
    bore = cyl(0.027, 0.027, 0.05, seg=14).rot(x=math.pi / 2).move(x=I_KNOB[0], y=I_KNOB[1])
    mortises = [box(0.006, 0.089, 0.026).move(x=-I_W / 2, y=y) for y in (0.28, 1.80)]
    s = cut(s, bore, *mortises)
    return s.move(y=I_Y0, z=I_Z0 + I_T / 2)


def interior_frame():
    # the jamb round the opening (its edge to the room), the casing on it:
    # a colonial profile, a flat face stepped down to a rounded inner bead
    ow, oh = I_W + 0.006, I_Y0 + I_H + 0.003
    jamb = ring_u(ow + 2 * 0.019, oh + 0.019, ow, oh, I_JAMB, r=0.0015, seg=1).move(y=(oh + 0.019) / 2, z=I_JAMB / 2)
    W, H = ow + 2 * I_CASE_W, oh + I_CASE_W
    back = ring_u(W, H, ow + 0.010, oh + 0.005, I_CASE_T * 0.6, r=0.002, seg=1).move(y=H / 2, z=I_JAMB + I_CASE_T * 0.3)
    face = ring_u(W - 0.016, H - 0.008, ow + 0.010, oh + 0.005, I_CASE_T, r=0.004, seg=2).move(y=(H - 0.008) / 2, z=I_JAMB + I_CASE_T / 2)
    return jamb + back + face


def interior_knob():
    x, y, z = I_KNOB[0], I_KNOB[1], I_Z0 + I_T
    rose = lathe([(0.0, 0.0), (0.0325, 0.0), (0.0325, 0.003), (0.028, 0.008), (0.012, 0.010), (0.0, 0.010)], seg=24)
    neck = lathe([(0.0, 0.0), (0.011, 0.0), (0.010, 0.020), (0.0, 0.020)], seg=16).move(y=0.008)
    knob = lathe([(0.0, 0.0), (0.012, 0.0), (0.020, 0.008), (0.026, 0.020), (0.027, 0.030), (0.024, 0.037),
                  (0.014, 0.040), (0.0, 0.041)], seg=24).move(y=0.026)
    return (rose + neck + knob).rot(x=math.pi / 2).move(x=x, y=y, z=z)


auto('InteriorDoor__leaf', interior_leaf, 1400)
auto('InteriorDoor__frame', interior_frame, 600)
auto('InteriorDoor__knob', interior_knob, 900)
