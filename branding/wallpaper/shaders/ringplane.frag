// Ring plane: just above the pale cloud tops of a gas giant, its rings rise
// as a thin luminous band, backlit by a sun just off the right edge.
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution;
  vec2 uv = frag / res, p = (frag - .5 * res) / res.y;
  float A = res.x / res.y;
  // sky: light at the top, deeper towards the middle, haze at the horizon
  vec3 col = mix(spaceBase(p), INK * .55, pocketMask(p, vec2(.1, .05), vec2(.95, .36)) * .92);
  float hor = .27 - .55 * pow(uv.x - .35, 2.) - .03 * uv.x;
  col = mix(col, mix(SKY, BRAND, .5), exp(-max(uv.y - hor, 0.) / .06) * .8);
  vec2 sun = vec2(1.04, .52);
  float ds = length((uv - sun) * vec2(A, 1.));
  col += (WARM * .5 + ICE * .5) * exp(-ds / .18) * .35 + ICE * exp(-ds / .04) * .5;
  col += vec3(.9, .95, 1.) * stars(frag, .03, res.y / 40.) * pocketMask(p, vec2(.1, .05), vec2(.95, .36)) * .6;
  // the ring band, from lower left to upper right
  // an arc of a large ellipse, from lower left to upper right
  vec2 ec = vec2(1.6, -1.4), er = vec2(2.6, 1.9);
  vec2 qe = (p - ec) / er;
  float across = (length(qe) - 1.) * min(er.x, er.y);
  float along = clamp((p.x + .5 * A) / A, 0., 1.);
  float w = mix(.13, .012, along);                  // thicker on the near side
  float r = across / w;                             // -1..1 across the band
  if (abs(r) < 1.3) {
    float rr = (r + 1.) * .5;
    float fine = vnoise(vec3(rr * 90., .5, 0.)) * .6 + vnoise(vec3(rr * 260., 3., 0.)) * .4;
    float coarse = smoothstep(.0, .06, rr) * smoothstep(1., .9, rr) * (1. - .9 * exp(-pow((rr - .38) / .025, 2.))) * (1. - .9 * exp(-pow((rr - .66) / .03, 2.)));
    float dens = coarse * (.35 + .9 * fine);
    float fwd = .35 + 1.4 * pow(along, 3.) + .6 * exp(-ds / .25);  // forward scattering
    vec3 rc = mix(ICE, WARM, .25 * smoothstep(.5, 1., along)) * fwd;
    // the planet's shadow across the rings
    float sh = mix(.5, 1., smoothstep(.0, .12, abs(along - .30 + across * 1.5 + pow(along - .3, 2.)) - .04));
    float fog = mix(.55, 1., smoothstep(.0, .5, along));
    col = mix(col, rc * sh, clamp(dens * .9 * fog, 0., 1.));
    col += SKY * .10 * coarse * fwd;
  }
  // cloud tops below the horizon, in perspective
  if (uv.y < hor + .01) {
    float h = max(hor - uv.y, 0.);
    float z = 1. / (h + .018);
    vec2 cp = vec2((uv.x - .5) * z * .55, z * .55);
    float c = fbm(vec3(cp * vec2(.25, 5.), 2.), 6) * .6 + fbm(vec3(cp * vec2(.8, 2.4), 2.), 7) * .4;
    float c2 = fbm(vec3(cp * vec2(2.5, 7.), 5.), 5);
    float shade = .78 + .65 * (c - .5) + .2 * (c2 - .5);
    vec3 cl = mix(mix(DEEP, VIOLET, .25), SKY, .35) * shade * (.8 + .4 * c2) * .75;
    cl = mix(cl, cl * mix(ICE, WARM, .3) * 1.3, smoothstep(.3, 1., uv.x));
    cl += ORCHID * .15 * smoothstep(.5, 0., uv.x);
    float haze = exp(-h / .035);
    cl = mix(cl, mix(SKY, ICE, .4), haze * .45);
    col = mix(col, cl, smoothstep(hor + .004, hor - .004, uv.y));
  }
  fragColor = finish(col, frag);
}
