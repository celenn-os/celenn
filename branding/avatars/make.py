"""Profile pictures (avatars) of Celenn OS: a white drawing on a coloured
gradient with a few faint stars, in the style of the dock icons.

Writes svg/<name>.svg. export.js renders them to the 512x512 PNGs of
system_files/usr/share/pixmaps/faces (the size COSMIC Settings resizes
profile pictures to). bbox.js measures each drawing; offsets.json holds the
shifts that centre it in the picture (ground lines excluded).
"""
import json, math, random, pathlib
OUT = pathlib.Path(__file__).parent / "svg"; OUT.mkdir(exist_ok=True)
W = "#fff"
# One hue per avatar, spread around the colour wheel (degrees), so that no two
# accounts look alike; the shooting
# star is neutral (slate), the radio dish too (stone). Gradient from light (top left) to dark.
import colorsys
HUES = {"astronaut": 0, "rover": 15, "rocket": 32, "constellation": 46, "ufo": 75,
        "alien": 100, "telescope": 128, "comet": 152, "earth": 175, "aurora": 193,
        "planet": 220, "galaxy": 243, "moon": 263, "black-hole": 283, "orbit": 303,
        "satellite": 325, "eclipse": 345, "star": 60, "lander": 140, "capsule": 207}
def hsl(h, s, l):
    r, g, b = colorsys.hls_to_rgb(h / 360, l, s)
    return "#%02x%02x%02x" % (round(r * 255), round(g * 255), round(b * 255))
def colors(name):
    if name == "shooting-star":
        return ("#94a3b8", "#1e293b")
    if name == "radio-dish":
        return ("#a8a29e", "#292524")
    h = HUES[name]
    # yellows and greens look lighter: darker so the white drawing stands out
    dim = .06 if 40 <= h <= 140 else 0
    # Muted, night-sky tones: moderate saturation, same depth for all
    sat = .36 if 90 <= h <= 150 else .44
    return (hsl(h, sat, .56 - dim), hsl(h, sat + .06, .26 - dim / 2))
def planet(d): return f'''<ellipse cx="128" cy="128" rx="84" ry="22" fill="none" stroke="{W}" stroke-width="9" opacity=".55" transform="rotate(-18 128 128)"/>
<circle cx="128" cy="128" r="46" fill="{W}"/><path d="M96 110c8-10 20-15 32-15" stroke="{d}" stroke-width="6" stroke-linecap="round" fill="none" opacity=".2"/>
<path d="M44 128A84 22 0 0 0 212 128" fill="none" stroke="{W}" stroke-width="9" stroke-linecap="round" transform="rotate(-18 128 128)"/>'''
def moon(d): return f'''<mask id="m"><rect width="256" height="256" fill="#fff"/><circle cx="150" cy="108" r="54" fill="#000"/></mask>
<circle cx="122" cy="132" r="62" fill="{W}" mask="url(#m)"/><circle cx="96" cy="150" r="9" fill="{d}" opacity=".18"/><circle cx="108" cy="176" r="6" fill="{d}" opacity=".18"/>
<path d="M186 70l4 11 11 4-11 4-4 11-4-11-11-4 11-4z" fill="{W}"/><path d="M196 150l3 7 7 3-7 3-3 7-3-7-7-3 7-3z" fill="{W}" opacity=".8"/>'''
def comet(d): return f'''<path d="M150 88L52 196" stroke="{W}" stroke-width="22" stroke-linecap="round" opacity=".25"/>
<path d="M156 96L76 194" stroke="{W}" stroke-width="12" stroke-linecap="round" opacity=".45"/><path d="M168 100L112 176" stroke="{W}" stroke-width="6" stroke-linecap="round" opacity=".7"/>
<circle cx="168" cy="88" r="26" fill="{W}"/>'''
def rocket(d): return f'''<g transform="rotate(35 128 128)">
<path d="M118 108L84 146q-4 5-4 11v15l38-20zM138 108l34 38q4 5 4 11v15l-38-20z" fill="{W}"/>
<path d="M128 40c26 22 34 56 30 104h-60c-4-48 4-82 30-104z" fill="{W}"/><circle cx="128" cy="96" r="13" fill="{d}"/>
<path d="M112 150h32c0 16-8 30-16 40-8-10-16-24-16-40z" fill="{W}" opacity=".5"/></g>'''
def satellite(d): return f'''<g transform="rotate(-30 128 128)"><rect x="110" y="104" width="36" height="48" rx="6" fill="{W}"/>
<rect x="40" y="112" width="60" height="32" rx="4" fill="{W}" opacity=".75"/><rect x="156" y="112" width="60" height="32" rx="4" fill="{W}" opacity=".75"/>
<path d="M60 112v32M80 112v32M176 112v32M196 112v32" stroke="{d}" stroke-width="3" opacity=".35"/><path d="M100 128h10M146 128h10" stroke="{W}" stroke-width="5"/>
<path d="M128 104v-14" stroke="{W}" stroke-width="5"/><path d="M108 90a20 12 0 0 0 40 0z" fill="{W}"/></g>'''
def telescope(d): return f'''<g transform="rotate(-28 128 112)"><rect x="66" y="96" width="112" height="30" rx="8" fill="{W}"/><rect x="176" y="88" width="20" height="46" rx="5" fill="{W}"/>
<rect x="46" y="102" width="22" height="18" rx="4" fill="{W}" opacity=".75"/></g>
<path d="M128 132l-34 74M128 132l34 74M128 132v76" stroke="{W}" stroke-width="8" stroke-linecap="round" opacity=".75"/><circle cx="128" cy="132" r="9" fill="{W}"/>
<path d="M190 52l3 8 8 3-8 3-3 8-3-8-8-3 8-3z" fill="{W}"/>'''
def helmet(d): return f'''<circle cx="128" cy="124" r="70" fill="{W}"/><rect x="78" y="92" width="100" height="66" rx="33" fill="{d}"/>
<path d="M94 112c6-8 16-11 26-11" stroke="{W}" stroke-width="7" stroke-linecap="round" opacity=".5" fill="none"/><circle cx="160" cy="140" r="4" fill="{W}" opacity=".5"/>
<rect x="84" y="186" width="88" height="22" rx="11" fill="{W}" opacity=".75"/><circle cx="58" cy="124" r="10" fill="{W}" opacity=".75"/><circle cx="198" cy="124" r="10" fill="{W}" opacity=".75"/>'''
def constellation(d):
    p=[(58,168),(96,120),(140,132),(172,84),(204,108),(150,190)]
    l=" ".join(f"{'M' if i==0 else 'L'}{x} {y}" for i,(x,y) in enumerate(p[:5]))
    return f'''<path d="{l}M140 132L150 190" fill="none" stroke="{W}" stroke-width="4" opacity=".55"/>'''+"".join(f'<circle cx="{x}" cy="{y}" r="{r}" fill="{W}"/>' for (x,y),r in zip(p,[9,11,8,13,9,10]))
