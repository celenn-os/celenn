// Glass orbit: the logo's "c" orbit as a large object of blue glass, lit
// like a studio product shot, its small planet resting inside.
const vec3 C = vec3(.55, -.05, -6.);
const float RR = 1.55, RT = .30, GAP = .75;
mat3 rotX(float a) { float c = cos(a), s = sin(a); return mat3(1, 0, 0, 0, c, s, 0, -s, c); }
mat3 rotY(float a) { float c = cos(a), s = sin(a); return mat3(c, 0, -s, 0, 1, 0, s, 0, c); }
mat3 rotZ(float a) { float c = cos(a), s = sin(a); return mat3(c, s, 0, -s, c, 0, 0, 0, 1); }
mat3 M() { return rotX(radians(62.)) * rotY(radians(-24.)) * rotZ(radians(-8.)); }
float sdC(vec3 p) {
  vec3 q = M() * (p - C);
  float a = atan(q.z, q.x);
  if (abs(a) < GAP) {
    vec3 e1 = vec3(cos(GAP), 0., sin(GAP)) * RR, e2 = vec3(cos(GAP), 0., -sin(GAP)) * RR;
    return min(length(q - e1), length(q - e2)) - RT;
  }
  return length(vec2(length(q.xz) - RR, q.y)) - RT;
}
float sdPlanet(vec3 p) { return length(p - C) - .62; }
float map(vec3 p) { return min(sdC(p), sdPlanet(p)); }
vec3 nrm(vec3 p) { vec2 e = vec2(.001, 0); return normalize(vec3(map(p + e.xyy) - map(p - e.xyy), map(p + e.yxy) - map(p - e.yxy), map(p + e.yyx) - map(p - e.yyx))); }
// a studio: dark navy room, two large soft lights
vec3 env(vec3 d) {
  vec3 col = mix(INK, ABYSS, smoothstep(-.6, .8, d.y)) * .9;
  col += ICE * 2.2 * smoothstep(.80, .97, dot(d, normalize(vec3(-.6, .7, .4))));
  col += mix(BRAND, CYAN, .4) * 1.4 * smoothstep(.85, .98, dot(d, normalize(vec3(.8, .1, .5))));
  col += VIOLET * .5 * smoothstep(.6, .95, dot(d, normalize(vec3(.1, -.9, .3))));
  return col;
}
vec3 background(vec2 uv, vec2 p) {
  vec3 col = mix(mix(DEEP, ABYSS, .55), mix(DEEP, BRAND, .2), smoothstep(.0, 1., uv.y)) * .55;
  col += mix(BRAND, VIOLET, .4) * .22 * exp(-dot(p - vec2(.3, 0.), p - vec2(.3, 0.)) / .35);
  col *= .75 + .4 * fbm2(p * 1.2, 4);
  return col;
}
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution;
  vec2 uv = frag / res, p = (frag - .5 * res) / res.y;
  vec3 ro = vec3(0), rd = normalize(vec3(p, -2.));
  vec3 col = background(uv, p);
  float t = 3., hit = -1.;
  for (int i = 0; i < 160; i++) { float d = map(ro + rd * t); if (d < .0006) { hit = t; break; } t += d; if (t > 12.) break; }
  // soft shadow-like glow of the object on the backdrop
  if (hit > 0.) {
    vec3 pos = ro + rd * hit, n = nrm(pos);
    float fr = .04 + .96 * pow(1. - max(dot(n, -rd), 0.), 5.);
    if (sdPlanet(pos) < sdC(pos)) {
      // matte pale planet
      vec3 L = normalize(vec3(-.6, .7, .4));
      float dif = max(dot(n, L), 0.);
      col = mix(ABYSS * .8, mix(ICE, SKY, .4), dif) + CYAN * .35 * pow(1. - max(dot(n, -rd), 0.), 3.);
    } else {
      // glass: reflection + tinted transmission, thicker in the middle
      vec3 rf = refract(rd, n, 1. / 1.45);
      float thick = 1. - max(dot(n, -rd), 0.);
      vec3 trans = env(rf) * mix(vec3(.55, .75, 1.), DEEP * 2.5, .45) + BRAND * .35 * (1. - thick);
      trans += CYAN * .5 * pow(thick, 3.);                      // light piped along the edges
      col = mix(trans, env(reflect(rd, n)), fr);
      // inner glow toward the planet
      col += BRAND * .25 * exp(-length(pos - C) * 1.2);
    }
  } else {
    // halo of light around the glass
    float d = map(ro + rd * 6.);
    col += BRAND * .12 * exp(-abs(d) * 3.);
  }
  fragColor = finish(col, frag);
}
