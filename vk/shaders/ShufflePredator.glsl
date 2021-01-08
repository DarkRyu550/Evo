#version 450
#pragma shader_stage(compute)
#include <Definitions/Dataset.glslh>
#include <Definitions/SimulationParams.glslh>
#include <Definitions/Matrix.glslh>

/* Shorthand for the individual. SPIR-V does not have refences as far as I know,
 * so doing this, instead, is not that bad. */
#define INDIVIDUAL Evo_Predators[gl_GlobalInvocationID.x]

void main() {
    /* Sometimes extra tasks will be spawned, make sure we quit out of them
     * immediately so we don't wrongly write to something. */
    if(gl_GlobalInvocationID.x <  Evo_LowerPredator)
        return;
    if(gl_GlobalInvocationID.x >= Evo_UpperPredator)
        return;

    if(INDIVIDUAL.energy >= 0.60)
    {
        /* Reproduce together with the best individual. */
        int a = int(Evo_LowerPredator);
        for(int i = int(Evo_LowerPredator); i < Evo_UpperPredator; ++i) {
            if(Evo_Predators[i].energy > Evo_Predators[a].energy
                && (i != gl_GlobalInvocationID.x || a == gl_GlobalInvocationID.x))
                a = i;
        }

        #define MATE Evo_Predators[a]
        #define OFFSPRING Evo_Predators[Evo_UpperPredator - 1]

        MATE.energy -= 0.10;
        INDIVIDUAL.energy -= 0.10;

        Evo_UpperPredator++;

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
        Evo_Individual tmp = Evo_Predators[Evo_LowerPredator];
        Evo_Predators[Evo_LowerPredator] = INDIVIDUAL;
        INDIVIDUAL = tmp;

        ++Evo_LowerPredator;
        return;
    }
}
