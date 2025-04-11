uniform vec2 iResolution;
uniform float iTime;
uniform float speed;
uniform vec3 background_color;
uniform vec3 accent_color;
uniform vec3 text_color;
uniform vec3 white_key_color;
uniform vec3 black_key_color;
uniform vec3 white_text_key_color;
uniform vec3 black_text_key_color;

float hash(vec2 p) {
  p = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
  return fract(sin(p.x + p.y) * 43758.5453);
}

float noise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  return mix(mix(hash(i + vec2(0.0, 0.0)), hash(i + vec2(1.0, 0.0)), f.x),
             mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), f.x), f.y);
}

void main() {
  // Pixelated UVs
  vec2 uv = floor(gl_FragCoord.xy / 10.0) * 10.0 / iResolution.xy;
  uv.x *= iResolution.x / iResolution.y;

  float t = iTime * speed;
  uv.y -= t * 0.4;
  uv.x += sin(uv.y * 2.0 + t) * 0.1;

  float flameBase = 1.0 - uv.y;
  float n = noise(vec2(uv.x * 4.0, uv.y * 2.0 + t));
  n += noise(vec2(uv.x * 8.0, uv.y * 4.0 + t * 1.5)) * 0.5;
  n /= 1.5;
  n = clamp(n, 0.0, 1.0);

  float flame = flameBase * n;
  flame = floor(flame * 4.0) / 4.0; // Quantize for palette
  vec3 flameColor = mix(background_color, accent_color, flame);

  float flicker = noise(vec2(uv.x * 6.0, t * 3.0));
  flameColor += vec3(flicker * 0.2) * (1.0 - uv.y);
  flameColor = clamp(flameColor, 0.0, 1.0);

  float alpha = smoothstep(0.1, 0.6, flameBase);
  flameColor *= alpha;

  gl_FragColor = vec4(flameColor, 1.0);
}