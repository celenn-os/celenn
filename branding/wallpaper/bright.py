#!/usr/bin/env python3
"""The bright cosmic-blue wallpapers, drawn with numpy (5120x2880).
Usage: bright.py NAME OUT.jpg [SCALE]
NAME: dawn, aurora, layers, crescent, halo, orbits-light, haze. SCALE (default 1) renders
a smaller preview, e.g. 0.5. Output is deterministic (fixed seeds).
Needs numpy, pillow and scipy.
"""
import math
import sys

import numpy as np
from PIL import Image
from scipy.ndimage import gaussian_filter, map_coordinates

name, out = sys.argv[1], sys.argv[2]
K = float(sys.argv[3]) if len(sys.argv) > 3 else 1.0
W, H = int(5120 * K), int(2880 * K)
y, x = np.mgrid[0:H, 0:W].astype(np.float32)
u, v = x / W, y / H                     # 0..1 across and down


def hexc(s):
    return np.array([int(s[i:i + 2], 16) / 255 for i in (1, 3, 5)], np.float32)


def screen(a, b):
    return 1 - (1 - a) * (1 - np.clip(b, 0, 1))


def mix(a, b, t):
    t = np.clip(t, 0, 1)
    return a + (b - a) * (t[..., None] if np.ndim(t) == 2 else t)


def ramp(t, stops):
    """Colour ramp: stops = [(pos, '#rrggbb'), ...] -> image."""
    t = np.clip(t, 0, 1)
    out = np.zeros(t.shape + (3,), np.float32)
    pos = [p for p, _ in stops]
    for c in range(3):
        out[..., c] = np.interp(t, pos, [hexc(s)[c] for _, s in stops])
    return out


def noise(scale, rng):
    """Smooth value noise with features of about `scale` px (full res)."""
    sc = max(scale * K, 2)
    gw, gh = int(W / sc) + 3, int(H / sc) + 3
    g = rng.random((gh, gw)).astype(np.float32)
    a = np.asarray(Image.fromarray(g, mode="F").resize((int(gw * sc), int(gh * sc)), Image.BICUBIC))
    ox, oy = rng.integers(0, int(sc) + 1, 2)
    return a[oy:oy + H, ox:ox + W]


def fbm(scale, octaves, rng, gain=.5):
    tot = np.zeros((H, W), np.float32)
    amp, norm = 1.0, 0.0
    for _ in range(octaves):
        tot += amp * noise(scale, rng)
        norm += amp
        amp *= gain
        scale /= 2
        if scale * K < 2:
            break
    return tot / norm


def stars(rng, n, size=1.0, bright=1.0, mask=None):
    """A star field: n stars per full-size image, mostly faint."""
    n = int(n * K * K) + 1
    sx, sy = rng.uniform(0, W, n), rng.uniform(0, H, n)
    m = rng.pareto(2.6, n) * .25 + .15
    acc = np.zeros((H, W), np.float32)
    np.add.at(acc, (sy.astype(int), sx.astype(int)), m)
    s = gaussian_filter(acc, .7 * size * max(K, .5)) * 5 + gaussian_filter(acc, 3 * size * K) * 6
    if mask is not None:
        s *= mask
    return np.clip(s * bright, 0, 1.4)


def blur(a, s):
    return gaussian_filter(a, s * K)


def save(img, grain=2.0):
    a = np.clip(img, 0, 1) * 255
    a = a + np.random.default_rng(57).normal(0, grain, a.shape[:2]).astype(np.float32)[..., None]
    Image.fromarray(np.clip(a, 0, 255).astype(np.uint8)).save(out, quality=92, subsampling=0, optimize=True)


def sphere(cx, cy, R, L=(-.5, -.6, .62)):
    """Unit normal of a sphere in screen space: (dx, dy, nz, inside, dist)."""
    dx, dy = (x - cx) / R, (y - cy) / R
    d = np.sqrt(dx * dx + dy * dy)
    nz = np.sqrt(np.clip(1 - d * d, 0, 1))
    inside = np.clip((1 - d) * R / 1.5, 0, 1)
    Lv = np.array(L, np.float32) / np.linalg.norm(L)
    lam = dx * Lv[0] + dy * Lv[1] + nz * Lv[2]
    return dx, dy, nz, inside, d, lam


