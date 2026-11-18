"""The Jahamun set's surfaces: CC0 photographs (Poly Haven, sources.json),
fetched once, checked by their SHA-1, and graded to the place as the
reference video and photographs show it (sets/jahamun/README.md).

    uv run --project sets/jahamun/pipeline python -I sets/jahamun/pipeline/textures.py [--check]

Writes sets/jahamun/assets/textures/*.jpg (albedo, sRGB, no colour profile)
and the gloss masks the live highlights read (*_gloss.jpg, grey). Every
texture is graded as an albedo (what the surface reflects), not as it looks
lit: the light is the bake's (set.py). No generative model is used.
"""
import hashlib
import json
import os
import sys
import urllib.request

import numpy as np
from PIL import Image
from scipy import ndimage

HERE = os.path.dirname(os.path.abspath(__file__))
SET = os.path.dirname(HERE)
OUT = os.path.join(SET, "assets", "textures")
CACHE = os.path.join(os.path.dirname(SET), "target", "cache", "polyhaven")
SOURCES = json.load(open(os.path.join(SET, "sources.json")))


# ------------------------------------------------------------ the sources ---

def sha1(path):
    return hashlib.sha1(open(path, "rb").read()).hexdigest()


def polyhaven(asset, part):
    """A 2K map of a Poly Haven texture, fetched once and checked by its SHA-1."""
    a = SOURCES["polyhaven"]["assets"][asset]
    name = f"{asset}_{part}_2k.jpg"
    out = os.path.join(CACHE, name)
    if not os.path.exists(out) or sha1(out) != a["files"][name]:
        os.makedirs(CACHE, exist_ok=True)
        print("fetching", name, "from Poly Haven")
        req = urllib.request.Request(f"https://dl.polyhaven.org/file/ph-assets/Textures/jpg/2k/{asset}/{name}", headers={"User-Agent": "jahamun-set"})
        open(out, "wb").write(urllib.request.urlopen(req).read())
    if sha1(out) != a["files"][name]:
        raise SystemExit(f"{name}: SHA-1 {sha1(out)}, not {a['files'][name]}: the map has changed; look at it before taking it")
    return out


def load(path):
    return np.asarray(Image.open(path).convert("RGB"), dtype=np.float32) / 255.0


def grey(path):
    return np.asarray(Image.open(path).convert("L"), dtype=np.float32) / 255.0


def to_lin(s):
    return np.where(s <= 0.04045, s / 12.92, ((s + 0.055) / 1.055) ** 2.4)


def to_srgb(lin):
    lin = np.clip(lin, 0, 1)
    return np.where(lin <= 0.0031308, lin * 12.92, 1.055 * lin ** (1 / 2.4) - 0.055)


def lum(lin):
    return lin[..., 0] * 0.2126 + lin[..., 1] * 0.7152 + lin[..., 2] * 0.0722


def save(lin, name, quality=90):
    """An albedo (linear) as a baseline JPEG: the set reads it as sRGB."""
    a = (to_srgb(lin) * 255.0 + 0.5).astype(np.uint8)
    os.makedirs(OUT, exist_ok=True)
    Image.fromarray(a).save(os.path.join(OUT, name), quality=quality, subsampling=0, optimize=True)
    print(f"wrote {name} {a.shape[1]}x{a.shape[0]}  mean albedo {np.round(lin.reshape(-1, 3).mean(0), 3)}")


def save_grey(g, name, quality=88):
    a = (np.clip(g, 0, 1) * 255.0 + 0.5).astype(np.uint8)
    Image.fromarray(a, mode="L").save(os.path.join(OUT, name), quality=quality, optimize=True)
    print(f"wrote {name} {a.shape[1]}x{a.shape[0]}  mean {g.mean():.3f}")


def resize(img, w, h):
    """Area-filtered resize of a float image (any channel count)."""
    if img.ndim == 2:
        return np.asarray(Image.fromarray(img, mode="F").resize((w, h), Image.Resampling.LANCZOS), dtype=np.float32)
    return np.stack([resize(img[..., c], w, h) for c in range(img.shape[2])], axis=-1)


def flatten(lin, sigma):
    """Take the slow light out of a photograph: divide by its blurred
    luminance (wrapped: the maps tile), keep its mean."""
    L = lum(lin)
    slow = ndimage.gaussian_filter(L, sigma, mode="wrap")
    return lin * (L.mean() / np.maximum(slow, 1e-4))[..., None]


