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

    /* Weave all of the herbivore data. */
    for(int z = int(Evo_LowerHerbivore); z < Evo_UpperHerbivore; ++z) {
        vec4 delta = imageLoad(
            Evo_HerbivoreFields,
            ivec3(ivec2(position), z));
        vec4 value = imageLoad(
            Evo_Field,
            ivec2(position));

        value.x = clamp(value.x + delta.x, 0.0, 1.0);
        value.y = clamp(value.y + delta.y, 0.0, 1.0);
        value.z = clamp(value.z + delta.z, 0.0, 1.0);
        value.w = clamp(value.w + delta.w, 0.0, 1.0);

        imageStore(
            Evo_Field,
            ivec2(position),
            value);
    }

    /* Weave all of the predator data. */
    for(int z = int(Evo_LowerPredator); z < Evo_UpperPredator; ++z) {
        vec4 delta = imageLoad(
            Evo_PredatorFields,
            ivec3(ivec2(position), z));
        vec4 value = imageLoad(
            Evo_Field,
            ivec2(position));

        value.x = clamp(value.x + delta.x, 0.0, 1.0);
        value.y = clamp(value.y + delta.y, 0.0, 1.0);
        value.z = clamp(value.z + delta.z, 0.0, 1.0);
        value.w = clamp(value.w + delta.w, 0.0, 1.0);

        imageStore(
            Evo_Field,
            ivec2(position),
            value);
    }

    /* Perform growth and decay. */
    vec4 value = imageLoad(
        Evo_Field,
        ivec2(position));

    value.x = clamp(value.x - Params.decomposition_rate * Params.delta, 0.0, 1.0);
    value.y = clamp(value.y - Params.decomposition_rate * Params.delta, 0.0, 1.0);
    value.z = clamp(value.z - Params.decomposition_rate * Params.delta, 0.0, 1.0);
    value.w = clamp(value.w + Params.growth_rate        * Params.delta, 0.0, 1.0);

    imageStore(
        Evo_Field,
        ivec2(position),
        value);
}