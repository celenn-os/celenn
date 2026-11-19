// Gravity well: a grid of light bent by a planet's mass, the way spacetime
// is drawn in physics, with the planet glowing at the bottom of the well.
float height(vec2 xz) {
  float r = length(xz - vec2(1.9, -8.));
  return -1.25 / sqrt(r * r + .35) + .02 * sin(xz.x * .5);
}
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution;
  vec2 uv = frag / res, p = (frag - .5 * res) / res.y;
  vec3 col = mix(hex(0x0f1a46), hex(0x1d2f78), smoothstep(-.2, .6, p.y));
  col += vec3(.9, .95, 1.) * stars(frag, .05, res.y / 48.) * smoothstep(.0, .3, p.y) * .7;
  vec3 ro = vec3(0., 1.15, 0.);
  vec3 rd = normalize(vec3(p.x, p.y - .32, -1.6));
  float t = .1, hit = -1.;
  for (int i = 0; i < 300; i++) {
    vec3 q = ro + rd * t;
    float dh = q.y - height(q.xz);
    if (dh < .002) { hit = t; break; }
    t += max(dh * .35, .004);
    if (t > 40.) break;
  }
  if (hit > 0.) {
    vec3 q = ro + rd * hit;
    vec2 g = q.xz * 2.2;
    vec2 fw = fwidth(g);
    vec2 gl = abs(fract(g - .5) - .5) / max(fw, 1e-4);
    float line = 1. - min(min(gl.x, gl.y), 1.);
    float fog = exp(-hit * .07);
    float depthC = smoothstep(.0, -1.6, height(q.xz));
    vec3 lc = mix(BRAND, CYAN, depthC) * (1.2 + 1.5 * depthC);
    vec3 surf = mix(hex(0x0c1638), DEEP * .5, depthC * .6);
    col = mix(col, surf + lc * line * .9, fog);
  }
  // the planet at the bottom of the well
  vec3 pc = vec3(1.9, -1.30, -8.);
  vec3 oc = ro - pc; float b = dot(oc, rd), h = b * b - dot(oc, oc) + .55 * .55;
  if (h > 0.) {
    float tp = -b - sqrt(h);
    if (hit < 0. || tp < hit) {
      vec3 n = normalize(ro + rd * tp - pc);
      float dif = max(dot(n, normalize(vec3(-.5, .8, .5))), 0.);
      col = mix(ABYSS, mix(SKY, ICE, .5), dif) + CYAN * .5 * pow(1. - max(dot(n, -rd), 0.), 3.);
    }
  }
  float dd = length(cross(pc - ro, rd));
  col += BRAND * .35 * exp(-max(dd - .55, 0.) / .25);
  fragColor = finish(col, frag);
}