# ---------------------------------------------------------------------------
def dawn():
    """Sunrise over the limb of a blue planet seen from orbit: a luminous
    sky, the bright arc of the atmosphere, a day side with clouds."""
    rng = np.random.default_rng(3)
    cx, cy, R = .40 * W, 2.55 * H, 1.86 * H       # planet top at y = .69 H
    r = np.hypot(x - cx, y - cy)
    alt = (r - R) / H                              # height above the limb
    sunx = .63 * W
    ang = np.arctan2(y - cy, x - cx)
    sang = np.arctan2(-R, sunx - cx)
    along = (ang - sang) * R / W                   # distance along the limb from the sun
    # sky: light near the horizon, deep blue at the top
    sky = ramp(np.clip(alt, 0, None) / .69, [(0, "#bfe3ff"), (.08, "#7fb6ff"), (.30, "#3f78f0"),
                                               (.65, "#3566dc"), (1, "#2f5fd6")])
    sky = screen(sky, stars(rng, 2600, bright=.8, mask=np.clip(alt * 4 - 1.6, 0, 1) * .5)[..., None])
    sun = np.hypot((x - sunx) / W, (y - (cy - R)) / H * 1.4)
    sky = screen(sky, (np.exp(-(sun / .32) ** 2) * .55)[..., None] * hexc("#d6ecff"))
    # thin airglow layers above the limb
    for h0, w0, a0, c in [(.004, .003, .9, "#e9f6ff"), (.014, .010, .45, "#8fd3ff"), (.05, .05, .25, "#6ea4ff")]:
        band = np.exp(-((alt - h0) / w0) ** 2) * (alt > -.002)
        band *= .45 + .55 * np.exp(-(along / .55) ** 2)
        sky = screen(sky, (band * a0)[..., None] * hexc(c))
    # the planet day side: ocean blue, clouds, haze towards the limb
    depth = np.clip(-alt, 0, None)
    rng2 = np.random.default_rng(11)
    # project clouds on the curved surface: stretch them along the limb
    # sample the noise in (along the limb, foreshortened depth) coordinates
    tex = fbm(700, 8, rng2, .55)
    tex2 = fbm(220, 6, rng2, .6)
    sx = np.clip((along * .8 + .5) * W, 0, W - 1)
    sy = np.clip(np.sqrt(depth / .4) * H * .9, 0, H - 1)
    q = map_coordinates(tex, [sy, sx], order=1)
    q2 = map_coordinates(tex2, [sy, sx], order=1)
    cloud = np.clip((q * .75 + q2 * .45 - .56) * 3.4, 0, 1) ** 1.2
    ground = ramp(np.clip(depth / .31, 0, 1), [(0, "#8cc6ff"), (.07, "#3f86ea"), (.45, "#1f5bcc"), (1, "#173f9c")])
    lit = .78 + .35 * np.exp(-(along / .6) ** 2)
    ground = mix(ground * lit[..., None], hexc("#eaf4ff") * lit[..., None], cloud * (.35 + .55 * np.clip(depth / .1, 0, 1)))
    ground = mix(ground, hexc("#a9d6ff"), np.exp(-depth / .025) * .7)
    inside = np.clip(-alt * H / 1.5, 0, 1)
    img = mix(sky, ground, inside)
    # sun on the limb: core, glare, horizontal streak
    sd = np.hypot(x - sunx, (y - (cy - R) + 8 * K) * 1.0)
    glare = np.exp(-(sd / (40 * K)) ** 2) * 1.4 + np.exp(-(sd / (260 * K)) ** 2) * .5
    streak = np.exp(-((y - (cy - R)) / (9 * K)) ** 2) * np.exp(-np.abs(x - sunx) / (380 * K)) * .35
    img = screen(img, (glare + streak)[..., None] * hexc("#f4fbff"))
    return img


