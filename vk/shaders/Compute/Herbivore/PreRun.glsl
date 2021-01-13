#version 450
#pragma shader_stage(compute)
#include <Definitions/Dataset.glslh>
#include <Definitions/SimulationParams.glslh>

void main() {
    uvec3 position  = gl_GlobalInvocationID.xyz;
    ivec3 dimension = imageSize(Evo_HerbivoreFields);

    /* Quit out of extra jobs. */
    if(position.x >= dimension.x) return;
    if(position.y >= dimension.y) return;
    if(position.z >= dimension.z) return;

    imageStore(Evo_HerbivoreFields, ivec3(position), vec4(0.0));
}
