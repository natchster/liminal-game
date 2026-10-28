"""Build the Building's photographed surfaces (crates/game/assets/textures/)
from the CC0 photographs listed in sources.json.

    python3 assets/src/textures/build.py [--write] [--only NAME,...]

Downloads the photographs into target/texture-sources/ (once; each is checked
against its SHA-1), builds every texture into target/textures/ with a 3x3
tiled preview beside it, and compares each with the committed file. With
--write it replaces the committed files (then `cargo xtask assets textures
--write` refreshes their fingerprints in MANIFEST.json; that command also runs
this script). Needs Pillow, NumPy and SciPy (`pip install pillow numpy scipy`).

Every texture is seamless both ways and graded to the mean colour of the
canvas-drawn surface it replaced, so the floors keep their light. The game
reads them as sRGB (no colour profile is written).
"""
import hashlib
import json
import os
import re
import sys
import urllib.parse
import urllib.request
import zipfile

import numpy as np
from scipy import ndimage

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import texlib as T  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CACHE = os.path.join(ROOT, "target", "texture-sources")
BUILD = os.path.join(ROOT, "target", "textures")
DEST = os.path.join(ROOT, "crates", "game", "assets", "textures")
UA = {"User-Agent": "OpenCarole-texture-build/1.0 (assets/src/textures/build.py)"}
SOURCES = json.load(open(os.path.join(HERE, "sources.json")))
QUALITY = 90


def sha1(path):
    return hashlib.sha1(open(path, "rb").read()).hexdigest()


def fetch(url, path):
    req = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(req, timeout=300) as r, open(path + ".part", "wb") as f:
        f.write(r.read())
    os.replace(path + ".part", path)


def commons(key):
    s = SOURCES["commons"][key]
    path = os.path.join(CACHE, key + ".jpg")
    if not os.path.exists(path) or sha1(path) != s["sha1"]:
        print("fetching", s["title"])
        fetch(s["url"], path)
    if sha1(path) != s["sha1"]:
        raise SystemExit(f"{key}: SHA-1 {sha1(path)} is not the one Commons published ({s['sha1']})")
    return path


def panelka(name):
    """A file of the Panelka pack (the zip is fetched from itch.io the way its
    download button does: the page's CSRF token, then the file's URL)."""
    p = SOURCES["panelka"]
    out = os.path.join(CACHE, "panelka", name)
    if os.path.exists(out) and sha1(out) == p["files"][name]:
        return out
    z = os.path.join(CACHE, "PanelkaPack.zip")
    if not os.path.exists(z) or sha1(z) != p["zip_sha1"]:
        print("fetching PanelkaPack.zip (173 MB) from itch.io")
        import http.cookiejar
        jar = http.cookiejar.CookieJar()
        op = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
        op.addheaders = [("User-Agent", "Mozilla/5.0")]
        page = op.open("https://kureca.itch.io/panelka-pack", timeout=60).read().decode("utf-8", "replace")
        tok = re.search(r'name="csrf_token" value="([^"]+)"', page).group(1)
        upload = re.search(r'data-upload_id="(\d+)"', page).group(1)
        body = urllib.parse.urlencode({"csrf_token": tok}).encode()
        r = op.open(f"https://kureca.itch.io/panelka-pack/file/{upload}?source=view_game&as_props=1&after_download_lightbox=true", body, timeout=60)
        fetch(json.load(r)["url"], z)
        if sha1(z) != p["zip_sha1"]:
            raise SystemExit(f"PanelkaPack.zip: SHA-1 {sha1(z)}, not {p['zip_sha1']}: the pack has changed; check its files")
    with zipfile.ZipFile(z, metadata_encoding="cp866") as zf:
        os.makedirs(os.path.dirname(out), exist_ok=True)
        open(out, "wb").write(zf.read(name))
    if sha1(out) != p["files"][name]:
        raise SystemExit(f"{name}: SHA-1 {sha1(out)}, not {p['files'][name]}")
    return out


# ------------------------------------------------------------------ panel ---