def aurora():
    """A soft mesh of luminous blues with a faint orbit line, very bright."""
    rng = np.random.default_rng(5)
    img = ramp(v * .6 + u * .4, [(0, "#9ec5ff"), (.5, "#4f8dff"), (1, "#3164dc")])
    blobs = [(.18, .22, .30, "#c9e2ff", .85), (.78, .30, .28, "#7fe0ff", .55), (.62, .78, .34, "#2f5ee0", .7),
             (.30, .82, .25, "#7b8cff", .55), (.92, .88, .22, "#1d3fb0", .6), (.50, .45, .22, "#a8d0ff", .45)]
    for bx, by, br, c, a in blobs:
        wx = (fbm(1800, 3, rng) - .5) * .25
        wy = (fbm(1800, 3, rng) - .5) * .25
        d = np.hypot((u - bx + wx) * 1.6, v - by + wy) / br
        img = mix(img, hexc(c), np.exp(-d ** 2) * a)
    # a single orbit drawn in light, crossing the gradient
    cx, cy, a, b, t = .55 * W, .60 * H, .60 * W, .22 * H, math.radians(-14)
    ex = ((x - cx) * math.cos(t) + (y - cy) * math.sin(t)) / a
    ey = (-(x - cx) * math.sin(t) + (y - cy) * math.cos(t)) / b
    e = np.abs(np.hypot(ex, ey) - 1) * b
    line = np.exp(-(e / (3 * K)) ** 2) * .55 + np.exp(-(e / (40 * K)) ** 2) * .18
    img = screen(img, line[..., None] * hexc("#eef6ff"))
    p = sphere(cx + a * math.cos(math.radians(-30)) * math.cos(t) - b * math.sin(math.radians(-30)) * math.sin(t),
               cy + a * math.cos(math.radians(-30)) * math.sin(t) + b * math.sin(math.radians(-30)) * math.cos(t),
               46 * K)
    dot = p[3] * (.6 + .4 * np.clip(p[5], 0, 1))
    img = mix(img, hexc("#ffffff"), dot)
    img = screen(img, (np.exp(-np.clip(p[4] - 1, 0, None) * 4) * (p[4] > 1) * .5)[..., None] * hexc("#dff0ff"))
    return img


def layers():
    """Translucent sheets of light folded over each other, from a pale sky
    at the top to deep blue at the bottom."""
    img = ramp(v, [(0, "#d4e8ff"), (1, "#7fb0ff")])
    sheets = [("#b4d4ff", "#8fbaff"), ("#8fbaff", "#6a9cff"), ("#7fa0ff", "#5a7df0"), ("#5d93ff", "#3f74ef"),
              ("#4a7ff2", "#2f5fd8"), ("#3567df", "#244dc4"), ("#3a6ae0", "#3060d6"), ("#4677ec", "#3a6be0")]
    n = len(sheets)
    for i, (c0, c1) in enumerate(sheets):
        ph = np.random.default_rng(70 + i).uniform(0, 2 * np.pi, 3)
        xs = u[0]
        base = .10 + i * .115
        edge = base + .09 * np.sin(xs * 2.2 + ph[0] + i * .5) + .035 * np.sin(xs * 5.1 + ph[1]) \
            - .10 * (xs - .5) * (1 if i % 2 else -.6)
        below = v - edge[None, :]
        # shadow cast on what lies behind, just above this sheet's edge
        sh = np.exp(np.clip(below, None, 0) / .028) * (below < 0) * .22
        img = mix(img, hexc("#1d47c4"), sh * 1.4)
        mask = np.clip(below * H / 1.2, 0, 1)
        dd = np.clip(below / .25, 0, 1)
        col = mix(hexc(c0), hexc(c1), dd)
        sheen = np.exp(-((u - (.2 + .6 * ((i * .37) % 1))) / .22) ** 2) * .18
        col = screen(col, (sheen + np.exp(-(below * H / (3 * K)) ** 2) * .55 + np.exp(-(below * H / (40 * K))) * .12)[..., None] * hexc("#ffffff"))
        img = mix(img, col, mask)
    del n
    return img


