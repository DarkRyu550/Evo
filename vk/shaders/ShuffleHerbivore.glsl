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

    if(INDIVIDUAL.energy >= Params.herbivore_reproduction_min)
    {
        /* Reproduce together with the best individual. */
        int a = int(Evo_LowerHerbivore);
        for(int i = int(Evo_LowerHerbivore); i < Evo_UpperHerbivore; ++i) {
            if(Evo_Herbivores[i].energy > Evo_Herbivores[a].energy
                && (i != gl_GlobalInvocationID.x || a == gl_GlobalInvocationID.x))
                a = i;
        }

        #define MATE Evo_Herbivores[a]
        #define OFFSPRING Evo_Herbivores[Evo_UpperHerbivore - 1]

        MATE.energy       -= Params.herbivore_reproduction_cost;
        INDIVIDUAL.energy -= Params.herbivore_reproduction_cost;

        Evo_UpperHerbivore++;

        OFFSPRING.position = mix(MATE.position, INDIVIDUAL.position, OFFSPRING.biases[0][0]);
        OFFSPRING.velocity = mix(MATE.velocity, INDIVIDUAL.velocity, OFFSPRING.biases[0][1]);
        OFFSPRING.energy   = Params.herbivore_offspring_energy;

        for(int i = 0; i < 2; ++i)
            for(int j = 0; j < 4; ++j)
                OFFSPRING.biases[i][j] = mix(
                    MATE.biases[i][j],
                    INDIVIDUAL.biases[i][j],
                    OFFSPRING.biases[i][j]);

        for(int i = 0; i < 16; ++i)
            for(int j = 0; j < 4; ++j)
                for(int k = 0; k < 4; ++k)
                    OFFSPRING.weights[i][j][k] = mix(
                        MATE.weights[i][j][k],
                        INDIVIDUAL.weights[i][j][k],
                        OFFSPRING.weights[i][j][k]);
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
