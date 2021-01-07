#version 450
#pragma shader_stage(fragment)

layout(location = 0) out vec4 Color;
void main() {
    Color = vec4(1.0, 1.0, 1.0, 0.5);
}
