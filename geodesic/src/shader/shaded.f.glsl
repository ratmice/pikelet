#version 150 core

uniform vec4 color;
uniform vec3 light_dir;

in vec3 v_normal;

out vec4 o_color;

void main() {
    float intensity = max(dot(light_dir, v_normal), 0.0);
    o_color = vec4(intensity * color.rgb, 1.0);
}
