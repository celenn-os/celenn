// An eclipse: a dark planet in front of its star, a ring of blue light
// around it with long, soft rays, high on the left of a deep blue sky.
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution, p = (frag - .5 * res) / res.y;
  vec3 col = skyBase(p);
  col += vec3(.85, .9, 1.) * stars(frag, .04, res.y / 60.) * .5;
  vec2 c = vec2(-.38, .1);
  float pr = .17;
  vec2 q = p - c;
  float r = length(q), a = atan(q.y, q.x);
  float out_ = max(r - pr, 0.);
  // corona: soft glow with streamers
  float stream = fbm(vec3(a * 3., 1.7, 0.), 4);
  float stream2 = ridged(vec3(a * 7., out_ * 6., 2.3), 4);
  float corona = exp(-out_ / (.05 + .09 * stream)) * (.6 + .6 * stream2);
  col += mix(BRAND, CYAN, .45) * corona * .55;
  col += mix(VIOLET, BRAND, .5) * exp(-out_ / .45) * .12;
  // bright thin rim, brightest at the lower right (the "diamond")
  float rim = exp(-abs(r - pr) / .0035);
  float diamond = pow(max(0., dot(normalize(q), normalize(vec2(.7, -.7)))), 30.);
  col += ICE * rim * (.8 + 2. * diamond);
  col += HAZE * diamond * exp(-out_ / .02) * 1.2;
  // the planet: dark, with a faint earthshine
  float body = smoothstep(pr + .001, pr - .001, r);
  col = mix(col, INK * (.7 + .5 * fbm2(q * 18., 3)) + ABYSS * .15, body);
  fragColor = finish(col, frag);
}
