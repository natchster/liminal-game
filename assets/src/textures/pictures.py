"""The pictures the rooms' framed photographs and paintings show
(crates/game/assets/textures/pictures.jpg), used by build.py.

Every picture is listed in sources.json under `pictures`: a CC0 file of
Wikimedia Commons (old photographs of anonymous sitters, before 1930, and
paintings, all released as CC0 by the Rijksmuseum), checked by the SHA-1
Commons publishes, cropped (`crop`, fractions of the original), narrowed
about the crop's middle to its cell's shape and laid in its cell of one
atlas (`cell`: x, y, w, h). Nothing is graded here: the game fades,
yellows and cracks each print as it paints it (`visuals::pictures`), so
two prints of one picture on two floors differ. The cells' places are
mirrored in `crates/game/src/visuals/pictures.rs` (`CELLS`)."""
import os

import numpy as np

import texlib as T


def fetch(sources, cache, key, get, sha1):
    """The original of picture `key`, fetched once into cache/pictures/ and
    checked by its SHA-1."""
    s = sources["pictures"]["files"][key]
    path = os.path.join(cache, "pictures", key + ".jpg")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    if not os.path.exists(path) or sha1(path) != s["sha1"]:
        print("fetching", s["title"])
        get(s["url"], path)
    if sha1(path) != s["sha1"]:
        raise SystemExit(f"{key}: SHA-1 {sha1(path)} is not the one Commons published ({s['sha1']})")
    return path


def cut(img, crop, w, h):
    """The crop box (fractions) narrowed about its middle to w:h, resized to w x h."""
    H, W = img.shape[:2]
    x0, y0, x1, y1 = crop[0] * W, crop[1] * H, crop[2] * W, crop[3] * H
    cw, ch = x1 - x0, y1 - y0
    if cw / ch > w / h:
        nw = ch * w / h
        x0, x1 = x0 + (cw - nw) / 2, x0 + (cw + nw) / 2
    else:
        nh = cw * h / w
        y0, y1 = y0 + (ch - nh) / 2, y0 + (ch + nh) / 2
    part = img[int(round(y0)):int(round(y1)), int(round(x0)):int(round(x1))]
    return np.clip(T.resize(part, w, h), 0.0, 1.0)


def build(sources, cache, get, sha1):
    """The atlas: every picture in its cell, the rest mid grey."""
    p = sources["pictures"]
    aw, ah = p["atlas"]["size"]
    out = np.ones((ah, aw, 3), np.float32) * np.array(p["atlas"]["background"], np.float32)
    for key, s in p["files"].items():
        img = T.load(fetch(sources, cache, key, get, sha1))
        x, y, w, h = s["cell"]
        out[y:y + h, x:x + w] = cut(img, s["crop"], w, h)
    return out
