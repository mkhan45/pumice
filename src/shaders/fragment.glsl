#version 450

layout(location = 0) in vec4 in_color;
layout(location = 0) out vec4 f_color;
layout(origin_upper_left) in vec4 gl_FragCoord;

void main() {
   //float x = 1024 - gl_FragCoord.x;
   //float y = 1024 - gl_FragCoord.y;
   //float b_color = 0.0;
   //if (x > y && x != 0) {
   //   b_color = y / x;
   //} else if (y > x && y != 0) {
   //   b_color = x / y;
   //}
   //f_color = vec4(x / 1024.0, y / 1024.0, b_color * b_color, 1.0);
   f_color = in_color;
}
