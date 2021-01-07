#version 450
#pragma shader_stage(vertex)
#include <dataset.glslh>

/* Graphics and Input Section. */
layout(set = 1, binding = 0) uniform Display
{
    /* Transformation from model space into world space. */
    mat4 ModelTransformation;
    /* Transformation from world space into projected view space. */
    mat4 ViewProjection;
};
layout(location = 0) in vec4 VectorPosition;

void main() {
    /* Transform the model point into a world point. */
    vec4 pos = ModelTransformation * VectorPosition;

    /* Move this individual by the position it's in. */
    pos.x = Evo_Predators[gl_InstanceIndex].position.x;
    pos.y = Evo_Predators[gl_InstanceIndex].position.y;

    /* Put into view, project and dispatch. */
    gl_Position = ViewProjection * pos;
}
