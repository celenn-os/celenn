// Shared helpers for the cosmic-blue wallpaper shaders.
// Palette (sRGB hex -> linear-ish floats are converted with srgb()).
vec3 srgb(vec3 c) { return pow(c, vec3(2.2)); }
vec3 hex(int h) { return srgb(vec3(float((h >> 16) & 255), float((h >> 8) & 255), float(h & 255)) / 255.); }
#define INK hex(0x0a1030)
#define ABYSS hex(0x13205c)
#define DEEP hex(0x1d47c4)
#define BRAND hex(0x4f8dff)
#define SKY hex(0x8fb6ff)
#define ICE hex(0xcfe0ff)
#define HAZE hex(0xeef3ff)
#define VIOLET hex(0x7b6cff)
#define ORCHID hex(0xb9a4ff)
#define CYAN hex(0x5fe3ff)
#define WARM hex(0xffc6a3)

float hash12(vec2 p) { vec3 p3 = fract(vec3(p.xyx) * .1031); p3 += dot(p3, p3.yzx + 33.33); return fract((p3.x + p3.y) * p3.z); }
float hash13(vec3 p3) { p3 = fract(p3 * .1031); p3 += dot(p3, p3.zyx + 31.32); return fract((p3.x + p3.y) * p3.z); }
float vnoise(vec3 p) {
  vec3 i = floor(p), f = fract(p); f = f * f * (3. - 2. * f);
  return mix(mix(mix(hash13(i), hash13(i + vec3(1, 0, 0)), f.x), mix(hash13(i + vec3(0, 1, 0)), hash13(i + vec3(1, 1, 0)), f.x), f.y),
             mix(mix(hash13(i + vec3(0, 0, 1)), hash13(i + vec3(1, 0, 1)), f.x), mix(hash13(i + vec3(0, 1, 1)), hash13(i + vec3(1, 1, 1)), f.x), f.y), f.z);
}
float fbm(vec3 p, int oct) { float a = .5, s = 0.; for (int i = 0; i < 8; i++) { if (i >= oct) break; s += a * vnoise(p); p = p * 2.03 + vec3(1.7, 9.2, 3.1); a *= .5; } return s; }
float fbm2(vec2 p, int oct) { return fbm(vec3(p, 0.37), oct); }

// Stars: sparse, small, varied; returns intensity.
float stars(vec2 frag, float density, float scale) {
  vec2 g = floor(frag / scale), f = fract(frag / scale);
  float h = hash12(g);
  if (h > density) return 0.;
  vec2 c = vec2(hash12(g + 7.1), hash12(g + 3.3)) * .8 + .1;
  float r = length(f - c) * scale;
  float m = pow(hash12(g + 1.9), 6.) * 1.5 + .15;
  return m * exp(-r * r / (1.2 * iResolution.y / 1440.));
}

// Film-like finish: ACES tone map, grade (violet shadows, warm highlights),
// sRGB encode, blue-noise-ish dither and grain.
vec3 aces(vec3 x) { return clamp((x * (2.51 * x + .03)) / (x * (2.43 * x + .59) + .14), 0., 1.); }
vec4 finish(vec3 col, vec2 frag) {
  col = aces(col * 1.05);
  float l = dot(col, vec3(.2126, .7152, .0722));
  col = mix(col, col * vec3(.92, .9, 1.08), smoothstep(.35, 0., l) * .5);
  col = mix(col, col * vec3(1.04, 1.01, .97), smoothstep(.6, 1., l) * .4);
  col = pow(col, vec3(1. / 2.2));
  float n = hash12(frag) + hash12(frag + 17.3) - 1.;
  col += n * (1.6 / 255.) + (hash12(frag * 1.37 + 5.) - .5) * (3.2 / 255.);
  return vec4(col, 1.);
}

// Ridged fbm: thin bright filaments, for gas and energy wisps.
float ridged(vec3 p, int oct) {
  float a = .5, s = 0.;
  for (int i = 0; i < 8; i++) { if (i >= oct) break; float n = 1. - abs(2. * vnoise(p) - 1.); s += a * n * n; p = p * 2.07 + vec3(3.1, 1.7, 5.3); a *= .5; }
  return s;
}
mat2 rot(float a) { return mat2(cos(a), sin(a), -sin(a), cos(a)); }
// Deep blue sky that never goes black: violet at the top, blue below,
// slow clouds of colour.
vec3 skyBase(vec2 p) {
  float n = fbm2(p * .9 + 3., 4);
  vec3 c = mix(mix(DEEP, ABYSS, .45), mix(ABYSS, VIOLET, .28), smoothstep(-.6, .6, p.y + (n - .5) * .5));
  return c * (.30 + .25 * n);
}
float sdSeg(vec2 p, vec2 a, vec2 b) { vec2 pa = p - a, ba = b - a; return length(pa - ba * clamp(dot(pa, ba) / dot(ba, ba), 0., 1.)); }
