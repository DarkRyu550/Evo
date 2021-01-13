#version 450
#pragma shader_stage(fragment)

layout(location = 0) in vec3 Normal;

layout(location = 0) out vec4 AlbedoSpecular;
layout(location = 1) out vec4 NormalShadow;

void main() {
    AlbedoSpecular = vec4(1.0, 1.0, 1.0, 0.5);
    NormalShadow   = vec4(Normal, 1.0);
}
