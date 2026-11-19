// Long exposure: thousands of orbits traced as light trails form a tilted,
// luminous disk around an unseen star.
void main() {
  vec2 frag = gl_FragCoord.xy, res = iResolution;
  vec2 uv = frag / res, p = (frag - .5 * res) / res.y;
  vec2 c = vec2(.42, .03);
  float pk = pocketMask(p, c, vec2(.95, .30));
  vec3 col = mix(spaceBase(p), mix(INK, ABYSS, .6), pk * .9);
  col += vec3(.9, .95, 1.) * stars(frag, .05, res.y / 48.) * pk * .7;
  float tilt = radians(-9.), f = .17;
  vec2 q0 = mat2(cos(tilt), sin(tilt), -sin(tilt), cos(tilt)) * (p - c);
  vec3 acc = vec3(0);
  float px = 1. / res.y;
  for (int i = 0; i < 160; i++) {
    float fi = float(i);
    float h1 = hash12(vec2(fi, 1.3)), h2 = hash12(vec2(fi, 7.7)), h3 = hash12(vec2(fi, 3.1)), h4 = hash12(vec2(fi, 9.4));
    float a = mix(.45, 1.35, h1);
    vec2 q = q0 / vec2(a, a * f);
    float k = length(q);
    float dist = abs(k - 1.) * a * f;
    float far = q0.y > 0. ? 1. : 0.;
    float wdt = mix(.6, 2.6, pow(h3, 3.)) * px * mix(1., 2.2, far);
    if (dist > wdt * 6.) continue;
    float ang = atan(q.y, q.x);
    float start = h2 * 6.2832, len = mix(.6, 2.8, h4);
    float t = mod(ang - start, 6.2832) / len;     // 0 at the tail, 1 at the head
    if (t > 1.) continue;
    float br = pow(t, 2.) * (.15 + 1.6 * pow(h3, 4.)) * mix(1., .45, far);
    float hc = hash12(vec2(fi, 5.5));
    vec3 tc = hc < .45 ? mix(BRAND, ICE, .5) : hc < .70 ? CYAN : hc < .78 ? ORCHID : hc < .81 ? WARM : SKY * .15;
    acc += tc * br * exp(-pow(dist / wdt, 2.)) * 1.2 + tc * br * exp(-dist / (wdt * 5.)) * .08;
  }
  col += acc;
  float rc = length(q0 / vec2(1., f));
  col += ICE * 1.4 * exp(-rc / .010) + WARM * .3 * exp(-rc / .04) + SKY * .15 * exp(-rc / .35);
  fragColor = finish(col, frag);
}
