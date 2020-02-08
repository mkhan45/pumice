#version 450

layout(location = 0) in vec4 in_color;
layout(location = 0) out vec4 f_color;
layout(origin_upper_left) in vec4 gl_FragCoord;

void main() {
   /* f_color = vec4(1.0 - in_color.x, 1.0 - in_color.y, 1.0 - in_color.z, 1.0); */
   f_color = in_color;
}
