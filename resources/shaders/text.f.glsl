#version 150 core

uniform vec4 color;
uniform sampler2D text;

in vec2 v_tex_coords;
out vec4 f_color;

void main() {
    vec4 c = vec4(color.rgb, color.a * texture(text, v_tex_coords));
    if (c.a <= 0.01) {
        discard;
    } else {
        f_color = c;
    }
}