def panel():
    """The Well's facade: twelve precast panels a tile wide and a storey high
    (4 m x 3 m each, 16 m x 9 m a repeat), each with its window, from five
    panels photographed on post-Soviet blocks (some turned over left for
    right). Each photograph is resampled piecewise so its window's frame
    lands on the same 1.7 m x 1.5 m in the middle of the panel (where the
    lit windows are laid) and its joints on the panel's edges; the glass is
    darkened (nothing is behind it)."""
    # per photograph: the panel's edges, the window's outer frame, the glass's darkening
    P = {
        1: ((2, 2, 510, 525), (128, 95, 378, 360), 0.3),
        2: ((2, 2, 510, 547), (122, 100, 380, 388), 0.85),
        4: ((2, 2, 492, 500), (122, 82, 362, 355), 0.8),
        5: ((2, 2, 473, 494), (126, 78, 351, 328), 0.85),
        6: ((2, 2, 481, 492), (126, 83, 361, 335), 0.75),
    }
    SW, SH = 512, 384
    WIN = (147, 96, 365, 288)

    def one(n, mirror):
        img = T.load(panelka(f"Panelka/Panels/Panel{n}.png"))
        (px0, py0, px1, py1), (wx0, wy0, wx1, wy1), glass = P[n]
        g = np.ones(img.shape[:2], np.float32)
        ix, iy = int((wx1 - wx0) * 0.09), int((wy1 - wy0) * 0.09)
        g[wy0 + iy:wy1 - iy, wx0 + ix:wx1 - ix] = glass
        img = T.to_srgb(T.to_lin(img) * ndimage.gaussian_filter(g, 1.2)[..., None])
        out = T.piecewise(img, [px0, wx0, wx1, px1], [0, WIN[0], WIN[2], SW], [py0, wy0, wy1, py1], [0, WIN[1], WIN[3], SH], SW, SH)
        return out[:, ::-1] if mirror else out

    layout = [[(1, 0), (5, 0), (2, 0), (6, 0)], [(4, 0), (6, 1), (1, 1), (5, 1)], [(2, 1), (4, 1), (5, 0), (1, 0)]]
    atlas = np.concatenate([np.concatenate([one(n, m) for n, m in r], 1) for r in layout], 0)
    return T.grade(atlas, (100, 98, 91), sat=0.8, warm=0.35)


# ----------------------------------------------------------------- ashlar ---

