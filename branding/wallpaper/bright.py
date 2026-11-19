#!/usr/bin/env python3
"""The bright cosmic-blue wallpapers, drawn with numpy (5120x2880).
Usage: bright.py NAME OUT.jpg [SCALE]
NAME: dawn, aurora, nebula, eclipse, dunes, layers. SCALE (default 1) renders
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
                                               (.65, "#2148bf"), (1, "#16308a")])
    sky = screen(sky, stars(rng, 2600, bright=.8, mask=np.clip(alt * 4 - 1.6, 0, 1))[..., None])
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
    img = ramp(v * .6 + u * .4, [(0, "#9ec5ff"), (.5, "#4f8dff"), (1, "#1d47c4")])
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


def nebula():
    """A bright, wide nebula of blues and cyans with dark dust and stars."""
    rng = np.random.default_rng(21)
    wx = (fbm(1400, 5, rng) - .5) * 700 * K
    wy = (fbm(1400, 5, rng) - .5) * 700 * K
    # a sweeping arc of gas across the screen
    cx, cy, R = .30 * W, 1.55 * H, 1.25 * H
    d = (np.hypot(x + wx - cx, y + wy - cy) - R) / H
    band = np.exp(-(d / .22) ** 2)
    det = fbm(700, 8, rng, .55)
    gas = band * np.clip((det - .25) * 2.0, 0, 1) ** 1.2
    puff = fbm(320, 7, rng, .55)
    gas2 = band * np.clip((puff - .5) * 4, 0, 1) ** 1.5
    dust = blur((1 - np.abs(2 * fbm(520, 6, rng, .5) - 1)) ** 4, 14) * np.exp(-((d + .05) / .10) ** 2)
    base = ramp(v, [(0, "#1d3f9e"), (.5, "#2551c2"), (1, "#1a3a96")])
    base = mix(base, hexc("#3f6fe0"), np.exp(-((u - .62) ** 2 / .12 + (v - .45) ** 2 / .08)) * .7)
    img = screen(base, stars(rng, 5000, bright=.9)[..., None])
    img = screen(img, (gas * .85)[..., None] * hexc("#6fb3ff"))
    img = screen(img, (gas2 * .7)[..., None] * hexc("#bfefff"))
    img = screen(img, (blur(gas, 60) * .5)[..., None] * hexc("#7a86ff"))
    img = img * (1 - .38 * np.clip(dust * 1.3, 0, 1))[..., None]
    # two bright young stars in the gas
    for sx, sy, s in [(.63, .34, 1.0), (.41, .52, .6)]:
        rs = np.hypot(x - sx * W, y - sy * H)
        ang = np.arctan2(y - sy * H, x - sx * W)
        core = np.exp(-(rs / (18 * K)) ** 2) * 1.5 + np.exp(-(rs / (160 * K)) ** 2) * .35
        spikes = np.abs(np.cos(2 * ang)) ** 600 * np.exp(-rs / (160 * K)) * .5
        img = screen(img, ((core + spikes) * s)[..., None] * hexc("#f2f9ff"))
    return img


def eclipse():
    """A planet passing in front of its star: a ring of light (the shape of
    the logo's orbit) on a luminous blue sky."""
    rng = np.random.default_rng(9)
    cx, cy, R = .56 * W, .46 * H, .20 * H
    rr = np.hypot(x - cx, y - cy) / R
    ang = np.arctan2(y - cy, x - cx)
    img = ramp(np.clip(np.hypot((u - .56) * 1.4, v - .47) / .95, 0, 1),
               [(0, "#5d9bff"), (.3, "#3b72ea"), (.7, "#2450c4"), (1, "#1a3a9a")])
    img = screen(img, stars(rng, 2200, bright=.6, mask=np.clip(rr / 3 - .3, 0, 1))[..., None])
    # corona: streamers, brighter towards the lower right (the "diamond")
    streamer = fbm(260, 5, rng)
    rays = np.interp((ang + np.pi) / (2 * np.pi) * 255, np.arange(256), np.random.default_rng(4).random(256))
    rays = gaussian_filter(rays, 0)
    cor = np.exp(-np.clip(rr - 1, 0, None) / (.35 + .25 * rays)) * (rr > 1)
    cor *= .75 + .5 * streamer
    img = screen(img, (cor * .9)[..., None] * hexc("#cfe6ff"))
    img = screen(img, (np.exp(-np.clip(rr - 1, 0, None) / 1.6) * .35 * (rr > .99))[..., None] * hexc("#8cc0ff"))
    ring = np.exp(-((rr - 1) * R / (5 * K)) ** 2) * 1.2
    img = screen(img, ring[..., None] * hexc("#f2f9ff"))
    da = math.radians(38)
    dmd = np.hypot(x - (cx + R * math.cos(da)), y - (cy + R * math.sin(da)))
    diamond = np.exp(-(dmd / (22 * K)) ** 2) * 1.6 + np.exp(-(dmd / (200 * K)) ** 2) * .55
    dang = np.arctan2(y - (cy + R * math.sin(da)), x - (cx + R * math.cos(da)))
    diamond += np.abs(np.cos(2 * (dang - .2))) ** 900 * np.exp(-dmd / (220 * K)) * .5
    # the dark planet, with a faint earthshine
    p = sphere(cx, cy, R, L=(-.6, -.5, .2))
    disc = hexc("#0c1f5c") * (.75 + .5 * np.clip(p[5], 0, 1))[..., None]
    img = mix(img, disc, p[3])
    img = screen(img, diamond[..., None] * hexc("#ffffff"))
    return img


def dunes():
    """Blue dunes of an icy world under a ringed planet, in layers that fade
    into a bright haze."""
    rng = np.random.default_rng(13)
    sky = ramp(v / .68, [(0, "#2b55cf"), (.5, "#5a92ff"), (.85, "#a9d0ff"), (1, "#d6eaff")])
    sky = screen(sky, stars(rng, 1500, bright=.6, mask=np.clip(1 - v * 3.5, 0, 1))[..., None])
    # ringed planet high in the sky, lit from the left
    pcx, pcy, pr = .69 * W, .27 * H, .11 * H
    t = math.radians(-14)
    dx, dy = (x - pcx) / pr, (y - pcy) / pr
    ux = dx * math.cos(t) + dy * math.sin(t)
    uy = -dx * math.sin(t) + dy * math.cos(t)
    d = np.hypot(dx, dy)
    nz = np.sqrt(np.clip(1 - d * d, 0, 1))
    inside = np.clip((1 - d) * pr / 1.5, 0, 1)
    Lv = np.array([-.8, -.25, .55]); Lv /= np.linalg.norm(Lv)
    lam = np.clip(dx * Lv[0] + dy * Lv[1] + nz * Lv[2], 0, 1)
    bands = .88 + .12 * np.interp(uy * 20 + 20, np.arange(41), np.random.default_rng(6).random(41))
    disc = mix(hexc("#1b3a9c"), hexc("#e3f0ff"), lam ** .9) * bands[..., None]
    rim = np.exp(-((d - 1) * pr / (4 * K)) ** 2) * (d > .97) * .5
    ry = uy / .2
    rd = np.hypot(ux, ry)
    rtex = .55 + .45 * np.interp(rd, np.linspace(1.45, 2.35, 48), np.random.default_rng(8).random(48))
    ring = np.clip((rd - 1.45) * 60, 0, 1) * np.clip((2.35 - rd) * 60, 0, 1) * rtex
    gap = 1 - .8 * np.exp(-((rd - 1.95) / .025) ** 2)
    ring *= gap
    front = (ry > 0) | (inside == 0)
    img = mix(sky, hexc("#eef6ff"), ring * (~front) * .7)
    img = mix(img, disc, inside)
    img = screen(img, rim[..., None] * hexc("#bfe0ff"))
    img = mix(img, hexc("#eef6ff"), ring * front * .8)
    # dune layers, back to front: each a ridge, the lit face to the left
    layers = [(.60, .025, "#b9d8ff", "#8ab8ff", .30), (.68, .040, "#9cc7ff", "#5a8ff0", .18),
              (.77, .055, "#7fb0ff", "#3a6ad8", .08), (.88, .065, "#6aa1ff", "#2350c0", 0)]
    for i, (base, amp, clit, cdark, haze) in enumerate(layers):
        ph = np.random.default_rng(40 + i).uniform(0, 2 * np.pi, 3)
        xs = u[0]
        ridge = base + amp * (np.sin(xs * (3 + i) + ph[0]) * .6 + np.sin(xs * (7 + 2 * i) + ph[1]) * .3
                              + np.sin(xs * (13 + i) + ph[2]) * .1)
        below = v - ridge[None, :]
        mask = np.clip(below * H / 1.2, 0, 1)
        slope = np.gradient(ridge)[None, :] * W
        lit = np.clip(.5 - slope * 2.5, 0, 1)
        dd = np.clip(below / .10, 0, 1)
        col = mix(hexc(clit), hexc(cdark), .25 + dd * .45 + (1 - lit) * .3)
        crest = np.exp(-(below * H / (5 * K)) ** 2) * (.25 + .6 * lit)
        col = screen(col, crest[..., None] * hexc("#ffffff"))
        col = mix(col, hexc("#d6eaff"), haze)
        img = mix(img, col, mask)
    return img


def layers():
    """Translucent sheets of light folded over each other, from a pale sky
    at the top to deep blue at the bottom."""
    img = ramp(v, [(0, "#d4e8ff"), (1, "#7fb0ff")])
    sheets = [("#b4d4ff", "#8fbaff"), ("#8fbaff", "#6a9cff"), ("#7fa0ff", "#5a7df0"), ("#5d93ff", "#3f74ef"),
              ("#4a7ff2", "#2f5fd8"), ("#3567df", "#244dc4"), ("#2752c9", "#1a3ca6"), ("#1f45b8", "#152f8a")]
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


img = {"dawn": dawn, "aurora": aurora, "nebula": nebula, "eclipse": eclipse, "dunes": dunes, "layers": layers}[name]()
save(img)
