// Line art: bundles of rounded orbit lines sweep in from two corners in
// a gradient of the brand blues, cyan and orchid; one small planet rides
// along. Plenty of quiet night-blue space in between.
float bundle(vec2 p, vec2 c, float r0, float gap, float px, out float t) {
  float r = length(p - c), k = (r - r0) / gap;
  float i = floor(k + .5);
  t = clamp(i / 5., 0., 1.);
  if (i < 0. || i > 5.) return 0.;
  float d = abs(r - (r0 + i * gap));
  return smoothstep(2.2 * px, .6 * px, d);
}
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution, p = (frag - .5 * res) / res.y;
  float px = 1. / res.y, ar = res.x / res.y;
  vec3 col = mix(hex(0x162043), hex(0x1d2b58), smoothstep(-.7, .7, p.y - p.x * .4));
  col += BRAND * .05 * exp(-dot(p, p) / .25);
  float t;
  vec2 c1 = vec2(-.5 * ar - .05, -.62);
  float l1 = bundle(p, c1, .42, .028, px, t);
  vec3 g1 = mix(mix(CYAN, BRAND, t), ORCHID, smoothstep(.0, .9, (p.y + .5) * 1.4) * .6);
  col = mix(col, g1, l1 * .9);
  col += mix(CYAN, BRAND, .5) * .05 * exp(-abs(length(p - c1) - .49) / .06);
  vec2 c2 = vec2(.5 * ar + .08, .64);
  float l2 = bundle(p, c2, .36, .028, px, t);
  vec3 g2 = mix(mix(ORCHID, BRAND, t), CYAN, smoothstep(.0, .9, (.5 - p.y) * 1.4) * .5);
  col = mix(col, g2, l2 * .9);
  // a planet on the outer track of the lower-left bundle
  vec2 pp = c1 + vec2(cos(radians(38.)), sin(radians(38.))) * (.42 + 5. * .028);
  float dp = length(p - pp);
  col = mix(col, mix(ICE, HAZE, .5), smoothstep(.016 + px, .016 - px, dp));
  col += BRAND * .12 * exp(-dp / .03);
  col += vec3(.8, .88, 1.) * stars(frag, .03, res.y / 40.) * .35;
  fragColor = finish(col * 1.5, frag);
}
