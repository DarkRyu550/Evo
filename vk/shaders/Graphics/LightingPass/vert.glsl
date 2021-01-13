#version 450
#pragma shader_stage(vertex)

layout(location = 0) in vec3 Position;
layout(location = 1) in vec3 Normal;

layout(location = 0) out vec2 OutPosition;

void main() {
    OutPosition = Position.xy;
    gl_Position = vec4(Position, 1.0);
}
