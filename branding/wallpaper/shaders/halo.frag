// Halo: the logo made physical. A luminous orbit ring with a gap circles a
// small planet, lit from the upper left, over a bright sky of light.
const vec3 C = vec3(.85, .02, -8.);       // planet centre
const float PR = .70, RR = 1.45, RT = .0055; // planet radius, ring radius, ring tube
const float GAP = .80;                      // half the gap, radians
const vec3 Lk = vec3(-.62, .52, .58);       // key light direction (normalized below)

mat3 rotX(float a) { float c = cos(a), s = sin(a); return mat3(1, 0, 0, 0, c, s, 0, -s, c); }
mat3 rotZ(float a) { float c = cos(a), s = sin(a); return mat3(c, s, 0, -s, c, 0, 0, 0, 1); }
vec3 toLocal(vec3 v) { return rotX(radians(-9.)) * (rotZ(radians(8.)) * v); }

float ring(vec3 pos) {
  vec3 q = toLocal(pos - C);
  float a = atan(q.z, q.x);
  if (abs(a) < GAP) {
    vec3 e1 = vec3(cos(GAP), 0., sin(GAP)) * RR, e2 = vec3(cos(GAP), 0., -sin(GAP)) * RR;
    return min(length(q - e1), length(q - e2)) - RT;
  }
  vec2 t = vec2(length(q.xz) - RR, q.y);
  return length(t) - RT;
}

vec2 sphere(vec3 ro, vec3 rd, vec3 c, float r) {
  vec3 oc = ro - c; float b = dot(oc, rd), h = b * b - dot(oc, oc) + r * r;
  if (h < 0.) return vec2(-1.); h = sqrt(h); return vec2(-b - h, -b + h);
}

vec3 background(vec2 uv, vec2 p, vec2 frag) {
  vec2 hero = vec2(.85 / 4., .02 / 4.);   // planet centre on screen
  float pk = pocketMask(p, hero, vec2(.95, .36));
  vec3 col = mix(spaceBase(p), INK * .55, pk * .92);
  // nebulous light inside the pocket
  float n = fbm(vec3(p * 1.3 + vec2(0., .2), 1.3), 7), n2 = fbm(vec3(p * 3.2, 7.), 6);
  col += (DEEP * .35 + VIOLET * .14) * smoothstep(.42, .8, n) * pk;
  col += ICE * .08 * smoothstep(.55, .85, n2) * smoothstep(.45, .7, n) * pk;
  // warm-orchid key glow from the lower-left corner
  col += (WARM * .06 + ORCHID * .06) * exp(-length((uv - vec2(0., 0.)) * vec2(1.4, 1.)) / .45);
  col += vec3(.9, .95, 1.) * stars(frag, .06, iResolution.y / 48.) * pk;
  return col;
}

void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution;
  vec2 uv = frag / res, p = (frag - .5 * res) / res.y;
  vec3 L = normalize(Lk);
  vec3 ro = vec3(0), rd = normalize(vec3(p, -2.));
  vec3 col = background(uv, p, frag);

  // ring: march inside its bounding sphere
  float tr = -1., dmin = 1e9;
  vec2 bs = sphere(ro, rd, C, RR + .08);
  if (bs.y > 0.) {
    float t = max(bs.x, 0.);
    for (int i = 0; i < 140; i++) {
      float d = ring(ro + rd * t);
      dmin = min(dmin, d);
      if (d < .0004) { tr = t; break; }
      t += d * .8;
      if (t > bs.y) break;
    }
  }
  vec2 sp = sphere(ro, rd, C, PR);
  float tp = sp.x;

  // planet atmosphere glow outside the disc
  vec3 oc = C - ro; float b = length(cross(oc, rd));
  vec3 cp = ro + rd * dot(oc, rd);
  float lit = clamp(dot(normalize(cp - C), L) * .7 + .4, 0., 1.);
  col += (CYAN * .55 + ICE * .3) * exp(-max(b - PR, 0.) / .035) * lit * step(PR, b) * .9;

  if (tp > 0. && (tr < 0. || tp < tr)) {
    vec3 hp = ro + rd * tp, n = normalize(hp - C);
    float dl = dot(n, L), diff = max(dl, 0.);
    float c = fbm(vec3(n.y * 7., n.xz * 1.2), 5);
    vec3 surf = mix(DEEP, SKY, .45 + .2 * c);
    vec3 day = surf * (diff * 1.3) + WARM * .10 * smoothstep(.18, .0, abs(dl - .06));
    vec3 night = ABYSS * .45 + DEEP * .12 * max(dot(n, normalize(vec3(.3, -1., .4))), 0.);
    vec3 pc = mix(night, day, smoothstep(-.08, .15, dl));
    // shadow of the ring on the planet
    float sh = 1.; float t = .02;
    for (int i = 0; i < 60; i++) { float d = ring(hp + L * t); if (d < .002) { sh = .35; break; } t += max(d, .01); if (t > 3.) break; }
    pc *= mix(1., sh, smoothstep(-.05, .2, dl));
    float fr = pow(1. - max(dot(n, -rd), 0.), 3.);
    pc += (CYAN * .8 + ICE * .3) * fr * (.25 + diff);
    col = pc;
  }
  if (tr > 0. && (tp < 0. || tr < tp)) {
    vec3 hp = ro + rd * tr;
    vec2 e = vec2(.0005, 0);
    vec3 n = normalize(vec3(ring(hp + e.xyy) - ring(hp - e.xyy), ring(hp + e.yxy) - ring(hp - e.yxy), ring(hp + e.yyx) - ring(hp - e.yyx)));
    float fr = pow(1. - abs(dot(n, -rd)), 2.);
    vec3 q = toLocal(hp - C);
    float side = .55 + .45 * dot(normalize(vec3(q.x, 0., q.z)), normalize(toLocal(L) * vec3(1, 0, 1)));
    float aa = atan(q.z, q.x); if (aa < 0.) aa += 6.2832;
    float s = clamp((aa - GAP) / (6.2832 - 2. * GAP), 0., 1.);
    vec3 core = mix(CYAN * 1.2, BRAND * 1.4, s) * mix(.5, 1.8, s) * (.7 + .5 * side);
    vec3 rc = core * (.8 + .4 * max(dot(n, L), 0.)) + mix(CYAN, ORCHID, smoothstep(-.3, .3, n.y)) * fr * 1.2;
    col = rc;
  }
  // ring glow (bloom), stronger on the lit side
  col += (BRAND * .7 + ICE * .4) * exp(-max(dmin, 0.) / .010) * .5 * step(tr, 0.);
  col += BRAND * .15 * exp(-max(dmin, 0.) / .06) * step(tr, 0.) * step(dmin, 1e8);

  // moon
  vec3 MC = vec3(2.05, .02, -8.);
  vec2 ms = sphere(ro, rd, MC, .09);
  if (ms.x > 0. && (tp < 0. || ms.x < tp)) {
    vec3 n = normalize(ro + rd * ms.x - MC);
    float dl = dot(n, L);
    col = mix(ABYSS * .6, ICE * 1.2, smoothstep(-.1, .6, dl)) + (CYAN * .3 + WARM * .3) * pow(1. - max(dot(n, -rd), 0.), 3.);
  }
  fragColor = finish(col, frag);
}
