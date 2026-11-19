// A luminous spiral galaxy seen at an angle, its arms made of fine
// streaks of stars, crossing the frame on a deep violet-blue sky.
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution, p = (frag - .5 * res) / res.y;
  vec3 col = skyBase(p);
  col += vec3(.85, .9, 1.) * stars(frag, .04, res.y / 60.) * .6;
  vec2 c = vec2(.05, .02);
  vec2 d = rot(radians(-24.)) * (p - c);
  d.y /= .36;                                   // inclination
  float r = length(d), th = atan(d.y, d.x);
  float lr = log(r + 1e-4);
  float u = th - lr * 2.6;                      // logarithmic spiral coordinate
  float arm = pow(.5 + .5 * cos(2. * u), 3.);
  float streak = ridged(vec3(u * 3.2, lr * 26., 1.3), 5);   // streaks along the arms
  float fine = ridged(vec3(u * 9., lr * 70., 4.1), 3);
  float disk = exp(-r / .42) * smoothstep(1.25, .2, r);
  float dens = disk * (.25 + 1.2 * arm) * (.35 + .9 * streak + .4 * fine);
  vec3 armc = mix(BRAND, CYAN, .35 + .3 * streak);
  col += armc * dens * 1.4;
  // sparkling stars caught in the arms
  vec2 sg = vec2(u * 60., lr * 140.);
  float sp = step(.965, hash12(floor(sg))) * exp(-dot(fract(sg) - .5, fract(sg) - .5) * 30.);
  col += ICE * sp * disk * (.4 + arm) * 2.;
  // glowing core
  col += HAZE * 2.2 * exp(-r / .07) + mix(CYAN, ICE, .5) * exp(-r / .2) * .7;
  fragColor = finish(col, frag);
}
