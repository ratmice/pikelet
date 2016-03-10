#version 150 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;
uniform vec3 eye;

in vec3 position;

out vec3 v_mv_pos;
out vec3 v_eye_relative_pos;

void main() {
    vec4 pos = vec4(position, 1.0);
    vec4 mv_pos = view * model * pos;

    gl_Position = proj * mv_pos;
    v_mv_pos = -mv_pos.xyz;
    v_eye_relative_pos = (model * pos).xyz - eye;
}