def galaxy(d):
    # Two tapered arms (filled), turning around a bright core
    arms = ""
    for rot in (0, math.pi):
        left, right = [], []
        n = 70
        for i in range(n):
            t = i / (n - 1)
            a = rot + t * 3.0
            r = 14 + t * 70
            w = 18 * math.sin(math.pi * min(1, t * 1.15)) ** 0.8 + 2.5
            x, y = r * math.cos(a), r * math.sin(a)
            nx, ny = math.cos(a), math.sin(a)
            left.append((x + nx * w / 2, y + ny * w / 2))
            right.append((x - nx * w / 2, y - ny * w / 2))
        pts = left + right[::-1]
        path = "M" + " L".join(f"{128 + x:.1f} {128 + 0.6 * y:.1f}" for x, y in pts) + "z"
        arms += f'<path d="{path}" fill="{W}" opacity=".9" transform="rotate(-25 128 128)"/>'
    return (f'<ellipse cx="128" cy="128" rx="44" ry="26" fill="{W}" opacity=".2" transform="rotate(-25 128 128)"/>'
            + arms + f'<circle cx="128" cy="128" r="17" fill="{W}"/>')
def ribbon(x0, x1, y, amp, thick, phase):
    # Wavy band with tapered ends
    top, bot = [], []
    for i in range(41):
        x = x0 + (x1 - x0) * i / 40
        c = y + amp * math.sin(phase + (x - x0) / (x1 - x0) * 2.4 * math.pi) - (x - x0) * 0.25
        t = thick * math.sin(math.pi * i / 40) ** 0.6
        top.append((x, c - t / 2)); bot.append((x, c + t / 2))
    return "M" + " L".join(f"{x:.1f} {y:.1f}" for x, y in top + bot[::-1]) + "z"
def aurora(d): return (f'<path d="{ribbon(50, 206, 112, 9, 26, 0.3)}" fill="{W}" opacity=".3"/>'
    f'<path d="{ribbon(56, 200, 132, 8, 14, 0.8)}" fill="{W}" opacity=".6"/>'
    f'<path d="M50 196l48-62 24 28 30-42 54 76z" fill="{W}" stroke="{W}" stroke-width="6" stroke-linejoin="round"/>')
