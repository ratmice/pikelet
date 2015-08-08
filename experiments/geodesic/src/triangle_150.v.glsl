#version 150 core

in vec3 a_Pos;
out vec4 v_Color;

uniform mat4 u_View;
uniform mat4 u_Proj;

void main() {
    v_Color = vec4(1.0, 1.0, 1.0, 1.0);
    gl_Position = u_Proj * u_View * vec4(a_Pos, 1.0);
}
