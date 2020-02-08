#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;
layout(location = 0) out vec4 fragcolor;

layout(set=0, binding=0) uniform Data {
   vec2 scale;
} uniforms;

void main() {
   float x = position.x * uniforms.scale.x;
   float y = position.y * uniforms.scale.y;
   gl_Position = vec4(x, y, 0.0, 1.0);
   fragcolor = color;
}
