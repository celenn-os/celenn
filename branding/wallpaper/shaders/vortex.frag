// A gravitational vortex: twisted filaments of blue gas spiralling into a
// dark core, with fainter wisps reaching out to the edges.
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution, p = (frag - .5 * res) / res.y;
  vec2 c = vec2(.12, .02);
  vec2 q = p - c;
  q.y /= .82;
  float r = length(q), th = atan(q.y, q.x);
  vec3 col = skyBase(p) * .9;
  col += vec3(.85, .9, 1.) * stars(frag, .05, res.y / 60.) * .55;
  float tw = th + 1.1 / (r + .18);              // twist, stronger near the core
  vec3 sp = vec3(cos(tw) * r, sin(tw) * r, 0.) * 3.2;
  vec2 w = vec2(fbm(sp + 1.3, 4), fbm(sp + 8.1, 4)) - .5;
  float tw2 = tw + w.x * .6;
  float fil = ridged(vec3(cos(tw2) * 1.6, sin(tw2) * 1.6, (r + w.y * .08) * 14.), 6);
  float ring = exp(-pow((r - .34) / .16, 2.)) + .35 * exp(-r / .9);
  float g = pow(fil, 3.2) * ring * 1.3;
  float hole = smoothstep(.13, .26, r);
  col += mix(DEEP, CYAN, smoothstep(.2, .9, g)) * g * 2.6 * hole;
  col += BRAND * .25 * exp(-pow((r - .3) / .2, 2.)) * hole;
  col *= mix(.15, 1., hole);
  // wisps out in the frame
  float far = ridged(vec3(p * 2.2 + w, 2.), 5);
  col += mix(DEEP, BRAND, .4) * pow(far, 3.) * .35 * smoothstep(.2, .7, r);
  fragColor = finish(col, frag);
}
