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
