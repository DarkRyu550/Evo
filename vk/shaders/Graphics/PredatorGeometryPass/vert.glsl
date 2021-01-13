#version 450
#pragma shader_stage(vertex)
#include <Definitions/Dataset.glslh>
#include <Definitions/RenderParams.glslh>

layout(location = 0) in vec3 Position;
layout(location = 1) in vec3 Normal;

layout(location = 0) out vec3 OutNormal;

void main() {
    /* Transform the model point into a world point. */
    vec4 pos = Params.ModelTransformation * vec4(Position, 1.0);

    /* Move this individual by the position it's in. */
    pos.x += Evo_Predators[gl_InstanceIndex].position.x;
    pos.y += Evo_Predators[gl_InstanceIndex].position.y;

    /* Write out the normals. */
    OutNormal = (Params.ModelNormalTransform * vec4(Normal, 1.0)).xyz;

    /* Put into view, project and dispatch. */
    gl_Position = Params.ViewProjection * pos;
}
