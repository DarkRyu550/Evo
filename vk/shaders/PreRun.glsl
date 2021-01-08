#version 450
#pragma shader_stage(compute)
#include <Definitions/Dataset.glslh>
#include <Definitions/SimulationParams.glslh>

void main() {
    uvec2 position  = gl_GlobalInvocationID.xy;
    ivec2 dimension = imageSize(Evo_Field);

    /* Quit out of extra jobs. */
    if(position.x >= dimension.x) return;
    if(position.y >= dimension.y) return;

    imageStore(Evo_Field,     ivec2(position), vec4(0.0));
    imageStore(Evo_FieldLock, ivec2(position), uvec4(0));
}