def crescent():
    """The lit edge of a huge planet in the lower left, under a luminous
    blue sky: few elements, lots of space."""
    rng = np.random.default_rng(31)
    t = np.clip(np.hypot((u - .95) * 1.2, v - .0) / 1.35, 0, 1)
    img = ramp(t, [(0, "#8fc0ff"), (.35, "#5d95ff"), (.7, "#3f74ea"), (1, "#3366dd")])
    haze = blur(fbm(900, 5, rng), 40)
    img = screen(img, (np.clip(haze - .45, 0, 1) * .5 * np.clip(1 - t * 1.3, 0, 1))[..., None] * hexc("#d6e9ff"))
    img = screen(img, stars(rng, 900, bright=.55, mask=np.clip(t * 2 - .3, 0, 1))[..., None])
    cx, cy, R = .16 * W, 1.30 * H, .95 * H
    p = sphere(cx, cy, R, L=(.75, -.55, .05))
    lam = np.clip(p[5], 0, 1)
    body = mix(hexc("#2a55c4"), hexc("#5d95ff"), lam ** 1.6)
    body = mix(body, hexc("#cfe6ff"), lam ** 6)
    img = mix(img, body, p[3])
    lit = np.clip((p[0] * .75 - p[1] * .55) / .9, 0, 1)
    out = np.clip(p[4] - 1, 0, None) * R / H
    glow = np.exp(-out / .012) * (p[4] > 1) * lit * .9 + np.exp(-out / .07) * (p[4] > 1) * lit * .35
    rim = np.exp(-(((p[4] - 1) * R) / (3 * K)) ** 2) * lit
    img = screen(img, (glow + rim)[..., None] * hexc("#bfe2ff"))
    # a small moon
    m = sphere(.74 * W, .30 * H, .035 * H, L=(-.6, .5, .4))
    img = mix(img, mix(hexc("#2a56c8"), hexc("#eaf4ff"), np.clip(m[5], 0, 1) ** .8), m[3])
    return img


def halo():
    """A ringed planet close up on the right, its rings sweeping across
    the screen."""
    rng = np.random.default_rng(33)
    img = ramp(np.clip(np.hypot(u - .75, (v - .45) * .8) / .95, 0, 1),
               [(0, "#7fb0ff"), (.4, "#4f8dff"), (1, "#3a6fe6")])
    img = screen(img, stars(rng, 1100, bright=.55)[..., None])
    cx, cy, R = .80 * W, .52 * H, .30 * H
    t = math.radians(-13)
    dx, dy = (x - cx) / R, (y - cy) / R
    ux = dx * math.cos(t) + dy * math.sin(t)
    uy = (-dx * math.sin(t) + dy * math.cos(t)) / .16
    rd = np.hypot(ux, uy)
    prof = np.random.default_rng(34).random(160)
    prof = gaussian_filter(prof, 1.2)
    tex = np.interp(rd, np.linspace(1.35, 3.1, 160), prof)
    ring = np.clip((rd - 1.35) * 25, 0, 1) * np.clip((3.1 - rd) * 25, 0, 1) * (.35 + .9 * tex)
    ring *= 1 - .85 * np.exp(-((rd - 2.25) / .035) ** 2)       # a dark gap
    ring *= np.clip(1.15 - np.abs(rd - 2.2) * .25, 0, 1)
    p = sphere(cx, cy, R, L=(-.7, -.45, .55))
    lam = np.clip(p[5], 0, 1)
    lat = (-dx * math.sin(t) + dy * math.cos(t))
    bands = .96 + .04 * np.sin(lat * 9 + np.sin(lat * 4) * 1.5)
    body = mix(hexc("#14307f"), hexc("#a9d0ff"), lam ** 1.1) * bands[..., None]
    body = screen(body, (np.exp(-((p[4] - 1) * R / (6 * K)) ** 2) * .5 * (lam + .2))[..., None] * hexc("#cfe6ff"))
    behind = (uy < 0)
    img = screen(img, (ring * behind * .75)[..., None] * hexc("#e6f2ff"))
    img = mix(img, body, p[3])
    img = screen(img, (ring * (~behind) * .9)[..., None] * hexc("#eef6ff"))
    img = screen(img, (blur(ring, 30) * .25)[..., None] * hexc("#8cc0ff"))
    return img