def grade(lin, target, spread=1.0):
    """Recolour to a target albedo (linear rgb): the photograph's own shading
    (luminance about its mean, scaled by `spread`) over the target's colour."""
    L = lum(lin)
    rel = 1.0 + (L / max(L.mean(), 1e-4) - 1.0) * spread
    return np.clip(np.asarray(target, np.float32)[None, None, :] * rel[..., None], 0, 1)


def fbm(h, w, scales, seed, wrap=True):
    """Fractal value noise in 0..1 over an h x w grid (sums of blurred white
    noise), periodic when `wrap`. Seeded: the same every run."""
    rng = np.random.default_rng(seed)
    acc = np.zeros((h, w), np.float32)
    amp_total = 0.0
    for s, amp in scales:
        n = rng.standard_normal((h, w)).astype(np.float32)
        n = ndimage.gaussian_filter(n, s, mode="wrap" if wrap else "reflect")
        n /= max(n.std(), 1e-6)
        acc += amp * n
        amp_total += amp
    acc /= amp_total
    return 0.5 + 0.5 * np.tanh(acc * 0.9)


def smooth(e0, e1, x):
    t = np.clip((x - e0) / (e1 - e0), 0, 1)
    return t * t * (3 - 2 * t)


# ------------------------------------------------------------- the surfaces ---

