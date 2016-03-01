#version 150 core

uniform mat4 model;
uniform mat4 view_proj;

in vec3 normal;
in vec3 position;

out vec3 v_normal;

void main() {
    v_normal = normal;
    gl_Position = view_proj * model * vec4(position, 1.0);
}
