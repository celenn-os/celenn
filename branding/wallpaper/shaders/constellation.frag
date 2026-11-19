// Flat illustration: a calm night-blue field, a large tone-on-tone spiral,
// small round stars and a constellation drawing the orbit "c" of the logo.
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution, p = (frag - .5 * res) / res.y;
  float px = 1. / res.y;
  vec3 col = mix(hex(0x14234a), hex(0x1a2f63), smoothstep(-.6, .6, p.y + p.x * .3));
  // tone-on-tone spiral
  vec2 d = rot(radians(-15.)) * (p - vec2(-.1, 0.)); d.y /= .7;
  float r = length(d), u = atan(d.y, d.x) - log(r + 1e-3) * 2.2;
  float arm = smoothstep(.2, .9, .5 + .5 * cos(2. * u + fbm2(d * 3., 3) * 1.5));
  col = mix(col, hex(0x22407e), arm * smoothstep(1.1, .15, r) * .55);
  col = mix(col, hex(0x2a4e94), smoothstep(.35, 0., r) * .5);
  // flat stars
  vec2 g = floor(frag / (res.y / 26.)), f = fract(frag / (res.y / 26.));
  float h = hash12(g);
  if (h < .45) {
    vec2 c = vec2(hash12(g + 7.1), hash12(g + 3.3)) * .7 + .15;
    float rad = mix(.8, 2.6, pow(hash12(g + 1.9), 3.)) * res.y / 1440.;
    float s = smoothstep(rad + .8, rad - .8, length(f - c) * res.y / 26.);
    col = mix(col, mix(SKY, HAZE, hash12(g + 4.)), s * mix(.35, .9, hash12(g + 5.)));
  }
  // constellation along an open "c"
  vec2 cc = vec2(.38, .08);
  float line = 1e9, nodes = 1e9;
  vec2 prev;
  for (int i = 0; i < 8; i++) {
    float fi = float(i);
    float a = radians(40. + fi * 40.) + (hash12(vec2(fi, 2.)) - .5) * .25;
    vec2 q = cc + vec2(cos(a), sin(a)) * .24 * (1. + (hash12(vec2(fi, 9.)) - .5) * .18);
    if (i > 0) line = min(line, sdSeg(p, prev, q));
    nodes = min(nodes, length(p - q));
    prev = q;
  }
  col = mix(col, mix(SKY, ICE, .4), smoothstep(1.4 * px, .2 * px, line) * .55);
  // four-point stars on the nodes
  vec2 n = vec2(0);
  float best = 1e9;
  for (int i = 0; i < 8; i++) {
    float fi = float(i);
    float a = radians(40. + fi * 40.) + (hash12(vec2(fi, 2.)) - .5) * .25;
    vec2 q = cc + vec2(cos(a), sin(a)) * .24 * (1. + (hash12(vec2(fi, 9.)) - .5) * .18);
    if (length(p - q) < best) { best = length(p - q); n = p - q; }
  }
  float sz = .011;
  vec2 an = abs(n) / sz;
  float star = pow(an.x, .5) + pow(an.y, .5);
  col = mix(col, mix(WARM, HAZE, .5), smoothstep(1.05, .95, star));
  col = mix(col, mix(BRAND, SKY, .5), (1. - smoothstep(1.5, 1.6, star)) * smoothstep(.95, 1.05, star) * .0);
  col += ICE * .06 * exp(-best / .02);
  fragColor = finish(col * 1.6, frag);
}
