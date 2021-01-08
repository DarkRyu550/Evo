#version 450
#pragma shader_stage(compute)
#include <Definitions/Dataset.glslh>
#include <Definitions/SimulationParams.glslh>
#include <Definitions/Matrix.glslh>

/* Shorthand for the individual. SPIR-V does not have refences as far as I know,
 * so doing this, instead, is not that bad. */
#define INDIVIDUAL Evo_Herbivores[gl_GlobalInvocationID.x]

void main() {
    /* Sometimes extra tasks will be spawned, make sure we quit out of them
     * immediately so we don't wrongly write to something. */
    if(gl_GlobalInvocationID.x <  Evo_LowerHerbivore)
        return;
    if(gl_GlobalInvocationID.x >= Evo_UpperHerbivore)
        return;

    if(INDIVIDUAL.energy >= 0.60)
    {
        /* Reproduce together with the best individual. */
        int a = int(Evo_LowerHerbivore);
        for(int i = int(Evo_LowerHerbivore); i < Evo_UpperHerbivore; ++i) {
            if(Evo_Herbivores[i].energy > Evo_Herbivores[a].energy)
                a = i;
        }

        #define MATE Evo_Herbivores[a]
        #define OFFSPRING Evo_Herbivores[Evo_UpperHerbivore - 1]

        MATE.energy -= 0.10;
        INDIVIDUAL.energy -= 0.10;

        Evo_UpperHerbivore++;

        OFFSPRING.position = mix(MATE.position, INDIVIDUAL.position, 0.5);
        OFFSPRING.velocity = mix(MATE.velocity, INDIVIDUAL.velocity, 0.5);
        OFFSPRING.energy   = 0.5;

        OFFSPRING.biases[0] = mix(MATE.biases[0], INDIVIDUAL.biases[0], 0.5);
        OFFSPRING.biases[1] = mix(MATE.biases[1], INDIVIDUAL.biases[1], 0.5);

        for(int i = 0; i < 16; ++i)
            for(int j = 0; j < 4; ++j)
                OFFSPRING.weights[i][j] = mix(MATE.weights[i][j], INDIVIDUAL.weights[i][j], 0.5);
    }
    else if(INDIVIDUAL.energy < 0.0)
    {
        /* Die. */
        Evo_Individual tmp = Evo_Herbivores[Evo_LowerHerbivore];
        Evo_Herbivores[Evo_LowerHerbivore] = INDIVIDUAL;
        INDIVIDUAL = tmp;

        ++Evo_LowerHerbivore;
        return;
    }
}
