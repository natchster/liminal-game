"""Image operations for turning photographs into seamless game textures
(used by build.py). Images are float32 sRGB arrays (h, w, 3) in 0..1;
anything about light is done in linear light."""
import numpy as np
from PIL import Image
from scipy import ndimage


def load(path):
    im = Image.open(path)
    if im.mode == "RGBA":
        bg = Image.new("RGBA", im.size, (128, 128, 128, 255))
        bg.alpha_composite(im)
        im = bg
    return np.asarray(im.convert("RGB"), dtype=np.float32) / 255.0


def save_jpeg(img, path, quality):
    """A baseline JPEG with no colour profile: the game reads its values as sRGB."""
    a = (np.clip(img, 0, 1) * 255.0 + 0.5).astype(np.uint8)
    Image.fromarray(a).save(path, quality=quality, subsampling=0, optimize=True)


def save_png(img, path):
    Image.fromarray((np.clip(img, 0, 1) * 255.0 + 0.5).astype(np.uint8)).save(path)


def to_lin(s):
    return np.where(s <= 0.04045, s / 12.92, ((s + 0.055) / 1.055) ** 2.4)


def to_srgb(l):
    l = np.clip(l, 0, 1)
    return np.where(l <= 0.0031308, l * 12.92, 1.055 * l ** (1 / 2.4) - 0.055)


def lum(lin):
    return lin[..., 0] * 0.2126 + lin[..., 1] * 0.7152 + lin[..., 2] * 0.0722


def resize(img, w, h):
    return np.stack([np.asarray(Image.fromarray(img[..., c]).resize((w, h), Image.LANCZOS)) for c in range(3)], -1).astype(np.float32)


def sample(img, sx, sy):
    """Bicubic samples of img at source pixel centres (sx, sy) (any shape)."""
    return np.stack([ndimage.map_coordinates(img[..., c], [sy - 0.5, sx - 0.5], order=3, mode="reflect") for c in range(3)], -1)


def homography(src, dst):
    """The 3x3 H with dst ~ H @ src for four point pairs."""
    A = []
    for (x, y), (u, v) in zip(src, dst):
        A.append([x, y, 1, 0, 0, 0, -u * x, -u * y, -u])
        A.append([0, 0, 0, x, y, 1, -v * x, -v * y, -v])
    _, _, vt = np.linalg.svd(np.array(A, dtype=np.float64))
    return vt[-1].reshape(3, 3) / vt[-1][-1]


def piecewise(img, xs_src, xs_dst, ys_src, ys_dst, w, h):
    """Separable piecewise-linear resampling: the source breakpoints land on the destination ones."""
    ox = np.interp(np.arange(w) + 0.5, xs_dst, xs_src)
    oy = np.interp(np.arange(h) + 0.5, ys_dst, ys_src)
    gy, gx = np.meshgrid(oy, ox, indexing="ij")
    return sample(img, gx, gy)


def delight(img, sigma, strength=1.0):
    """Divide out the light's slow change across a photograph, keeping its mean."""
    lin = to_lin(img)
    L = lum(lin) + 1e-4
    low = ndimage.gaussian_filter(L, sigma, mode="reflect")
    return to_srgb(lin * ((low.mean() / low) ** strength)[..., None])


def grade(img, target_srgb, sat=1.0, warm=0.0):
    """Scale (in linear light) so the mean luminance is the target's; keep `sat` of
    the colour; move the white balance `warm` of the way to the target's tint."""
    lin = to_lin(img)
    t = to_lin(np.array(target_srgb, dtype=np.float32) / 255.0)
    L = lum(lin)[..., None]
    lin = L + (lin - L) * sat
    m = lin.reshape(-1, 3).mean(0)
    tint = (t / t.mean()) / (m / m.mean())
    lin = lin * (1 + (tint - 1) * warm)
    lin *= lum(t[None, None, :])[0, 0] / lum(lin).mean()
    return to_srgb(lin)


