// Uniforms (matching your original shader)
uniform vec2 iResolution; // Unused
uniform float iTime;      // Unused
uniform float speed;      // Unused
uniform vec3 background_color;
uniform vec3 accent_color;
uniform vec3 text_color;           // Unused
uniform vec3 white_key_color;      // Unused
uniform vec3 black_key_color;      // Unused
uniform vec3 white_text_key_color; // Unused
uniform vec3 black_text_key_color; // Unused


void main() {
  gl_FragColor = vec4(background_color, 1.0);
}