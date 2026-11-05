"""The pictures the rooms' framed photographs, paintings, maps, plates and
prints show (crates/game/assets/textures/pictures.jpg), used by build.py.

Every picture is listed in sources.json under `pictures`: a CC0 file of
Wikimedia Commons (old photographs of anonymous sitters, before 1930,
paintings, maps and botanical and anatomical plates, released as CC0 by the
Rijksmuseum and the Cleveland Museum of Art), checked by the SHA-1 Commons
publishes, cropped (`crop`, fractions of the original), narrowed about the
crop's middle to its cell's shape and laid in its cell of one atlas (`cell`:
x, y, w, h). With `pad`, the crop is not narrowed but fitted whole in its
cell and the rest filled with the paper round it (a plate narrower than its
frame's opening). `same_as` names another picture whose file this one is
cut from (the triptych's three panels are one photograph). Nothing is
graded here: the game fades, yellows and cracks each print as it paints it
(`visuals::pictures`), so two prints of one picture on two floors differ.
The cells' places are mirrored in `crates/game/src/visuals/pictures.rs`
(`CELLS`)."""
import os

import numpy as np
from PIL import Image

import texlib as T

Image.MAX_IMAGE_PIXELS = None


def fetch(sources, cache, key, get, sha1):
    """The original of picture `key` (or of the picture it is `same_as`),
    fetched once into cache/pictures/ and checked by its SHA-1."""
    files = sources["pictures"]["files"]
    key = files[key].get("same_as", key)
    s = files[key]
    path = os.path.join(cache, "pictures", key + ".jpg")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    if not os.path.exists(path) or sha1(path) != s["sha1"]:
        print("fetching", s["title"])
        get(s["url"], path)
    if sha1(path) != s["sha1"]:
        raise SystemExit(f"{key}: SHA-1 {sha1(path)} is not the one Commons published ({s['sha1']})")
    return path


def box(W, H, crop, w, h, narrow=True):
    """The crop box (fractions) in px, narrowed about its middle to w:h."""
    x0, y0, x1, y1 = crop[0] * W, crop[1] * H, crop[2] * W, crop[3] * H
    cw, ch = x1 - x0, y1 - y0
    if narrow:
        if cw / ch > w / h:
            nw = ch * w / h
            x0, x1 = x0 + (cw - nw) / 2, x0 + (cw + nw) / 2
        else:
            nh = cw * h / w
            y0, y1 = y0 + (ch - nh) / 2, y0 + (ch + nh) / 2
    return int(round(x0)), int(round(y0)), int(round(x1)), int(round(y1))


def cut(path, crop, w, h, pad=False):
    """The picture's crop as a w x h cell: narrowed to w:h and resized, or
    (pad) fitted whole and set on the paper round it (the mean of its
    outermost 2%)."""
    im = Image.open(path)
    W, H = im.size
    b = box(W, H, crop, w, h, narrow=not pad)
    part = np.asarray(im.crop(b).convert("RGB"), dtype=np.float32) / 255.0
    if not pad:
        return np.clip(T.resize(part, w, h), 0.0, 1.0)
    ph, pw = part.shape[:2]
    k = min(w / pw, h / ph)
    fw, fh = max(1, int(round(pw * k))), max(1, int(round(ph * k)))
    fit = np.clip(T.resize(part, fw, fh), 0.0, 1.0)
    e = max(2, int(0.02 * min(pw, ph)))
    ring = np.concatenate([part[:e].reshape(-1, 3), part[-e:].reshape(-1, 3), part[:, :e].reshape(-1, 3), part[:, -e:].reshape(-1, 3)])
    out = np.ones((h, w, 3), np.float32) * np.median(ring, 0)
    ox, oy = (w - fw) // 2, (h - fh) // 2
    out[oy:oy + fh, ox:ox + fw] = fit
    return out


def build(sources, cache, get, sha1):
    """The atlas: every picture in its cell, the rest mid grey."""
    p = sources["pictures"]
    aw, ah = p["atlas"]["size"]
    out = np.ones((ah, aw, 3), np.float32) * np.array(p["atlas"]["background"], np.float32)
    for key, s in p["files"].items():
        x, y, w, h = s["cell"]
        out[y:y + h, x:x + w] = cut(fetch(sources, cache, key, get, sha1), s["crop"], w, h, s.get("pad", False))
    return out
