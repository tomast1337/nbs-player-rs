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

vec2 hash(vec2 p) {
  p = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
  return fract(sin(p) * 18.5453);
}

// Voronoi function with animated jitter
vec2 voronoi(vec2 x) {
  vec2 n = floor(x);
  vec2 f = fract(x);

  vec3 m = vec3(8.0);
  for (int j = -1; j <= 1; j++) {
    for (int i = -1; i <= 1; i++) {
      vec2 g = vec2(float(i), float(j));
      vec2 o = hash(n + g);
      vec2 r = g - f + (0.5 + 0.5 * sin(iTime * speed + 6.2831 * o));
      float d = dot(r, r);
      if (d < m.x)
        m = vec3(d, o);
    }
  }

  return vec2(sqrt(m.x), m.y + m.z);
}

void main() {
  vec2 uv = gl_FragCoord.xy / iResolution.xy;
  uv.x *= iResolution.x / iResolution.y; // correct aspect ratio

  // Scroll vertically over time (controlled by speed)
  vec2 scrollUV = vec2(uv.x, uv.y + iTime * speed * 0.25);

  // Fixed scale Voronoi
  vec2 c = voronoi(12.0 * scrollUV);

  // Color blending
  vec3 base = mix(background_color, accent_color, smoothstep(0.0, 1.0, c.y));
  vec3 keyMix = mix(white_key_color, black_key_color, smoothstep(0.0, 1.0, c.y));

  float edge = 1.0 - smoothstep(0.08, 0.12, c.x);

  vec3 color = base * (1.0 - c.x * 0.4) + edge * keyMix;


  gl_FragColor = vec4(color, 1.0);
}
