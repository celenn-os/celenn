// Flat illustration: a calm night-blue field, a large tone-on-tone spiral
// and small round stars.
// SPIRAL = centre of the spiral (frame heights).
#ifndef SPIRAL
#define SPIRAL vec2(0., -.02)
#endif
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution, p = (frag - .5 * res) / res.y;
  vec3 col = mix(hex(0x14234a), hex(0x1a2f63), smoothstep(-.6, .6, p.y + p.x * .3));
  // tone-on-tone spiral
  vec2 d = rot(radians(-15.)) * (p - SPIRAL); d.y /= .7;
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
  fragColor = finish(col * 1.6, frag);
}
