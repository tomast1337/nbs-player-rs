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

// Constants for behavior
#define SCALE 10.0
#define STEP 4.0
#define GRASSPIECE 0.95
#define UPP 0.125
#define STR 0.2
#define FLOWER 0.995

// --- Hash functions ---
float hash2to1(vec2 p) {
  vec3 p3 = fract(vec3(p.xyx) * 0.1031);
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

float hash3to1(vec3 p3) {
  p3 = fract(p3 * 0.1031);
  p3 += dot(p3, p3.zyx + 31.32);
  return fract((p3.x + p3.y) * p3.z);
}

// --- Flower logic ---
float FlowerCenter(vec2 uv) {
  vec2 pixUV = floor(uv / UPP) * UPP;
  return ceil(hash2to1(pixUV) - FLOWER);
}

float FlowerNeighbor(vec2 uv) {
  vec2 offset = vec2(0, UPP);
  return max(
      FlowerCenter(uv + offset.xy),
      max(FlowerCenter(uv - offset.xy),
          max(FlowerCenter(uv + offset.yx), FlowerCenter(uv - offset.yx))));
}

float FlowerShadow(vec2 uv) {
  vec3 offset = vec3(-UPP, UPP, 0);
  return max(FlowerCenter(uv + offset.zy * 2.0),
             max(FlowerCenter(uv + offset.yy), FlowerCenter(uv + offset.xy)));
}

// --- Main Fragment Shader ---
void main() {
  vec2 uv = gl_FragCoord.xy / iResolution.y;
  uv *= SCALE;
  uv.y += iTime * speed;

  vec2 subuv = fract(uv);
  vec2 coord = floor(uv);
  float noiseForTime = hash2to1(coord) + floor((subuv.x - 0.5) / UPP) * UPP;

  float sinTime = floor(sin(iTime * speed + noiseForTime) * STR / UPP) * UPP;
  float subuvx = floor((subuv.x + sinTime) / UPP) * UPP;
  float noiseForBottom = hash3to1(vec3(coord, subuvx));

  float isBottomPixel =
      1.0 - max(ceil(noiseForBottom - 0.3), step(UPP, subuv.y));
  coord.y -= isBottomPixel;

  vec2 grassPieceUV = vec2(uv.x + sinTime, uv.y * 0.5);
  float noiseForGrassPiece =
      ceil(hash2to1(floor(grassPieceUV / UPP) * UPP) - GRASSPIECE);
  noiseForGrassPiece *= (noiseForGrassPiece * 2.0 - 1.0) * 0.25;

  float noise = hash2to1(coord) + noiseForGrassPiece;
  float gray = ceil(noise * STEP) / STEP;

  // Color blending
  vec3 col = mix(background_color, accent_color, gray); // Ground to grass
  col = mix(col, black_key_color, 0.2 * gray); // Add depth with blade shadows
  col =
      mix(col, white_key_color, 0.1 * (1.0 - gray)); // Highlight tops of blades

  float flowerNeighbor = FlowerNeighbor(uv);
  col = mix(col, text_color, flowerNeighbor * 0.7); // Nearby flower glow

  float flower = FlowerCenter(uv);
  col = mix(col, white_text_key_color, flower); // Actual flower

  float flowerShadow = FlowerShadow(uv);
  col = mix(col, col - 0.15, flowerShadow); // Shadowed side of flower

  // Final output
  gl_FragColor = vec4(col, 1.0);
}