def tiles():
    """The walkway's wall: pillow-glazed white tiles, about 20 x 9 cm in
    running bond (the video). Poly Haven's long white tiles are 4 x 9 cm
    upright, stacked: turned a quarter, they run in rows; the set lays a
    repeat every 2.8 m, so each is about 20 x 9 cm. Gloss: the glaze shines,
    the grout does not."""
    src = to_lin(load(polyhaven("long_white_tiles", "diff")))
    rough = grey(polyhaven("long_white_tiles", "rough"))
    ao = grey(polyhaven("long_white_tiles", "ao"))
    src = np.rot90(src, 1)
    rough = np.rot90(rough, 1)
    ao = np.rot90(ao, 1)
    src = flatten(src, 160)
    L = lum(src)
    # the joints, found from the photograph's profiles (stacked rows and
    # columns: a joint is a dark line right across); redrawn 6 mm wide (the
    # photograph's, scaled up with its tiles, would be 2 cm)
    H, W = L.shape

    def joints(profile):
        """Where the joints cross a profile: its period from the spectrum (a
        whole number of tiles: the photograph tiles), its phase at the
        darkest line."""
        p = profile - profile.mean()
        spec = np.abs(np.fft.rfft(p))
        spec[:4] = 0
        n = int(np.argmax(spec))
        # a harmonic can outweigh the tiles' own period: take the fundamental if it is there
        while n % 2 == 0 and spec[n // 2] > 0.3 * spec[n]:
            n //= 2
        period = len(p) / n
        sm = ndimage.gaussian_filter1d(profile, 1.0, mode="wrap")
        # the phase: the darkest offset, summed over every period
        offs = np.arange(int(period))
        score = [sum(sm[int(round(o + k * period)) % len(p)] for k in range(n)) for o in offs]
        o = offs[int(np.argmin(score))]
        return [int(round(o + k * period)) % len(p) for k in range(n)]

    rows = joints(L.mean(1))
    cols = joints(L.mean(0))
    print(f"  tiles: {len(rows)} rows, {len(cols)} columns")
    ys, xs = np.mgrid[0:H, 0:W].astype(np.float32)
    dy = np.min(np.stack([np.minimum(np.abs(ys - r), H - np.abs(ys - r)) for r in rows]), axis=0)
    dx = np.min(np.stack([np.minimum(np.abs(xs - c), W - np.abs(xs - c)) for c in cols]), axis=0)
    d = np.minimum(dx, dy)
    old = smooth(18.0, 12.0, d)
    # the glaze over the old joints: the photograph closed over its dark lines
    surface = np.stack([ndimage.grey_closing(src[..., c], size=(41, 41), mode="wrap") for c in range(3)], -1)
    src = src * (1 - old[..., None]) + surface * old[..., None]
    grout = smooth(2.0, 1.2, d)
    pillow = 1.0 - 0.07 * smooth(9.0, 2.0, d)
    glaze = np.array([0.70, 0.725, 0.715], np.float32)  # white glaze, a little cool (the video's tile reads green-grey)
    mortar = np.array([0.36, 0.36, 0.345], np.float32)  # grimy cement grout, in its recess
    base = grade(src, glaze, spread=0.55)
    base = base * pillow[..., None]
    col = base * (1 - grout[..., None]) + mortar[None, None, :] * grout[..., None]
    # old glaze: faint crazing and stains, slow, cool-grey
    stain = fbm(*L.shape, [(40, 1.0), (12, 0.5)], seed=11)
    col *= (1.0 - 0.10 * smooth(0.55, 0.9, stain))[..., None]
    save(col, "tile.jpg")
    gloss = (1 - grout) * smooth(0.9, 0.35, rough) * 0.85 + 0.05
    save_grey(gloss, "tile_gloss.jpg")


def floor():
    """The walkway's floor, a module long (6 m) and 1.8 m across: red-brown
    anti-slip paint over concrete. Poly Haven's painted concrete is green
    paint worn off a concrete floor (2 m square): its paint is recoloured to
    the walkway's, and where it is worn the concrete shows dark (wet in the
    tunnel). Over the photograph's own wear a slow mask of the place's: worn
    along the middle where feet go, dark and damp along the glass's foot
    (u = 1), cleaner by the wall. Gloss: the paint is satin, the bare
    concrete matte, the damp patches shine."""
    src = to_lin(load(polyhaven("painted_concrete", "diff")))
    rough = grey(polyhaven("painted_concrete", "rough"))
    src = flatten(src, 220)
    H, W = 3456, 1024  # 6 m x 1.8 m at 1.74 mm a pixel
    # the 2 m photograph laid over the module three times, each time moved
    # and turned (it tiles, so a wrapped crop has no seam of its own); the
    # three are joined across 40 cm with a blend, and the module's two ends
    # meet the next module's (the same texture) where the photograph does
    px = 2048 / 2.0  # source pixels a metre
    tgt_px = H / 6.0
    s = px / tgt_px
    ys, xs = np.mgrid[0:H, 0:W].astype(np.float32)

    def sample(img, oy, ox, flip):
        sy = (ys * s + oy) % img.shape[0]
        sx = ((W - 1 - xs if flip else xs) * s + ox) % img.shape[1]
        if img.ndim == 2:
            return ndimage.map_coordinates(img, [sy, sx], order=1, mode="wrap")
        return np.stack([ndimage.map_coordinates(img[..., c], [sy, sx], order=1, mode="wrap") for c in range(3)], -1)

    pieces = [(0, 0, False), (700, 1100, True), (1500, 400, False)]
    seg = H / 3.0
    weights = []
    for k in range(3):
        c = (k + 0.5) * seg
        d = np.abs(((ys[:, :1] - c + H / 2) % H) - H / 2)  # distance along, wrapped
        weights.append(smooth(seg * 0.5 + 0.2 * tgt_px, seg * 0.5 - 0.2 * tgt_px, d))
    wsum = sum(weights)
    col = np.zeros((H, W, 3), np.float32)
    rgh = np.zeros((H, W), np.float32)
    for (oy, ox, fl), wk in zip(pieces, weights):
        col += sample(src, oy, ox, fl) * (wk / wsum)[..., None]
        rgh += sample(rough, oy, ox, fl) * (wk / wsum)
    # the photograph's paint: saturated green; its bare concrete: pale and grey
    mx = col.max(-1)
    mn = col.min(-1)
    sat = (mx - mn) / np.maximum(mx, 1e-4)
    L = lum(col)
    painted = smooth(0.18, 0.42, sat) * smooth(0.30, 0.12, L)
    # only the photograph's larger chips: the small flecks are repainted
    chips = ndimage.gaussian_filter(1.0 - painted, 2.5, mode="wrap")
    painted = 1.0 - smooth(0.35, 0.7, chips) * (1.0 - painted)
    # the place's wear, slow: along the middle (u ~ 0.45) and at the glass's foot
    u = xs / (W - 1)
    v = ys / H
    slow = fbm(H // 8, W // 8, [(18, 1.0), (6, 0.6)], seed=21)
    slow = resize(slow, W, H)
    feet = np.exp(-((u - 0.47) / 0.17) ** 2) * (0.55 + 0.45 * smooth(0.35, 0.75, slow))
    foot_of_glass = smooth(0.84, 0.97, u)
    by_wall = smooth(0.10, 0.0, u)
    keep = painted * (1.0 - 0.8 * foot_of_glass)
    keep = np.clip(keep, 0, 1)
    paint = np.array([0.60, 0.165, 0.100], np.float32)  # red-brown anti-slip paint under a grey film (the video's floor against its tiles)
    concrete = np.array([0.13, 0.12, 0.11], np.float32)  # bare concrete, damp and dark
    p = grade(col, paint, spread=0.7)
    # the bare concrete is graded against its own median, not the paint's:
    # the photograph's chips are pale (primer); in the tunnel they read dark
    Lb = float(np.median(L[keep < 0.5])) if (keep < 0.5).any() else float(L.mean())
    rel_c = np.clip(1.0 + (L / max(Lb, 1e-4) - 1.0) * 0.45, 0.65, 1.35)
    c = concrete[None, None, :] * rel_c[..., None]
    out = p * keep[..., None] + c * (1 - keep[..., None])
    # the walked middle: the paint scuffed paler and greyer (the video's light lane)
    scuff = np.clip(feet * 0.8, 0, 1)[..., None]
    out = out * (1 - 0.45 * scuff) + (lum(out)[..., None] * 0.6 + 0.07) * 0.45 * scuff
    # damp: darker where water stands (along both edges, and in slow patches)
    damp = np.clip(0.6 * foot_of_glass + 0.45 * by_wall + 0.4 * smooth(0.62, 0.85, slow), 0, 1)
    out *= (1.0 - 0.4 * damp)[..., None]
    save(out, "floor.jpg")
    gloss = (0.5 * keep + 0.15 * (1 - keep)) * smooth(0.95, 0.4, rgh) + 0.35 * damp
    save_grey(np.clip(gloss, 0, 1), "floor_gloss.jpg")
    # the plaza outside: the same paint, kept (2 m square, it tiles)
    src2 = src
    mx = src2.max(-1)
    mn = src2.min(-1)
    sat2 = (mx - mn) / np.maximum(mx, 1e-4)
    keep2 = np.clip(smooth(0.15, 0.4, sat2) * 0.9 + 0.1, 0, 1)
    p2 = grade(src2, np.array([0.36, 0.085, 0.05], np.float32), spread=0.5)
    c2 = grade(src2, concrete * 1.3, spread=1.0)
    plaza = p2 * keep2[..., None] + c2 * (1 - keep2[..., None])
    save(resize(plaza, 1024, 1024), "plaza.jpg")


def vault():
    """The tunnel's lining above the tiles: concrete painted grey (the video
    reads it green-grey under the LEDs). Poly Haven's plastered wall 04, its
    slow light taken out, a repeat every 3.2 m. Reused, tinted, for the
    painted steel and the stairs' concrete (the set's materials)."""
    src = flatten(to_lin(load(polyhaven("plastered_wall_04", "diff"))), 200)
    stain = fbm(*src.shape[:2], [(60, 1.0), (15, 0.4)], seed=31)
    col = grade(src, [0.40, 0.41, 0.395], spread=0.9)
    col *= (1.0 - 0.07 * smooth(0.45, 0.85, stain))[..., None]
    save(resize(col, 1024, 1024), "vault.jpg")
    # a neutral paint (mean 1/2 of white): the base panels, the steel, signs' backs
    neutral = grade(src, [0.5, 0.5, 0.5], spread=0.8)
    save(resize(neutral, 512, 512), "paint.jpg")


def road():
    """The carriageway: Poly Haven's asphalt 02, a repeat every 3 m, graded
    to an old road's grey (asphalt's albedo about 0.08-0.12)."""
    src = flatten(to_lin(load(polyhaven("asphalt_02", "diff"))), 220)
    col = grade(src, [0.085, 0.085, 0.083], spread=1.1)
    save(resize(col, 1024, 1024), "road.jpg")


def stone():
    """The retaining walls and the portal's face: polygonal stones with thin
    joints, faced flat (the photographs of the stairs' wall). Poly Haven's
    rock wall 11 (beige, 1.91 m), graded to the grey granite of the place,
    its joints darker."""
    src = flatten(to_lin(load(polyhaven("rock_wall_11", "diff"))), 240)
    ao = grey(polyhaven("rock_wall_11", "ao"))
    col = grade(src, [0.30, 0.30, 0.29], spread=1.0)
    col *= (0.78 + 0.22 * smooth(0.35, 0.85, ao))[..., None]
    var = fbm(*src.shape[:2], [(50, 1.0), (14, 0.5)], seed=41)
    col *= (0.9 + 0.2 * var)[..., None] * np.array([1.0, 1.0, 0.985], np.float32)
    save(col, "stone.jpg")


def panel():
    """The noise barrier's foot on the walkway side, one bay (2 m) by 0.75 m:
    white painted steel, two access doors a bay (their gaps, hinges and key
    holes), the bay's seam at each post, and the walkway's grime: a dark,
    damp kick band at the bottom and splashes above it (the video)."""
    src = flatten(to_lin(load(polyhaven("plastered_wall_04", "diff"))), 200)
    W, H = 2048, 768  # 2.0 m x 0.75 m: 1.024 px a mm
    base = grade(resize(src, W, H), [0.60, 0.605, 0.60], spread=0.12)
    mm = W / 2000.0
    ys, xs = np.mgrid[0:H, 0:W].astype(np.float32)
    zmm = (H - 1 - ys) / mm  # height over the walkway, mm
    xmm = xs / mm
    shade = np.ones((H, W), np.float32)

    def line(mask, k):
        shade[mask] *= k

    for x0 in (190.0, 1190.0):  # two doors, 620 x 560 mm, 120 mm over the floor
        x1, z0, z1 = x0 + 620.0, 120.0, 680.0
        gap = ((np.abs(xmm - x0) < 3.5) | (np.abs(xmm - x1) < 3.5)) & (zmm > z0) & (zmm < z1)
        gap |= ((np.abs(zmm - z0) < 3.5) | (np.abs(zmm - z1) < 3.5)) & (xmm > x0) & (xmm < x1)
        line(gap, 0.18)
        # the shadow under the door's top edge and beside its free side
        line((np.abs(zmm - (z1 - 6.0)) < 3.0) & (xmm > x0) & (xmm < x1), 0.75)
        # the door's face a shade apart from the frame (a separate sheet)
        inside = (xmm > x0 + 2) & (xmm < x1 - 2) & (zmm > z0 + 2) & (zmm < z1 - 2)
        shade[inside] *= 0.975
        # key hole and hinges
        kh = (xmm - (x1 - 45.0)) ** 2 + (zmm - (z1 - 70.0)) ** 2 < 9.0 ** 2
        line(kh, 0.35)
        for hz in (z0 + 70.0, z1 - 70.0):
            hinge = (np.abs(xmm - (x0 + 6.0)) < 5.0) & (np.abs(zmm - hz) < 30.0)
            line(hinge, 0.8)
    seam = (xmm < 3.0) | (xmm > 1997.0)
    line(seam, 0.45)
    # a lip under the sill
    line(np.abs(zmm - 735.0) < 2.0, 0.55)
    # grime: the kick band (damp, dark), splashes, slow stains
    slow = fbm(H // 4, W // 4, [(10, 1.0), (3, 0.5)], seed=51)
    slow = resize(slow, W, H)
    fine = fbm(H, W, [(4.0, 1.0), (1.5, 0.35)], seed=52)
    kick = smooth(140.0, 40.0, zmm + 60.0 * (slow - 0.5))
    splash = smooth(260.0, 60.0, zmm) * smooth(0.6, 0.78, fine) * 0.45
    stain = smooth(0.6, 0.9, slow) * 0.06
    dirt = np.clip(0.82 * kick + 0.4 * splash + stain, 0, 0.9)
    grime = np.array([0.10, 0.095, 0.085], np.float32)
    col = base * shade[..., None]
    col = col * (1 - dirt[..., None]) + grime[None, None, :] * dirt[..., None]
    save(col, "panel.jpg")


def main():
    os.makedirs(OUT, exist_ok=True)
    tiles()
    floor()
    vault()
    road()
    stone()
    panel()


if __name__ == "__main__":
    main()