def ashlar():
    """The Stair Halls' wall: the rock-faced granite of Tverskaya Street 11,
    Moscow (1949), rectified, cut into its blocks along their joints, and laid
    again in running bond: 8 courses of 0.75 m, 8 m x 6 m a repeat, every
    block a photographed one, with a recess at every joint."""
    img = T.load(commons("moscow_rustication"))
    # two joints (the course lines four courses apart) at two places along the wall
    quad = [(2550, 410), (3450, 270), (3450, 1880), (2550, 1795)]
    Hm = T.homography([(0, 0), (1100, 0), (1100, 1520), (0, 1520)], quad)
    stretch = 1.2  # the blocks' true length for their height

    def rect(x0, y0, x1, y1):
        w, h = int((x1 - x0) * stretch), int(y1 - y0)
        ys, xs = np.mgrid[0:h, 0:w].astype(np.float64)
        p = Hm @ np.stack([(xs / stretch + x0).ravel(), (ys + y0).ravel(), np.ones(xs.size)])
        return T.sample(img, (p[0] / p[2]).reshape(h, w), (p[1] / p[2]).reshape(h, w))

    # the rectified wall's courses and the unshadowed, unhidden blocks in them
    # (rectified px, from (-700, -400) of the rectangle above)
    OX, OY = -700, -400
    courses = {1: (40, 427), 2: (427, 802), 3: (802, 1170), 4: (1170, 1545), 5: (1545, 1922), 6: (1922, 2300)}
    blocks = [(1, 420, 1070), (1, 1070, 1440), (2, 830, 1300), (2, 1300, 1800), (3, 985, 1335), (3, 1335, 1800),
              (4, 760, 1210), (4, 1210, 1800), (5, 960, 1330), (5, 1330, 1740), (6, 700, 1150), (6, 1150, 1690)]

    def refine(c, x):
        """The darkest column (the joint) within 30 px of x in course c."""
        a, b = courses[c]
        s = rect(OX + x - 40, OY + a + (b - a) * 0.15, OX + x + 40, OY + a + (b - a) * 0.85)
        prof = ndimage.gaussian_filter1d(T.lum(T.to_lin(s)).mean(0), 3)
        m = int(10 * stretch)
        return x - 40 + (int(np.argmin(prof[m:-m])) + m) / stretch

    lib = []
    for c, x0, x1 in blocks:
        a, b = courses[c]
        lib.append(rect(OX + (refine(c, x0) if x0 > 450 else x0), OY + a, OX + (refine(c, x1) if x1 < 1750 else x1), OY + b))
    # even the light from block to block halfway (the wall turns away from the sun)
    means = [T.lum(T.to_lin(b)).mean() for b in lib]
    lib = [T.to_srgb(T.to_lin(b) * (np.mean(means) / m) ** 0.5) for b, m in zip(lib, means)]

    PPM, ch, W, NC = 256, 192, 2048, 8
    scale = ch / 376.0
    rng = np.random.default_rng(1953)
    rows, prev, used = [], [], [0] * len(lib)
    for _ in range(NC):
        best = None
        for _attempt in range(3000):
            seq, total = [], 0.0
            while total < W * 0.93:
                w8 = np.array([1.0 / (1 + used[j] + 3 * seq.count(j)) ** 2 for j in range(len(lib))])
                i = int(rng.choice(len(lib), p=w8 / w8.sum()))
                if seq and i == seq[-1]:
                    continue
                seq.append(i)
                total += lib[i].shape[1] * scale
            f = W / total
            if not 0.88 < f < 1.12:
                continue
            off = int(rng.integers(W))
            joints, x = [], 0.0
            for i in seq:
                joints.append((x + off) % W)
                x += lib[i].shape[1] * scale * f
            # running bond: no joint within 0.2 m of one in the course below
            gap = min([min(abs(j - p), W - abs(j - p)) for j in joints for p in prev] or [W])
            if gap > 0.2 * PPM and (best is None or abs(f - 1) < abs(best[1] - 1)):
                best = (seq, f, off, joints)
        seq, f, off, prev = best
        for i in seq:
            used[i] += 1
        widths = [int(round(lib[i].shape[1] * scale * f)) for i in seq]
        widths[-1] += W - sum(widths)
        row = np.concatenate([T.resize(lib[i], w, ch) for i, w in zip(seq, widths)], 1)
        # a dark recess at every joint hides where two photographed blocks meet
        cuts = np.cumsum([0] + widths[:-1])
        xs = np.arange(W)[None, :]
        d = np.min([np.minimum(np.abs(xs - c), W - np.abs(xs - c)) for c in cuts], axis=0)
        ys = np.arange(ch)[:, None]
        dd = np.minimum(np.broadcast_to(d, (ch, W)), np.broadcast_to(np.minimum(ys, ch - 1 - ys), (ch, W))).astype(np.float32)
        row = T.to_srgb(T.to_lin(row) * (1 - 0.6 * np.exp(-(dd / 3.0) ** 2))[..., None])
        rows.append(np.roll(row, off, axis=1))
    return T.grade(np.concatenate(rows, 0), (101, 96, 87), sat=0.6, warm=0.25)


# --------------------------------------------------------------- concrete ---

def poured():
    """Poured concrete (the Well's parapets, the Works' walls and housings): a
    whitewashed Soviet precast fence, its lime flaking off the grey; the four
    fields between its ribs quilted into 4 m x 4 m, with slow stains over metres."""
    img = T.load(panelka("Panelka/Fences/FenceConcrete3diffuse.png"))
    k = 256.0 / (img.shape[1] / 4.0)  # the fence is 4 m long
    small = T.resize(img, int(img.shape[1] * k), int(img.shape[0] * k))
    fields = [(180, 72, 947, 1440), (947, 72, 1774, 1440), (1774, 72, 2589, 1440), (2589, 72, 3428, 1050)]
    pool = []
    for x0, y0, x1, y1 in fields:
        m = 60
        pool.append(T.delight(small[int((y0 + m) * k):int((y1 - m) * k), int((x0 + m) * k):int((x1 - m) * k)], 90, 0.6))
    tex = T.quilt(pool, 1024, 1024, 150, 36, np.random.default_rng(1961), tries=200, tol=0.6)
    tex = T.to_srgb(T.to_lin(tex) * (1 + 0.07 * T.smooth_noise(16, 1024, np.random.default_rng(7)))[..., None])
    return T.grade(tex, (122, 118, 109), sat=0.5, warm=0.6)


