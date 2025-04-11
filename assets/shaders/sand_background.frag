#extension GL_OES_standard_derivatives : enable
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

// Hash function
float hash(float n) { return fract(sin(n) * 43758.5453); }

// Smooth 2D noise
float noise(vec2 x) {
  vec2 p = floor(x);
  vec2 f = fract(x);
  f = f * f * (3.0 - 2.0 * f);
  float n = p.x + p.y * 57.0;
  return mix(mix(hash(n + 0.0), hash(n + 1.0), f.x),
             mix(hash(n + 57.0), hash(n + 58.0), f.x), f.y);
}

// Directional mapping using animated noise
vec2 map(vec2 p, float offset) {
  p.x += 0.1 * sin(iTime * speed + 2.0 * p.y);
  p.y += 0.1 * sin(iTime * speed + 2.0 * p.x);
  float a = noise(p * 1.5 + sin(0.1 * iTime * speed)) * 6.2831;
  a -= offset;
  return vec2(cos(a), sin(a));
}

void main() {
  vec2 fragCoord = gl_FragCoord.xy;
  vec2 p = fragCoord / iResolution;
  vec2 uv = -1.0 + 2.0 * p;
  uv.x *= iResolution.x / iResolution.y;

  float offset = iTime + fragCoord.x / iResolution.x;

  float acc = 0.0;
  vec3 col = vec3(0.0);

  for (int i = 0; i < 32; i++) {
    vec2 dir = map(uv, offset);

    float h = float(i) / 32.0;
    float w = 4.0 * h * (1.0 - h);

    // Color blend based on wave depth
    vec3 blend1 = mix(background_color, accent_color, h);
    vec3 blend2 =
        mix(text_color, white_key_color, sin(iTime + float(i)) * 0.5 + 0.5);
    vec3 blend3 = mix(black_key_color, white_text_key_color,
                      cos(iTime + float(i) * 0.5) * 0.5 + 0.5);
    vec3 blend4 = mix(black_text_key_color, blend1, h);

    vec3 ttt =
        w * mix(mix(blend1, blend2, h), mix(blend3, blend4, 1.0 - h), 0.5);
    col += w * ttt;
    acc += w;

    uv += 0.008 * dir;
  }

  col /= acc;

  float gg = dot(col, vec3(0.333));
  vec3 nor = normalize(vec3(dFdx(gg), 0.5, dFdy(gg)));
  col += vec3(0.4) * dot(nor, vec3(0.7, 0.01, 0.7));

  vec2 di = map(uv, offset);
  col *= 0.65 + 0.35 * dot(di, vec2(0.707));
  col *= 0.20 + 0.80 * pow(4.0 * p.x * (1.0 - p.x), 0.1);
  col *= 1.7;

  gl_FragColor = vec4(col, 1.0);
}