def orbits_light():
    """Thin luminous orbits around an unseen star, with soft glows; calm and
    abstract."""
    rng = np.random.default_rng(35)
    img = ramp(u * .45 + v * .55, [(0, "#7fb0ff"), (.5, "#4f86f5"), (1, "#3f72e6")])
    img = screen(img, stars(rng, 600, bright=.45)[..., None])
    cx, cy = .30 * W, .35 * H
    t = math.radians(-20)
    for i, (a, dot, ang) in enumerate([(.22, 18, 40), (.36, 26, 200), (.52, 14, 300), (.72, 34, 120), (.95, 22, 250)]):
        ex = ((x - cx) * math.cos(t) + (y - cy) * math.sin(t)) / (a * W)
        ey = (-(x - cx) * math.sin(t) + (y - cy) * math.cos(t)) / (a * W * .45)
        e = np.abs(np.hypot(ex, ey) - 1) * a * W * .45
        line = np.exp(-(e / (2.2 * K)) ** 2) * .55 + np.exp(-(e / (30 * K)) ** 2) * .10
        img = screen(img, line[..., None] * hexc("#e8f3ff"))
        th = math.radians(ang)
        px = cx + a * W * math.cos(th) * math.cos(t) - a * W * .45 * math.sin(th) * math.sin(t)
        py = cy + a * W * math.cos(th) * math.sin(t) + a * W * .45 * math.sin(th) * math.cos(t)
        r = np.hypot(x - px, y - py) / (dot * K)
        img = screen(img, (np.clip(1.6 - r, 0, 1) + np.exp(-r / 2.5) * .5)[..., None] * hexc("#f4f9ff"))
    rs = np.hypot(x - cx, y - cy) / W
    img = screen(img, (np.exp(-(rs / .05) ** 2) * .9 + np.exp(-(rs / .25) ** 2) * .35)[..., None] * hexc("#eaf4ff"))
    # soft out-of-focus lights
    for bx, by, br, a in [(.82, .25, .10, .10), (.70, .78, .14, .08), (.92, .62, .06, .12), (.12, .85, .08, .07)]:
        d = np.hypot(u - bx, (v - by) * H / W) / (br * .56)
        img = screen(img, (np.clip((1 - d) * 30, 0, 1) * a)[..., None] * hexc("#cfe4ff"))
    return img


def haze():
    """A calm, out-of-focus nebula: soft clouds of light in blues and a few
    stars."""
    rng = np.random.default_rng(37)
    img = ramp(v, [(0, "#4a82f5"), (.5, "#3567e0"), (1, "#4378ee")])
    n1 = blur(fbm(1500, 4, rng), 60)
    n2 = blur(fbm(800, 4, rng), 40)
    band = np.exp(-((v - .55 + (u - .5) * .55) / .30) ** 2)
    c = np.clip((n1 * .7 + n2 * .5 - .52) * 4, 0, 1) * band
    img = screen(img, (c * .9)[..., None] * hexc("#6fb3ff"))
    img = screen(img, (np.clip(c - .45, 0, 1) * 1.6)[..., None] * hexc("#e2f3ff"))
    img = screen(img, (blur(c, 80) * .35)[..., None] * hexc("#8a9bff"))
    img = screen(img, stars(rng, 1600, bright=.7)[..., None])
    return img


img = {"dawn": dawn, "aurora": aurora, "layers": layers, "crescent": crescent, "halo": halo,
       "orbits-light": orbits_light, "haze": haze}[name]()
save(img)
