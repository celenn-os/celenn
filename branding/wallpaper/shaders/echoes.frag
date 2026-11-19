// Background for the logo mark: the logo's open orbit echoes outwards in
// fainter and fainter arcs, a few small planets ride on them, over the
// night-blue field of the deep spiral.
// LOGO = centre of the logo's orbit, R = its radius (frame heights); set
// them with SHADER_DEFINES="$(python3 logo_mark.py --defines)".
#ifndef LOGO
#define LOGO vec2(-.1389, .0208)
#define R .125
#endif
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution, p = (frag - .5 * res) / res.y;
  float px = 1. / 720.;
  vec3 col = mix(hex(0x14234a), hex(0x1a2f63), smoothstep(-.6, .6, p.y + p.x * .3));
  col = mix(col, hex(0x22407e), exp(-dot(p - LOGO, p - LOGO) / .5) * .6);
  // flat stars
  vec2 g = floor(frag / (res.y / 26.)), f = fract(frag / (res.y / 26.));
  if (hash12(g) < .40) {
    vec2 c = vec2(hash12(g + 7.1), hash12(g + 3.3)) * .7 + .15;
    float rad = mix(.8, 2.6, pow(hash12(g + 1.9), 3.)) * res.y / 1440.;
    float s = smoothstep(rad + .8, rad - .8, length(f - c) * res.y / 26.);
    col = mix(col, mix(SKY, HAZE, hash12(g + 4.)), s * mix(.3, .8, hash12(g + 5.)));
  }
  vec2 q = p - LOGO;
  float r = length(q), a = atan(q.y, q.x);
  // echoes of the orbit, open towards the right like the logo
  for (int i = 1; i <= 7; i++) {
    float fi = float(i);
    float ri = R * (1. + fi * .9 + fi * fi * .18);
    float open = radians(40. - fi * 2.5);
    float arc = smoothstep(open - .02, open + .02, abs(a));
    float w = mix(1.6, .9, fi / 7.) * px;
    float line = smoothstep(w, w * .3, abs(r - ri)) * arc;
    float fade = .55 * pow(.78, fi);
    vec3 lc = mix(CYAN, BRAND, clamp(.5 - q.y / ri * .5, 0., 1.));
    col = mix(col, lc, line * fade);
    col += lc * .05 * fade * exp(-abs(r - ri) / .01) * arc;
    // a small planet on some echoes
    if (i == 2 || i == 4 || i == 6) {
      float pa = radians(fi == 2. ? 128. : fi == 4. ? -100. : 160.);
      vec2 pp = vec2(cos(pa), sin(pa)) * ri;
      float d = length(q - pp), pr = .006 + .002 * fi / 6.;
      col = mix(col, mix(SKY, HAZE, .6), smoothstep(pr + px * .5, pr - px * .5, d) * .9);
      col += BRAND * .08 * exp(-d / .02);
    }
  }
  fragColor = finish(col * 1.6, frag);
}
