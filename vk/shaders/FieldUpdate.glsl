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

    vec4 cell = imageLoad(Evo_Field, ivec2(position));
    cell.x = clamp(cell.x - Params.decomposition_rate * Params.delta, 0.0, 1.0);
    cell.y = clamp(cell.y - Params.decomposition_rate * Params.delta, 0.0, 1.0);
    cell.z = clamp(cell.z - Params.decomposition_rate * Params.delta, 0.0, 1.0);
    cell.w = clamp(cell.w + Params.growth_rate * Params.delta, 0.0, 1.0);
    imageStore(Evo_Field, ivec2(position), cell);
}
