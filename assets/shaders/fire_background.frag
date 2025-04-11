#version 330

// Input from vertex shader
out vec4 finalColor;
in vec4 gl_FragCoord;
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

// Improved noise function for better fire
float hash(float n) { return fract(sin(n) * 1e4); }
float hash(vec2 p) {
  return fract(1e4 * sin(17.0 * p.x + p.y * 0.1) *
               (0.1 + abs(sin(p.y * 13.0 + p.x))));
}

float noise(vec3 x) {
  vec3 p = floor(x);
  vec3 f = fract(x);
  f = f * f * (3.0 - 2.0 * f);

  float n = p.x + p.y * 157.0 + 113.0 * p.z;
  return mix(mix(mix(hash(n + 0.0), hash(n + 1.0), f.x),
                 mix(hash(n + 157.0), hash(n + 158.0), f.x), f.y),
             mix(mix(hash(n + 113.0), hash(n + 114.0), f.x),
                 mix(hash(n + 270.0), hash(n + 271.0), f.x), f.y),
             f.z);
}

float fbm(vec3 p) {
  float f = 0.0;
  f += 0.5 * noise(p);
  p *= 2.01;
  f += 0.25 * noise(p);
  p *= 2.02;
  f += 0.125 * noise(p);
  p *= 2.03;
  f += 0.0625 * noise(p);
  return f / 0.9375;
}

void main() {
  // Normalized pixel coordinates (from 0 to 1)
  vec2 uv = gl_FragCoord.xy / iResolution.xy;

  // Adjust for aspect ratio
  uv.x *= iResolution.x / iResolution.y;

  // Time with speed adjustment
  float time = iTime * speed;

  // Flip coordinates so fire comes from bottom
  uv.y = 1.0 - uv.y;

  // Create fire effect - base position at bottom
  vec3 p = vec3(uv.x * 2.0, uv.y * 3.0 + time * 0.5, time * 0.5);

  // Fire turbulence
  float f = fbm(p * 1.5) * 1.5;
  f += fbm(p * 3.0) * 0.5;

  // Flame shape (wider at bottom)
  float flameShape =
      smoothstep(0.0, 0.3, uv.y) * (1.0 - smoothstep(0.5, 1.0, uv.y)) *
      (1.0 - abs(uv.x - 0.5 * iResolution.x / iResolution.y) * 2.0);

  // Combine noise with shape
  f *= flameShape * 3.0;

  // Color gradient - hottest at base
  vec3 col = background_color;
  col = mix(col, accent_color, clamp(f * 1.5, 0.0, 1.0));
  col = mix(col, text_color, clamp((f - 0.5) * 2.0, 0.0, 1.0));
  col = mix(col, white_key_color, pow(clamp(f - 0.7, 0.0, 1.0), 2.0));

  // Add sparks/embers
  if (noise(vec3(uv * 20.0, time * 0.1)) > 0.99) {
    col = mix(black_text_key_color, white_text_key_color, 0.9);
  }

  // Output to screen
  finalColor = vec4(col, 1.0);
}