def orbit(d): return f'''<ellipse cx="128" cy="128" rx="88" ry="40" fill="none" stroke="{W}" stroke-width="6" opacity=".5" transform="rotate(-25 128 128)"/>
<ellipse cx="128" cy="128" rx="58" ry="24" fill="none" stroke="{W}" stroke-width="5" opacity=".75" transform="rotate(30 128 128)"/>
<circle cx="128" cy="128" r="24" fill="{W}"/><circle cx="206" cy="92" r="12" fill="{W}"/><circle cx="82" cy="100" r="8" fill="{W}"/>'''
def shooting(d):
    s=[]
    for i in range(10):
        a=math.radians(-90+i*36); r=40 if i%2==0 else 17
        s.append(f"{158+r*math.cos(a):.1f} {104+r*math.sin(a):.1f}")
    return f'''<path d="M126 132L48 210M138 150L80 208M114 120L52 182" stroke="{W}" stroke-width="8" stroke-linecap="round" opacity=".45"/><path d="M{' L'.join(s)}z" fill="{W}" stroke="{W}" stroke-width="8" stroke-linejoin="round"/>'''
def ufo(d): return f'''<path d="M100 150L70 214h116l-30-64z" fill="{W}" opacity=".3"/><ellipse cx="128" cy="134" rx="78" ry="24" fill="{W}"/>
<path d="M90 124a38 36 0 0 1 76 0z" fill="{W}" opacity=".75"/><circle cx="96" cy="136" r="6" fill="{d}"/><circle cx="128" cy="140" r="6" fill="{d}"/><circle cx="160" cy="136" r="6" fill="{d}"/>'''
def eclipse(d):
    rays="".join(f'<path d="M{128+62*math.cos(math.radians(a)):.1f} {128+62*math.sin(math.radians(a)):.1f}L{128+86*math.cos(math.radians(a)):.1f} {128+86*math.sin(math.radians(a)):.1f}" stroke="{W}" stroke-width="7" stroke-linecap="round" opacity=".6"/>' for a in range(0,360,30))
    return rays+f'<circle cx="128" cy="128" r="50" fill="{W}"/><circle cx="140" cy="120" r="44" fill="{d}"/>'
def blackhole(d):
    swirl="".join(f'<circle cx="128" cy="128" r="{r}" fill="none" stroke="{W}" stroke-width="{w}" stroke-linecap="round" opacity="{o}" stroke-dasharray="{2*math.pi*r*.28:.1f} {2*math.pi*r*.22:.1f}" transform="rotate({a} 128 128)"/>' for r,w,o,a in [(88,5,.3,10),(75,7,.45,55),(62,9,.65,100)])
    return swirl+f'<circle cx="128" cy="128" r="46" fill="{W}"/><circle cx="128" cy="128" r="38" fill="{d}"/>'
def rover(d): return f'''<rect x="70" y="120" width="112" height="40" rx="8" fill="{W}"/>
<path d="M150 120v-34M138 86h30" stroke="{W}" stroke-width="7" stroke-linecap="round"/><rect x="82" y="104" width="52" height="14" rx="3" fill="{W}" opacity=".75"/>
<circle cx="86" cy="178" r="16" fill="{W}"/><circle cx="126" cy="178" r="16" fill="{W}"/><circle cx="166" cy="178" r="16" fill="{W}"/>
<circle cx="86" cy="178" r="6" fill="{d}"/><circle cx="126" cy="178" r="6" fill="{d}"/><circle cx="166" cy="178" r="6" fill="{d}"/>'''
# Land: Natural Earth 1:110m (public domain), orthographic view (earth-ortho.py)
LAND=(pathlib.Path(__file__).parent/"earth-land.txt").read_text().strip()
def earth(d): return f'''<clipPath id="e"><circle cx="128" cy="128" r="60"/></clipPath><circle cx="128" cy="128" r="60" fill="{W}"/>
<path d="{LAND}" fill="{d}" stroke="{d}" stroke-width="5" stroke-linejoin="round" opacity=".55" clip-path="url(#e)"/><circle cx="196" cy="68" r="12" fill="{W}"/>'''
def alien(d): return f'''<path d="M110 74L94 46M146 74L162 46" stroke="{W}" stroke-width="6" stroke-linecap="round"/><circle cx="92" cy="42" r="9" fill="{W}"/><circle cx="164" cy="42" r="9" fill="{W}"/>
<path d="M128 64c40 0 64 28 64 60 0 38-34 82-64 82s-64-44-64-82c0-32 24-60 64-60z" fill="{W}"/>
<path d="M90 128c14-4 28 6 30 22-14 4-28-6-30-22zM166 128c-14-4-28 6-30 22 14 4 28-6 30-22z" fill="{d}"/><path d="M118 180c7 4 13 4 20 0" stroke="{d}" stroke-width="5" stroke-linecap="round" fill="none"/>'''
def star(d):
    # A four-pointed sparkle, with two small ones
    def sparkle(cx, cy, r, w):
        return (f"M{cx} {cy - r}Q{cx + w} {cy - w} {cx + r} {cy}Q{cx + w} {cy + w} {cx} {cy + r}"
                f"Q{cx - w} {cy + w} {cx - r} {cy}Q{cx - w} {cy - w} {cx} {cy - r}z")
    return (f'<circle cx="122" cy="134" r="40" fill="{W}" opacity=".18"/><path d="{sparkle(122, 134, 76, 12)}" fill="{W}"/>'
            f'<path d="{sparkle(190, 66, 22, 4)}" fill="{W}" opacity=".85"/><path d="{sparkle(196, 190, 14, 3)}" fill="{W}" opacity=".6"/>')
