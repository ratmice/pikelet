#version 150 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;

in vec3 position;

out vec3 v_mv_pos;

void main() {
    vec4 pos = vec4(position, 1.0);
    vec4 mv_pos = view * model * pos;

    gl_Position = proj * mv_pos;
    v_mv_pos = -mv_pos.xyz;
}
