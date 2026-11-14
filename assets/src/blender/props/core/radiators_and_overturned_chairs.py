"""Pilot batch: the worked example for every other batch.

place_radiators       cast-iron column radiator (body, front fins, top slat)
place_overturned_chair wooden chair on its side (seat, spindle back, turned legs)
"""
from propkit import *


# ---------------------------------------------------------------- radiator --
def radiator_body():
    # ten cast-iron columns on a top and a bottom manifold, 0.70 × 0.50 × 0.10
    cols = Shape()
    n, w = 10, 0.70
    pitch = w / n
    for i in range(n):
        x = -w / 2 + pitch * (i + 0.5)
        col = rbox(pitch * 0.82, 0.44, 0.086, r=0.014, seg=2)
        cols = cols + col.move(x=x)
    manifold = lambda y: rbox(w, 0.05, 0.06, r=0.012, seg=2).move(y=y)
    feet = sum((rbox(0.05, 0.03, 0.1, r=0.008, seg=1).move(x=sx * 0.29, y=-0.235) for sx in (-1, 1)), Shape())
    return cols + manifold(0.215) + manifold(-0.2) + feet

part('Radiators__body', f_box(0.70, 0.50, 0.10), radiator_body, budget=1500)
part('Radiators__fin', f_box(0.06, 0.48, 0.02),
     lambda: taper(rbox(0.06, 0.48, 0.02, r=0.008, seg=2), top=0.9, along='y', axes='x'))
part('Radiators__cap', f_box(0.72, 0.03, 0.12),
     lambda: rbox(0.72, 0.03, 0.12, r=0.011, seg=3))


# ------------------------------------------------------------------ chair --
def chair_seat():
    # a slightly dished plank seat with eased edges
    s = slab(0.42, 0.42, 0.04, 8, 8).rot(x=-math.pi / 2)
    def dish(p):
        if p.y > 0:
            r2 = (p.x / 0.21) ** 2 + (p.z / 0.21) ** 2
            p.y -= 0.007 * max(0.0, 1 - r2)
        return p
    return bevel(s.warp(dish), 0.008, seg=2, angle=40)

def chair_back():
    # two stiles, a curved top rail, a lower rail and three spindles
    w, h, d = 0.42, 0.50, 0.04
    stile = lambda sx: rbox(0.04, h, d, r=0.008).move(x=sx * (w / 2 - 0.02))
    top = bend(rbox(w - 0.02, 0.09, 0.028, r=0.008, seg=2), 0.0, 'y', 'x').move(y=h / 2 - 0.045)
    low = rbox(w - 0.06, 0.035, 0.024, r=0.006).move(y=-h / 2 + 0.09)
    spindles = Shape()
    for sx in (-0.09, 0, 0.09):
        spindles = spindles + lathe([(0.0, -0.17), (0.009, -0.17), (0.011, -0.08), (0.008, 0.0), (0.011, 0.08), (0.009, 0.17), (0.0, 0.17)], seg=10).move(x=sx, y=0.04)
    return stile(-1) + stile(1) + top + low + spindles

def chair_leg():
    # a turned leg: square block at the top where it meets the seat, then a taper
    block = rbox(0.04, 0.07, 0.04, r=0.004).move(y=0.165)
    turned = lathe([(0.0, -0.2), (0.012, -0.2), (0.014, -0.185), (0.013, -0.06), (0.016, -0.04), (0.017, 0.06),
                    (0.019, 0.1), (0.019, 0.13), (0.0, 0.13)], seg=14)
    return block + turned

part('OverturnedChair__seat', f_box(0.42, 0.04, 0.42), chair_seat)
part('OverturnedChair__back', f_box(0.42, 0.50, 0.04), chair_back)
part('OverturnedChair__leg', f_box(0.04, 0.40, 0.04), chair_leg)
