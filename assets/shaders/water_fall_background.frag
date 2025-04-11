#define FALLING_SPEED 0.25
#define STRIPES_FACTOR 30.0

// Uniforms
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

// Sphere glow function
float sphere(vec2 coord, vec2 pos, float r) {
  vec2 d = pos - coord;
  return smoothstep(60.0, 0.0, dot(d, d) - r * r);
}

void main() {
  vec2 fragCoord = gl_FragCoord.xy;
  vec2 uv = fragCoord / iResolution.xy;

  // Pixelate UV (GLSL ES 2.0-compatible round)
  vec2 clamped_uv = (floor(fragCoord / STRIPES_FACTOR + 0.5) * STRIPES_FACTOR) /
                    iResolution.xy;

  float value = fract(sin(clamped_uv.x * 10.0) * 43758.5453123);

  float stripe =
      mod(uv.y * 0.5 + (iTime * (FALLING_SPEED + value / 5.0) * speed) + value,
          0.5);
  float baseStripe = 1.0 - stripe;

  // Alternate color stripes
  vec3 col = mix(text_color, white_key_color,
                 step(0.5, mod(clamped_uv.x * 10.0, 2.0)));
  col = mix(col, black_key_color, step(1.0, mod(clamped_uv.x * 10.0, 2.0)));

  col *= baseStripe;

  // Glowing sphere
  float glow = sphere(
      fragCoord, vec2(clamped_uv.x, (1.0 - 2.0 * stripe) * iResolution.y), 0.9);

  vec3 pulseColor = mix(white_text_key_color, black_text_key_color, value);
  col += accent_color * glow * 0.5 + pulseColor * glow * 0.3;

  col = mix(col, background_color, 0.5);

  gl_FragColor = vec4(col, 1.0);
}
