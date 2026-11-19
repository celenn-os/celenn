// Orbit lines: the logo's motif as a pattern, a field of thin "c" orbits
// with their planets, in blues on night, a few lit up.
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution;
  vec2 uv = frag / res, p = (frag - .5 * res) / res.y;
  vec3 col = mix(hex(0x0e1633), hex(0x16245a), smoothstep(-.6, .7, p.x * .4 + p.y));
  float px = 1. / res.y;
  float cell = .30;
  vec2 g = p / cell;
  vec2 id = floor(g); vec2 f = fract(g) - .5;
  for (int j = -1; j <= 1; j++) for (int i = -1; i <= 1; i++) {
    vec2 o = vec2(i, j), cid = id + o;
    vec2 q = (f - o) * cell;
    float h = hash12(cid * 1.7 + 3.);
    float r = cell * mix(.28, .46, h);
    float rot = floor(hash12(cid + 9.) * 4.) * 1.5708;
    float a = atan(q.y, q.x) - rot;
    a = mod(a + 3.14159, 6.2832) - 3.14159;
    float gap = .65;
    float arc = abs(length(q) - r);
    float onArc = step(gap, abs(a));
    // stroke gradient along the arc (bright head, fading tail)
    float s = clamp((abs(a) - gap) / (3.14159 - gap), 0., 1.);
    float lit = step(.86, hash12(cid + 21.));
    float w = mix(1.1, 2.0, lit) * px * 1.5;
    float line = exp(-pow(arc / w, 2.)) * onArc;
    vec3 lc = mix(mix(BRAND, CYAN, s), mix(ICE, BRAND, s), lit) * mix(.30, 1.6, lit) * (a > 0. ? 1. : mix(1., .45, s));
    col += lc * line;
    // the planet in the middle of each orbit
    float pd = length(q) - r * .32;
    col += mix(DEEP * .9, mix(BRAND, ICE, .4), lit) * smoothstep(px * 1.5, -px * 1.5, pd) * mix(.5, 1.2, lit);
    col += BRAND * .25 * lit * exp(-max(pd, 0.) / .02);
  }
  col += BRAND * .10 * exp(-dot(p - vec2(.4, .1), p - vec2(.4, .1)) / .5);
  fragColor = finish(col, frag);
}
