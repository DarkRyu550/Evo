#version 450
#pragma shader_stage(compute)
#include <Definitions/Dataset.glslh>

void main() {
    /* Sometimes extra tasks will be spawned, make sure we quit out of them
     * immediately so we don't wrongly write to something. */
    if(gl_GlobalInvocationID.x <  Evo_LowerHerbivore)
        return;
    if(gl_GlobalInvocationID.x >= Evo_UpperHerbivore)
        return;


}