def formwork():
    """The Works' walls and housings: bare grey concrete cast against steel
    forms (the four fields of a Soviet precast fence, its pores and rust
    drips, quilted into 4 m x 4 m), with the forms' own marks laid over it:
    panels 2 m x 1.33 m, each pour a little its own tone, their seams a dark
    line with a fin of grout, and tie holes 0.5 m in from each panel's edges,
    most plugged with mortar, some open and bleeding rust."""
    img = T.load(panelka("Panelka/Fences/FenceConcrete2diffuse.png"))
    k = 256.0 / (img.shape[1] / 4.0)  # the fence is 4 m long
    small = T.resize(img, int(img.shape[1] * k), int(img.shape[0] * k))
    fields = [(165, 70, 862, 1225), (920, 70, 1598, 1225), (1665, 70, 2352, 1225), (2417, 70, 3132, 1225)]
    pool = []
    for x0, y0, x1, y1 in fields:
        m = 40
        pool.append(T.delight(small[int((y0 + m) * k):int((y1 - m) * k), int((x0 + m) * k):int((x1 - m) * k)], 90, 0.6))
    rng = np.random.default_rng(1966)
    S = 1024
    tex = T.quilt(pool, S, S, 110, 28, rng, tries=200, tol=0.5)
    lin = T.to_lin(tex)
    yy, xx = np.mgrid[0:S, 0:S].astype(np.float32)
    PW, PH = S // 2, S // 3
    # each form panel a pour of its own
    panel = (yy // PH).clip(0, 2).astype(int) * 2 + (xx // PW).astype(int)
    lin *= np.array([rng.uniform(0.93, 1.07) for _ in range(6)], np.float32)[panel][..., None]
    lin *= (1 + 0.06 * T.smooth_noise(16, S, rng))[..., None]
    # the seams: a dark line, a light fin of grout beside it
    rough = T.smooth_noise(32, S, rng)
    dx = np.minimum(xx % PW, PW - xx % PW)
    dy = np.minimum(yy % PH, PH - yy % PH)
    d = np.minimum(dx, dy) + 0.8 * rough
    lin *= (1 - 0.45 * np.exp(-(np.clip(d, 0, None) / 1.3) ** 2) + 0.12 * np.exp(-((d - 3.5) / 1.2) ** 2))[..., None]
    # the tie holes
    for py in range(3):
        for px in range(2):
            for ty in (0.25, 0.75):
                for tx in (0.25, 0.75):
                    cx, cy = (px + tx) * PW + rng.uniform(-3, 3), (py + ty) * PH + rng.uniform(-3, 3)
                    r = np.hypot(xx - cx, yy - cy)
                    if rng.random() < 0.5:  # plugged: a ring of paler mortar, a dimple
                        lin *= (1 + 0.25 * np.exp(-((r - 6) / 2.0) ** 2) - 0.4 * np.exp(-(r / 3.0) ** 2))[..., None]
                    else:  # open, and the rust runs down from it
                        lin *= (1 - 0.9 * np.exp(-(r / 4.6) ** 4))[..., None]
                        run = rng.uniform(25, 90)
                        streak = np.exp(-((xx - cx) / 2.2) ** 2) * np.clip((yy - cy) / 4, 0, 1) * np.exp(-np.clip(yy - cy, 0, None) / run)
                        lin = lin * (1 - 0.35 * streak[..., None]) + streak[..., None] * 0.35 * np.array([0.09, 0.045, 0.02], np.float32)
    return T.grade(T.to_srgb(lin), (122, 118, 109), sat=0.5, warm=0.6)


def deck():
    """The Well's decks: a cracked, patched concrete yard photographed among
    post-Soviet blocks (8 m a repeat), its cloned top band cut off and wrapped
    again along our own seams."""
    img = T.load(panelka("Panelka/Concrete1diffuse.png"))
    img = T.periodic(img[150:150 + 1178, 0:1178], 90)
    return T.grade(T.delight(img, 200, 0.5), (112, 109, 101), sat=0.35, warm=0.7)


def factory():
    """The Works' floors: a factory slab, quilted from the less cracked parts of
    the same photographed yard concrete (at its own 147 px/m), with saw-cut
    control joints on the 4 m grid and a few old oil stains (8 m a repeat).
    (Cast-iron hex plates were tried first; a blind review preferred plain
    concrete on every view: on walks, stairs and gantries they read as a
    repeated decorative tile.)"""
    img = T.load(panelka("Panelka/Concrete1diffuse.png"))[150:150 + 1178, 0:1178]
    img = T.delight(img, 200, 0.5)
    # the cracks: much darker than the concrete round them, and long
    L = T.lum(T.to_lin(img))
    dark = (L / ndimage.gaussian_filter(L, 12)) < 0.6
    lab, n = ndimage.label(dark)
    sizes = ndimage.sum(dark, lab, range(1, n + 1))
    crack = ndimage.binary_dilation(np.isin(lab, np.flatnonzero(sizes >= 60) + 1), iterations=5)
    pool = []
    for y in range(0, img.shape[0] - 160, 80):
        for x in range(0, img.shape[1] - 160, 80):
            if crack[y:y + 160, x:x + 160].mean() < 0.04:  # hairline cracks, no craters
                pool.append(img[y:y + 160, x:x + 160])
    rng = np.random.default_rng(1974)
    S = 1176
    tex = T.quilt(pool, S, S, 100, 25, rng, tries=200, tol=0.5)
    lin = T.to_lin(tex)
    # old oil stains: soft, irregular, a few to a repeat
    stain = np.zeros((S, S), np.float32)
    yy, xx = np.mgrid[0:S, 0:S].astype(np.float32)
    rough = T.smooth_noise(24, S, rng)
    for _ in range(7):
        cx, cy, r = rng.uniform(0, S), rng.uniform(0, S), rng.uniform(30, 120)
        for dx in (-S, 0, S):
            for dy in (-S, 0, S):
                d = np.hypot(xx - cx - dx, yy - cy - dy) / r + 0.35 * rough
                stain = np.maximum(stain, np.clip(1.2 - d, 0, 1))
    lin *= (1 - 0.35 * ndimage.gaussian_filter(stain, 3))[..., None]
    # each 4 m bay was poured on its own day: its own tone
    bay = (yy >= S / 2).astype(int) * 2 + (xx >= S / 2).astype(int)
    lin *= np.array([rng.uniform(0.92, 1.08) for _ in range(4)], np.float32)[bay][..., None]
    # saw-cut joints at 0 and 4 m, their arrises a little chipped
    d = np.minimum(np.minimum(yy % (S / 2), S / 2 - yy % (S / 2)), np.minimum(xx % (S / 2), S / 2 - xx % (S / 2)))
    d = d + 1.2 * np.clip(rough, -1, 1)
    lin *= (1 - 0.6 * np.exp(-(np.clip(d, 0, None) / 1.4) ** 2) - 0.1 * np.exp(-(np.clip(d, 0, None) / 6.0) ** 2))[..., None]
    return T.grade(T.to_srgb(lin), (112, 108, 101), sat=0.35, warm=0.7)


def ribbed():
    """The Works' ceiling: ribbed precast concrete, a Soviet fence panel's grid
    of ribs and recessed fields, cut rib to rib (4 m x 1.6 m a repeat)."""
    img = T.load(panelka("Panelka/Fences/FenceConcrete2diffuse.png"))
    a = T.delight(img[33:1268, 137:3150], 300, 0.5)
    a = T.resize(a, 1024 + 32, int(round((1024 + 32) * a.shape[0] / a.shape[1])))
    return T.grade(T.periodic_y(T.periodic_x(a, 32), 24), (100, 97, 90), sat=0.45, warm=0.6)


# ------------------------------------------------------------------ stone ---

def stone():
    """The Stair Halls' floor: slabs of the dark speckled granite of Arsenalna
    station (Kyiv, 1960), 1 m x 0.5 m in running bond, each its own tone as
    on the platform. The grain is the photograph's: sampled from inside its
    slabs, stretched until it has no direction (the floor was seen at a
    slant), and quilted."""
    img = T.load(commons("arsenalna_floor"))
    samples = [(300, 4820, 1050, 5160), (30, 4600, 560, 4860), (700, 4600, 1300, 4820), (330, 4360, 760, 4530), (1000, 4980, 1400, 5184)]

    def spectrum_axes(a):
        L = T.lum(T.to_lin(a))
        P = np.abs(np.fft.fftshift(np.fft.fft2(L - ndimage.gaussian_filter(L, 6)))) ** 2
        h, w = P.shape
        fy, fx = np.mgrid[-(h // 2):h - h // 2, -(w // 2):w - w // 2]
        fy, fx = fy / h, fx / w
        r = np.hypot(fx, fy)
        m = (r > 0.02) & (r < 0.25)
        C = np.array([[np.sum(P[m] * fx[m] ** 2), np.sum(P[m] * fx[m] * fy[m])], [np.sum(P[m] * fx[m] * fy[m]), np.sum(P[m] * fy[m] ** 2)]])
        return np.linalg.eigh(C)

    pool = []
    for x0, y0, x1, y1 in samples:
        a = img[y0:y1, x0:x1]
        a = T.resize(a, a.shape[1], int(a.shape[0] * 1.8))
        ev, V = spectrum_axes(a)
        M = np.eye(2) + (np.sqrt(ev[1] / ev[0]) - 1) * np.outer(V[:, 1], V[:, 1])
        h, w = a.shape[:2]
        corners = M @ np.array([[0, w, w, 0], [0, 0, h, h]], dtype=np.float64)
        lo, hi = corners.min(1), corners.max(1)
        ow, oh = int(hi[0] - lo[0]), int(hi[1] - lo[1])
        oy, ox = np.mgrid[0:oh, 0:ow].astype(np.float64)
        q = np.linalg.inv(M) @ np.stack([ox.ravel() + lo[0], oy.ravel() + lo[1]])
        b = np.stack([ndimage.map_coordinates(a[..., c], [q[1], q[0]], order=3, mode="constant", cval=-1).reshape(oh, ow) for c in range(3)], -1)
        valid = b[..., 0] >= 0
        t = 1.0
        while t > 0.1:
            hh, hw = int(h * t / 2), int(w * t / 2)
            if valid[int(oh / 2 - hh):int(oh / 2 + hh), int(ow / 2 - hw):int(ow / 2 + hw)].all():
                break
            t -= 0.02
        a = T.delight(b[int(oh / 2 - hh):int(oh / 2 + hh), int(ow / 2 - hw):int(ow / 2 + hw)], 60, 0.8)
        pool.append(T.resize(a, int(a.shape[1] * 0.42), int(a.shape[0] * 0.42)))
    rng = np.random.default_rng(1960)
    sheet = T.to_lin(T.grade(T.quilt(pool, 1024, 1024, 56, 14, rng, tries=150, tol=0.5), (78, 78, 74), sat=0.6, warm=0.3))
    PPM = 256
    W, SW, SH = 4 * PPM, PPM, PPM // 2
    tex = np.zeros((W, W, 3), np.float32)
    for r in range(W // SH):
        off = (SW // 2) * (r % 2)
        for c in range(-1, W // SW + 1):
            x0, y0 = c * SW + off, r * SH
            tone = rng.uniform(0.8, 1.15)
            hue = np.array([1.0, 1.0 + rng.uniform(-0.04, 0.05), 1.0 + rng.uniform(-0.05, 0.03)])
            sx, sy = int(rng.integers(0, W)), int(rng.integers(0, W))
            piece = np.roll(np.roll(sheet, -sy, 0), -sx, 1)[:SH, :SW] * tone * hue
            for dx in (0, -W, W):
                a, b = max(0, x0 + dx), min(W, x0 + dx + SW)
                if a < b:
                    tex[y0:y0 + SH, a:b] = piece[:, a - (x0 + dx):b - (x0 + dx)]
    # hairline joints and a worn arris
    yy, xx = np.mgrid[0:W, 0:W]
    d = np.minimum(yy % SH, SH - 1 - yy % SH).astype(np.float32)
    xo = (xx + (SW // 2) * ((yy // SH) % 2)) % SW
    d = np.minimum(d, np.minimum(xo, SW - 1 - xo))
    tex = T.to_srgb(tex * (1 - 0.55 * np.exp(-(d / 1.3) ** 2) - 0.12 * np.exp(-(d / 5.0) ** 2))[..., None])
    return T.grade(tex, (99, 95, 86), sat=1.0, warm=0.35)


# ------------------------------------------------------------------- lit ---

def lit():
    """The Well's lit windows: four of the photographed windows as they look
    lit from inside at night (the curtains and the patterned glass glowing,
    the frames, mullions and a grille in silhouette), side by side; each
    fills the lit quad laid over its panel's glass (1.42 m x 1.32 m). Grey
    light: the game tints it warm, neutral or blue."""
    P = {2: ((2, 2, 510, 547), (122, 100, 380, 388)), 4: ((2, 2, 492, 500), (122, 82, 362, 355)),
         5: ((2, 2, 473, 494), (126, 78, 351, 328)), 6: ((2, 2, 481, 492), (126, 83, 361, 335))}
    # the frames in each crop (182 x 169 px: x0, y0, x1, y1), read off the photographs
    FRAMES = {2: [(0, 0, 182, 4), (0, 0, 3, 169), (82, 0, 100, 169), (0, 47, 82, 60), (178, 0, 182, 169)],
              4: [(80, 0, 100, 169), (0, 160, 182, 169)],
              5: [(0, 0, 182, 3), (77, 0, 100, 169), (0, 40, 77, 58), (0, 0, 2, 169)],
              6: [(0, 0, 182, 5), (74, 0, 97, 169), (0, 155, 75, 169), (0, 0, 3, 169)]}
    SW, SH, WIN = 512, 384, (147, 96, 365, 288)
    qx0, qy0, qw, qh = 256 - 91, 192 - 84, 182, 169
    cells = []
    for n in (5, 2, 6, 4):
        img = T.load(panelka(f"Panelka/Panels/Panel{n}.png"))
        (px0, py0, px1, py1), (wx0, wy0, wx1, wy1) = P[n]
        out = T.piecewise(img, [px0, wx0, wx1, px1], [0, WIN[0], WIN[2], SW], [py0, wy0, wy1, py1], [0, WIN[1], WIN[3], SH], SW, SH)
        c = out[qy0:qy0 + qh, qx0:qx0 + qw]
        L = T.lum(T.to_lin(c))
        # the curtain's folds and the glass's pattern, as the light comes through them
        d = (L - L.mean()) / (L.std() + 1e-6)
        glow = np.clip(0.84 + 0.1 * d, 0.45, 1.0)  # (in sRGB: the eye's steps)
        yy, xx = np.mgrid[0:qh, 0:qw].astype(np.float32)
        # the room's lamp: brighter high and in the middle
        glow *= 1.0 - 0.12 * (((xx - qw * 0.5) / qw) ** 2 * 2 + ((yy - qh * 0.35) / qh) ** 2 * 1.5)
        glow = T.to_lin(glow)
        frame = np.zeros((qh, qw), np.float32)
        for x0, y0, x1, y1 in FRAMES[n]:
            frame[y0:y1, x0:x1] = 1
        if n == 4:  # the grille: its blue bars
            frame = np.maximum(frame, (c[..., 2] - (c[..., 0] + c[..., 1]) / 2 > 0.08).astype(np.float32))
        frame = ndimage.gaussian_filter(frame, 0.8)
        # the frame against the light: its own colour, barely lit from the room
        sil = T.to_lin(c) * 0.12
        lin = glow[..., None] * (1 - frame[..., None]) + sil * frame[..., None]
        cells.append(T.resize(T.to_srgb(lin), 256, 240))
    return np.concatenate(cells, 1)


TEXTURES = {"panel": panel, "ashlar": ashlar, "poured": poured, "formwork": formwork, "deck": deck, "stone": stone, "factory": factory, "ribbed": ribbed, "lit": lit}


def main():
    write = "--write" in sys.argv
    only = None
    if "--only" in sys.argv:
        only = sys.argv[sys.argv.index("--only") + 1].split(",")
    os.makedirs(CACHE, exist_ok=True)
    os.makedirs(BUILD, exist_ok=True)
    from PIL import Image
    worst = 0.0
    for name, fn in TEXTURES.items():
        if only and name not in only:
            continue
        tex = fn()
        out = os.path.join(BUILD, name + ".jpg")
        T.save_jpeg(tex, out, QUALITY)
        T.save_jpeg(T.tile3(tex)[::3, ::3], os.path.join(BUILD, name + "_tiled.jpg"), 85)
        dest = os.path.join(DEST, name + ".jpg")
        if os.path.exists(dest):
            a = np.asarray(Image.open(out), dtype=np.float32)
            b = np.asarray(Image.open(dest), dtype=np.float32)
            diff = float(np.abs(a - b).mean()) if a.shape == b.shape else float("inf")
        else:
            diff = float("inf")
        worst = max(worst, diff)
        print(f"{name}: {tex.shape[1]}x{tex.shape[0]}, {os.path.getsize(out) // 1024} KB, mean sRGB {np.round(tex.reshape(-1, 3).mean(0) * 255, 1)}; differs from the committed file by {diff:.2f} levels")
        if write:
            os.makedirs(DEST, exist_ok=True)
            os.replace(out, dest)
    if not write and worst > 1.0:
        print("some textures differ from the committed ones (a different Pillow, NumPy or SciPy can move them by a level or two); look at target/textures/ and rebuild with --write")


if __name__ == "__main__":
    main()
