// A diagonal river of stars and glowing gas, cut by dark dust lanes,
// across a deep blue sky.
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution, p = (frag - .5 * res) / res.y;
  vec3 col = skyBase(p) * .8;
  vec2 q = rot(radians(18.)) * (p - vec2(.05, .0));
  float wob = (fbm2(vec2(q.x * 1.3, 2.), 4) - .5) * .25;
  float y = q.y - wob;
  float band = exp(-y * y / .02) * (.5 + .5 * fbm2(q * 2. + 4., 5));
  float core = exp(-dot(q - vec2(.12, 0.), q - vec2(.12, 0.)) / .08);
  float gas = fbm(vec3(q * 4., .8), 6);
  float wisp = ridged(vec3(q * vec2(3., 7.) + 2., 1.), 6);
  vec3 glow = mix(DEEP, mix(BRAND, ICE, core), .4 + .6 * gas) * band * pow(gas, 1.6) * (1.2 + 1.2 * core) * (.6 + .8 * wisp);
  float dust = smoothstep(.45, .75, fbm(vec3(q * vec2(3., 9.) + vec2(0., wob * 4.), 3.), 6)) * exp(-pow((y + .015) / .07, 2.));
  col += glow * (1. - .85 * dust) * .75;
  float dense = band * 1.5 + .25;
  col += vec3(.85, .9, 1.) * stars(frag, .10 * dense, res.y / 110.) * .8;
  col += vec3(.9, .95, 1.) * stars(frag + 37., .03, res.y / 30.) * 1.2;
  fragColor = finish(col, frag);
}
