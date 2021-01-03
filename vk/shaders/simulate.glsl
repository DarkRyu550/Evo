#version 450
#pragma shader_stage(compute)
/* # Simulation model
 *
 */

layout(std430, set = 0, binding = 0) buffer Population
{
    struct Individual
    {
        vec2 direction_weights[];
        vec2 direction_biases[20]

        vec2 position;
    };
};

void main()
{

}