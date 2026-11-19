// Veils of light: soft folded ribbons of blue, cyan and violet light flow
// across the screen like an aurora, over a starry night.
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution, p = (frag - .5 * res) / res.y;
  vec3 col = mix(hex(0x0c1235), hex(0x18205a), smoothstep(.5, -.6, p.y - p.x * .3)) * .9;
  col += vec3(.85, .9, 1.) * stars(frag, .05, res.y / 55.) * .7 * smoothstep(-.1, .4, p.y - p.x * .5);
  vec2 w = vec2(fbm2(p * 1.2 + 2., 4), fbm2(p * 1.2 + 9., 4)) - .5;
  vec2 q = p + w * .45;
  vec3 acc = vec3(0);
  for (int i = 0; i < 6; i++) {
    float fi = float(i);
    float y0 = -.25 + fi * .07 + .5 * (q.x + .2) * (.6 + .1 * fi);
    float cy = y0 + .12 * sin(q.x * (2.2 + fi * .3) + fi * 1.7);
    float dy = q.y - cy;
    float wdt = .05 + .04 * hash12(vec2(fi, 3.));
    float sheet = exp(-dy * dy / (wdt * wdt));
    // curtain rays rising from the ribbon
    float rays = .6 + .4 * fbm(vec3(q.x * 18. + fi * 3., dy * 2., fi), 3);
    float up = smoothstep(-.02, .0, dy) * exp(-max(dy, 0.) / (.18 + .08 * fi));
    float lum = (sheet * .9 + up * .35) * rays;
    vec3 c = fi < 2. ? CYAN : fi < 4. ? mix(BRAND, SKY, .4) : VIOLET;
    acc += c * lum * (.2 + .04 * fi);
  }
  float pink = exp(-dot(p - vec2(.55, -.42), p - vec2(.55, -.42)) / .12);
  acc += mix(ORCHID, hex(0xff9ad5), .4) * pink * .22 * (.5 + fbm2(q * 3., 4));
  col += acc;
  fragColor = finish(col, frag);
}