def _min_cut(err):
    """The least-error path from top to bottom through err (h x w): a column per row."""
    h, w = err.shape
    E = err.copy()
    for i in range(1, h):
        p = E[i - 1]
        E[i] += np.minimum(np.minimum(np.r_[np.inf, p[:-1]], p), np.r_[p[1:], np.inf])
    path = np.zeros(h, dtype=int)
    path[-1] = int(np.argmin(E[-1]))
    for i in range(h - 2, -1, -1):
        j = path[i + 1]
        lo, hi = max(0, j - 1), min(w, j + 2)
        path[i] = lo + int(np.argmin(E[i, lo:hi]))
    return path


def seam_mask(a, b, feather=1.5):
    """For two overlapping strips, the mask that takes a left of their least-error
    cut and b right of it (1 = b)."""
    err = ((a - b) ** 2).sum(-1)
    path = _min_cut(err)
    m = (np.arange(err.shape[1])[None, :] >= path[:, None]).astype(np.float32)
    return ndimage.gaussian_filter(m, feather)[..., None] if feather > 0 else m[..., None]


def periodic_x(img, overlap):
    """Wrap left to right: the result (overlap px narrower) tiles without a seam."""
    w = img.shape[1]
    tail, head = img[:, w - overlap:], img[:, :overlap]
    m = seam_mask(tail, head)
    out = img[:, : w - overlap].copy()
    out[:, :overlap] = tail * (1 - m) + head * m
    return out


def periodic_y(img, overlap):
    return periodic_x(img.transpose(1, 0, 2), overlap).transpose(1, 0, 2)


def periodic(img, overlap):
    return periodic_y(periodic_x(img, overlap), overlap)


def quilt(pool, out_h, out_w, patch, overlap, rng, tries, tol):
    """Image quilting (Efros & Freeman 2001) from sample images, made periodic.
    Each patch is drawn by chance among the candidates within `tol` of the best
    match (the best alone favours the smoothest patches and washes the texture
    out), and cut into its neighbours along least-error seams."""
    step = patch - overlap
    H, W = out_h + overlap + patch, out_w + overlap + patch
    out = np.zeros((H, W, 3), np.float32)

    def cand():
        s = pool[rng.integers(len(pool))]
        y = rng.integers(0, s.shape[0] - patch)
        x = rng.integers(0, s.shape[1] - patch)
        return s[y:y + patch, x:x + patch]

    for y in range(0, H - patch + 1, step):
        for x in range(0, W - patch + 1, step):
            cands = [cand() for _ in range(tries if (x or y) else 1)]
            errs = np.zeros(len(cands))
            for i, c in enumerate(cands):
                if x:
                    errs[i] += ((c[:, :overlap] - out[y:y + patch, x:x + overlap]) ** 2).sum()
                if y:
                    errs[i] += ((c[:overlap, :] - out[y:y + overlap, x:x + patch]) ** 2).sum()
            ok = np.flatnonzero(errs <= errs.min() * (1 + tol) + 1e-9)
            c = cands[int(ok[rng.integers(len(ok))])].copy()
            m = np.ones((patch, patch, 1), np.float32)
            if x:
                m[:, :overlap] = np.minimum(m[:, :overlap], seam_mask(out[y:y + patch, x:x + overlap], c[:, :overlap]))
            if y:
                my = seam_mask(out[y:y + overlap, x:x + patch].transpose(1, 0, 2), c[:overlap, :].transpose(1, 0, 2))
                m[:overlap, :] = np.minimum(m[:overlap, :], my.transpose(1, 0, 2))
            out[y:y + patch, x:x + patch] = out[y:y + patch, x:x + patch] * (1 - m) + c * m
    return periodic(out[: out_h + overlap, : out_w + overlap], overlap)


def smooth_noise(n, size, rng):
    """Periodic, smooth unit noise over a size x size texture (n x n knots)."""
    f = rng.normal(size=(n, n))
    k = np.fft.fftfreq(n)
    f = np.real(np.fft.ifft2(np.fft.fft2(f) * np.exp(-(k[:, None] ** 2 + k[None, :] ** 2) * 60)))
    f = (f - f.mean()) / (f.std() + 1e-6)
    return ndimage.zoom(np.tile(f, (3, 3)), size / n, order=3)[size:2 * size, size:2 * size]


def tile3(img):
    return np.concatenate([np.concatenate([img] * 3, 1)] * 3, 0)
