// Clay planets: a small solar system modelled in soft matte clay, floating
// in a dark blue studio, one planet with a ring.
struct P { vec3 c; float r; vec3 col; };
const int N = 6;
vec3 Pc(int i) {
  if (i == 0) return vec3(-1.9, -.15, -7.);
  if (i == 1) return vec3(-1.05, .32, -7.5);
  if (i == 2) return vec3(-.15, -.05, -6.8);
  if (i == 3) return vec3(.95, .25, -7.2);
  if (i == 4) return vec3(1.75, -.25, -7.);
  return vec3(2.35, .30, -7.4);
}
float Pr(int i) { return i == 0 ? .30 : i == 1 ? .22 : i == 2 ? .55 : i == 3 ? .36 : i == 4 ? .18 : .12; }
vec3 Pcol(int i) { return i == 0 ? SKY : i == 1 ? ORCHID : i == 2 ? BRAND : i == 3 ? ICE : i == 4 ? CYAN * .8 : WARM; }
float ringSd(vec3 p) {  // ring around planet 2
  vec3 q = p - Pc(2);
  float c = cos(radians(14.)), s = sin(radians(70.)); q.yz = mat2(c, s, -s, c) * q.yz;
  float c2 = cos(radians(-12.)), s2 = sin(radians(-12.)); q.xy = mat2(c2, s2, -s2, c2) * q.xy;
  vec2 t = vec2(length(q.xz) - .95, q.y);
  return length(max(abs(t) - vec2(.22, .006), 0.)) - .01;
}
float map(vec3 p, out int id) {
  float d = 1e9; id = -1;
  for (int i = 0; i < N; i++) { float di = length(p - Pc(i)) - Pr(i); if (di < d) { d = di; id = i; } }
  float r = ringSd(p); if (r < d) { d = r; id = 9; }
  return d;
}
float mapd(vec3 p) { int i; return map(p, i); }
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution;
  vec2 uv = frag / res, p = (frag - .5 * res) / res.y;
  vec3 col = mix(hex(0x101a44), hex(0x1c2c6e), smoothstep(-.5, .6, p.y)) * (.85 + .25 * fbm2(p, 3));
  col += BRAND * .10 * exp(-dot(p - vec2(-.2, .2), p - vec2(-.2, .2)) / .2);
  vec3 ro = vec3(0), rd = normalize(vec3(p, -2.2));
  float t = 4.; int id = -1; bool hit = false;
  for (int i = 0; i < 140; i++) { float d = map(ro + rd * t, id); if (d < .0005) { hit = true; break; } t += d; if (t > 12.) break; }
  if (hit) {
    vec3 pos = ro + rd * t; vec2 e = vec2(.001, 0);
    vec3 n = normalize(vec3(mapd(pos + e.xyy) - mapd(pos - e.xyy), mapd(pos + e.yxy) - mapd(pos - e.yxy), mapd(pos + e.yyx) - mapd(pos - e.yyx)));
    vec3 base = id == 9 ? mix(ICE, SKY, .3) : Pcol(id);
    vec3 L = normalize(vec3(-.5, .75, .45));
    float dif = max(dot(n, L), 0.);
    // soft shadows and ambient occlusion
    float sh = 1., st = .02;
    for (int k = 0; k < 40; k++) { float h = mapd(pos + L * st); sh = min(sh, 12. * h / st); st += clamp(h, .02, .2); if (sh < .01 || st > 4.) break; }
    sh = clamp(sh, 0., 1.);
    float ao = 0.; for (int k = 1; k <= 5; k++) { float hh = .06 * float(k); ao += (hh - mapd(pos + n * hh)) / float(k); }
    ao = clamp(1. - ao * 3., 0., 1.);
    // clay: subtle surface noise, wrap lighting, cool fill from below, rim
    float clayN = fbm(pos * 9., 4);
    vec3 c = base * (.18 + .95 * dif * sh) * (.92 + .16 * clayN);
    c += base * DEEP * .6 * max(dot(n, vec3(0, -1, .3)), 0.);
    c += mix(CYAN, ICE, .5) * .25 * pow(1. - max(dot(n, -rd), 0.), 4.);
    col = c * ao;
  }
  fragColor = finish(col, frag);
}