def lander(d): return f'''<path d="M100 150L66 200M156 150L190 200M112 156l-14 44M144 156l14 44" stroke="{W}" stroke-width="7" stroke-linecap="round" opacity=".75"/>
<path d="M54 202h26M176 202h26" stroke="{W}" stroke-width="8" stroke-linecap="round"/>
<rect x="84" y="128" width="88" height="34" rx="6" fill="{W}"/><path d="M98 128l8-44h44l8 44z" fill="{W}"/>
<rect x="114" y="96" width="28" height="18" rx="5" fill="{d}"/><path d="M128 84V60" stroke="{W}" stroke-width="5" stroke-linecap="round"/><circle cx="128" cy="56" r="7" fill="{W}"/>
<path d="M114 162h28l-6 14h-16z" fill="{W}" opacity=".6"/>'''
def capsule(d): return f'''<path d="M48 102a80 62 0 0 1 160 0q-20-10-40 0q-20-10-40 0q-20-10-40 0q-20-10-40 0z" fill="{W}"/>
<path d="M88 66v36M128 40v62M168 66v36" stroke="{d}" stroke-width="4" opacity=".2"/>
<path d="M52 104l58 80M204 104l-58 80M128 102v82" stroke="{W}" stroke-width="3.5" opacity=".7"/>
<path d="M112 182h32l12 30h-56z" fill="{W}"/><rect x="122" y="192" width="12" height="10" rx="3" fill="{d}"/>'''
def radiodish(d): return f'''<g transform="rotate(-30 120 120)"><path d="M58 104a62 62 0 0 0 124 0z" fill="{W}"/>
<path d="M120 104V62" stroke="{W}" stroke-width="5"/><circle cx="120" cy="58" r="8" fill="{W}"/></g>
<path d="M120 150l-26 56h52z" fill="{W}" opacity=".75"/><rect x="80" y="204" width="80" height="10" rx="5" fill="{W}"/>
<path d="M166 66a26 26 0 0 1 22 22M172 46a46 46 0 0 1 38 38" stroke="{W}" stroke-width="6" stroke-linecap="round" fill="none" opacity=".8"/>'''
GLYPHS=[("planet",planet),("moon",moon),("comet",comet),("rocket",rocket),("satellite",satellite),("telescope",telescope),
        ("astronaut",helmet),("constellation",constellation),("galaxy",galaxy),("aurora",aurora),("orbit",orbit),("shooting-star",shooting),("ufo",ufo),("eclipse",eclipse),("black-hole",blackhole),("rover",rover),("earth",earth),("alien",alien),
        ("star",star),("lander",lander),("capsule",capsule),("radio-dish",radiodish)]
# Ground drawn edge to edge, outside the centred drawing
GROUND={"rover":'<path d="M-10 190c50-16 90-10 140-4s80 4 136-8v90H-10z" fill="#fff" opacity=".3"/>'}
OFF={k:tuple(v) for k,v in json.loads((pathlib.Path(__file__).parent/"offsets.json").read_text()).items()} if (pathlib.Path(__file__).parent/"offsets.json").exists() else {}
for name,fn in GLYPHS:
    c1,c2=colors(name)
    random.seed(name); stars="".join(f'<circle cx="{random.uniform(8,248):.0f}" cy="{random.uniform(8,248):.0f}" r="{random.choice([1,1.3,1.8])}" fill="#fff" opacity="{random.uniform(.25,.6):.2f}"/>' for _ in range(22))
    svg=f'''<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256"><defs><radialGradient id="g" cx=".35" cy=".3" r=".9"><stop offset="0" stop-color="{c1}"/><stop offset="1" stop-color="{c2}"/></radialGradient></defs>
<rect width="256" height="256" fill="url(#g)"/>{stars}<g class="ground">{GROUND.get(name,"").replace("{d}",c2)}</g><g transform="translate({OFF.get(name,(0,0))[0]} {OFF.get(name,(0,0))[1]})">{fn(c2)}</g></svg>'''
    (OUT/f"{name}.svg").write_text(svg)
print("ok")
