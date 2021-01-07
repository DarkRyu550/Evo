#version 450
#pragma shader_stage(vertex)
#include <dataset.glslh>

layout(location = 0) in vec3 Position;
layout(location = 1) in vec3 Normal;

void main() {
    /* Transform the model point into a world point. */
    vec4 pos = ModelTransformation * vec4(Position, 1.0);

    /* Move this individual by the position it's in. */
    pos.x = Evo_Predators[gl_InstanceIndex].position.x;
    pos.y = Evo_Predators[gl_InstanceIndex].position.y;

    /* Put into view, project and dispatch. */
    gl_Position = ViewProjection * pos;
}
