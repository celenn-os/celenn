// A giant ringed planet rising from the lower right corner, its thin rings
// sweeping across the frame, under a quiet night-blue sky with a few stars.
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution, p = (frag - .5 * res) / res.y;
  float px = 1. / 720., ar = res.x / res.y;
  vec3 col = skyBase(p);
  col += vec3(.85, .9, 1.) * stars(frag, .035, res.y / 55.) * .55;
  vec2 c = vec2(.5 * ar - .36, -.3);           // planet centre, near the lower right corner
  float pr = .36;                               // planet radius
  vec2 q = p - c;
  float r = length(q);
  // light comes from the upper left
  vec2 ld = normalize(vec2(-.8, .6));
  // the rings: a flattened ellipse tilted with the planet
  vec2 rq = rot(radians(16.)) * q;
  vec2 e = vec2(rq.x, rq.y / .24) / pr;
  float er = length(e);
  float band = smoothstep(1.32, 1.36, er) * smoothstep(1.98, 1.92, er);
  float grooves = .55 + .45 * sin(er * 150.) * sin(er * 41. + 1.);
  float gap = smoothstep(.012, .0, abs(er - 1.72)) ;
  float ring = band * grooves * (1. - .85 * gap);
  vec3 rc = mix(CYAN, ORCHID, smoothstep(1.32, 1.98, er) * .7);
  bool front = rq.y < 0.;                      // near half of the rings, over the planet
  // planet body
  float body = smoothstep(pr + px, pr - px, r);
  vec3 n = vec3(q / pr, sqrt(max(0., 1. - dot(q, q) / (pr * pr))));
  float lit = clamp(dot(n, normalize(vec3(ld, .7))), 0., 1.);
  float lat = rq.y / pr;
  float bands = fbm2(vec2(lat * 9., lat * 2. + q.x * .6), 4);
  vec3 pc = mix(ABYSS, DEEP, .35 + .4 * bands) * (.12 + .8 * lit);
  pc += SKY * .35 * pow(1. - n.z, 3.) * (.3 + lit);   // atmosphere rim
  // rings behind the planet
  if (!front) col += rc * ring * .4 * (1. - body);
  col = mix(col, pc, body);
  // ring shadow on the planet
  col *= 1. - body * .45 * smoothstep(.05, .0, abs(rq.y + .025));
  if (front) col = mix(col, rc * .55, ring * .5 * body) + rc * ring * .4 * (1. - body);
  // halo around the planet
  col += BRAND * .14 * exp(-max(r - pr, 0.) / .07) * (1. - body);
  fragColor = finish(col, frag);
}
