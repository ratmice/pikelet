#version 150 core

uniform vec4 color;
uniform mat4 model;
uniform mat4 camera;

in vec3 position;

out vec4 vColor;

void main() {
    vColor = color;
    gl_Position = camera * model * vec4(position, 1.0);
}
