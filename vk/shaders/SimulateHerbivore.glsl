#version 450
#pragma shader_stage(compute)
#include <Definitions/Dataset.glslh>
#include <Definitions/Matrix.glslh>
#include <Definitions/SimulationParams.glslh>

/* Shorthand for the individual. SPIR-V does not have refences as far as I know,
 * so doing this, instead, is not that bad. */
#define INDIVIDUAL Evo_Herbivores[gl_GlobalInvocationID.x]

vec3 GradientIntensityAt(int x, int y, int component, vec2 view) {
    int top, bottom, left, right;

    float radius = min(view.x, view.y);
    top    = round(clamp(float(y) - abs(radius), 0.0, imageSize(Evo_Field).y - 1));
    bottom = round(clamp(float(y) + abs(radius), 0.0, imageSize(Evo_Field).y - 1));
    left   = round(clamp(float(x) - abs(radius), 0.0, imageSize(Evo_Field).x - 1));
    right  = round(clamp(float(x) + abs(radius), 0.0, imageSize(Evo_Field).x - 1));

    float width  = float(right - left);
    float height = float(bottom - top);

    vec2 center = vec2(x, y);
    float center_val = imageLoad(Evo_Field, ivec2(x, y))[component];

    bool inside = false;
    vec2 gradient = vec2(0.0);

    for(int i = top; i <= bottom; ++i) {
        for(int j = left; j <= right; ++j) {
            vec2 pos = vec2(i - top, j - left);

            float dist = distance(center, pos);
            vec2 direction = normalize(pos - center);


            if(dist > radius && inside)
            {
                /* We're at the right side of the rim. */
                float val = imageLoad(Evo_Field, ivec2(j, i))[component];
                gradient += (val - center_val) * direction;

                inside = false;
            }
            else if(dist <= radius && !inside)
            {
                /* We're at the left side of the rim. */
                float val = imageLoad(Evo_Field, ivec2(j, i))[component];
                gradient += (val - center_val) * direction;

                inside = true;
            }
        }
        inside = false;
    }

    return vec3(normalize(gradient).xy, length(gradient));
}

void main()
{
    /* Sometimes extra tasks will be spawned, make sure we quit out of them
     * immediately so we don't wrongly write to something. */
    if(gl_GlobalInvocationID.x <  Evo_LowerHerbivore)
        return;
    if(gl_GlobalInvocationID.x >= Evo_UpperHerbivore)
        return;

    /* Coerce the individual back into bounds if necessary. */
    INDIVIDUAL.position.x = clamp(INDIVIDUAL.position.x, 0.0, Params.field_size.x - 0.01);
    INDIVIDUAL.position.y = clamp(INDIVIDUAL.position.y, 0.0, Params.field_size.y - 0.01);

    /* Figure out where we are in the simulation field. */
    vec2 view = vec2(
        Params.herbivore_view_radius / Params.field_size.x * imageSize(Evo_Field).x,
        Params.herbivore_view_radius / Params.field_size.y * imageSize(Evo_Field).y
    );
    int field_x = floor(INDIVIDUAL.position.x / Params.field_size.x * imageSize(Evo_Field).x);
    int field_y = floor(INDIVIDUAL.position.y / Params.field_size.y * imageSize(Evo_Field).y);

    /* Create the input for the network. */
    vec4[4] nn_input;
    nn_input[0] = vec4(0.0);
    nn_input[1] = vec4(0.0);
    nn_input[2] = vec4(0.0);
    nn_input[3] = vec4(0.0);

    nn_input[0][0] = INDIVIDUAL.velocity.x;
    nn_input[0][1] = INDIVIDUAL.velocity.y;

    vec3 red   = GradientIntensityAt(field_x, field_y, 0, Params.herbivore_view_radius);
    vec3 green = GradientIntensityAt(field_x, field_y, 1, Params.herbivore_view_radius);
    vec3 blue  = GradientIntensityAt(field_x, field_y, 2, Params.herbivore_view_radius);

    nn_input[0][2] = red.x;
    nn_input[0][3] = red.y;
    nn_input[1][0] = green.x;
    nn_input[1][1] = green.y;
    nn_input[1][2] = green.z;
    nn_input[1][3] = blue.x;
    nn_input[2][0] = blue.y;
    nn_input[2][1] = blue.z;

    /* Calculate an output value. */
    vec4[4] nn_output = MatrixMultiplyByVec16(INDIVIDUAL.weights);
    for(int i = 0; i < 16; ++i) {
        nn_output[i / 4][i % 4] = Sigmoid(nn_output[i / 4][i % 4]);
    }

    /* Perform the actions we got from the output. */
    vec2 movement = vec2(
        nn_output[0][1] * cos(nn_output[0][0] * 2 * 3.1415),
        nn_output[0][1] * sin(nn_output[0][0] * 2 * 3.1415));

    float penalty = nn_output[]
}
