#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec3 rot; // [degrees, x, y]
layout(location = 0) out vec4 fragcolor;

layout(set=0, binding=0) uniform Data {
   vec2 scale;
} uniforms;

void main() {
   float x = position.x;
   float y = position.y;

   float rad = radians(rot.x);
   vec2 rot_point = vec2(rot.y, rot.z);

   float sin_ang = sin(rad);
   float cos_ang = cos(rad);

   x = cos_ang * (x - rot_point.x) - sin_ang * (y - rot_point.y) + rot_point.x;
   y = sin_ang * (x - rot_point.x) + cos_ang * (y - rot_point.y) + rot_point.y;

   x *= uniforms.scale.x;
   y *= uniforms.scale.y;

   gl_Position = vec4(x, y, 0.0, 1.0);
   fragcolor = color;
}
