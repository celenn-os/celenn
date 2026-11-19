// Deep-sea blue space: veils of glowing gas drift around a small cluster
// of young stars, fine filaments catch the light.
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution, p = (frag - .5 * res) / res.y;
  vec2 c = vec2(.28, .18);
  vec2 w = vec2(fbm2(p * 1.4, 5), fbm2(p * 1.4 + 6.2, 5)) - .5;
  float cloud = fbm(vec3((p + w * .6) * 1.8, 1.7), 6);
  float glow = exp(-dot(p - c, p - c) / .18);
  vec3 col = mix(INK, DEEP, .45) * .4 + DEEP * pow(cloud, 1.5) * .4;
  col += mix(DEEP, BRAND, glow) * pow(cloud, 2.) * glow * 1.8;
  float fil = ridged(vec3((p + w) * 3., 5.), 6);
  col += mix(BRAND, SKY, .5) * pow(fil, 5.) * (.15 + glow) * .9;
  col += mix(SKY, HAZE, .5) * exp(-length(p - c) / .05) * .6;
  // the cluster
  for (int i = 0; i < 18; i++) {
    float fi = float(i);
    vec2 o = c + (vec2(hash12(vec2(fi, 1.)), hash12(vec2(fi, 2.))) - .5) * .22;
    float b = pow(hash12(vec2(fi, 3.)), 3.) * 2. + .2;
    float d = length(p - o) * res.y / 1440.;
    col += ICE * b * (exp(-d * d / 2e-6) + .05 * exp(-d / .006));
  }
  col += vec3(.8, .88, 1.) * stars(frag, .05, res.y / 55.) * .7;
  fragColor = finish(col, frag);
}
