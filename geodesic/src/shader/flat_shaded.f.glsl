#version 150 core

uniform vec4 color;
uniform vec3 light_dir;

in vec3 v_mv_pos;
in vec3 v_eye_relative_pos;

out vec4 o_color;

// From https://github.com/stackgl/glsl-face-normal
vec3 faceNormal(vec3 pos) {
    vec3 fdx = dFdx(pos);
    vec3 fdy = dFdy(pos);
    return normalize(cross(fdx, fdy));
}

void main() {
    vec3 ambient = vec3(0.0075, 0.0075, 0.0075); // avoid pure black unless you're out in space.
    vec3 normal = faceNormal(v_eye_relative_pos);

    float intensity = max(dot(light_dir, normal), 0.0);

    // TODO: Gather light contributions in a loop (support another light or two).
    vec3 lighting = min((intensity + ambient), 1.0);

    o_color = vec4(lighting * color.rgb, 1.0);
}
