#version 150 core

uniform vec4 color;
uniform vec3 light_dir;

in vec3 v_mv_pos;

out vec4 o_color;

// From https://github.com/stackgl/glsl-face-normal
vec3 faceNormal(vec3 pos) {
  vec3 fdx = dFdx(pos);
  vec3 fdy = dFdy(pos);
  return normalize(cross(fdx, fdy));
}

void main() {
    vec3 normal = faceNormal(v_mv_pos);

    float intensity = max(dot(light_dir, normal), 0.0);
    o_color = vec4(intensity * color.rgb, 1.0);
}
