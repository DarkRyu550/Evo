#version 450
#pragma shader_stage(compute)
#include <Definitions/Dataset.glslh>
#include <Definitions/Matrix.glslh>
#include <Definitions/SimulationParams.glslh>
#include <Definitions/ImageLock.glslh>

/* Shorthand for the individual. SPIR-V does not have refences as far as I know,
 * so doing this, instead, is not that bad. */
#define INDIVIDUAL Evo_Herbivores[gl_GlobalInvocationID.x]

vec3 GradientIntensityAt(int x, int y, int component, vec2 view) {
    int top, bottom, left, right;

    float radius = min(view.x, view.y);
    top    = int(round(clamp(float(y) - abs(radius), 0.0, imageSize(Evo_Field).y - 1)));
    bottom = int(round(clamp(float(y) + abs(radius), 0.0, imageSize(Evo_Field).y - 1)));
    left   = int(round(clamp(float(x) - abs(radius), 0.0, imageSize(Evo_Field).x - 1)));
    right  = int(round(clamp(float(x) + abs(radius), 0.0, imageSize(Evo_Field).x - 1)));

    float width  = float(right - left);
    float height = float(bottom - top);

    vec2 center = vec2(x, y);
    float center_val = clamp(FieldLoad(x, y)[component], 0.0, 1.0);

    vec2 gradient = vec2(0.000001);
    for(int i = top; i <= bottom; ++i) {
        for(int j = left; j <= right; ++j) {
            vec2 pos = vec2(i - top, j - left);

            float dist = distance(center, pos);
            vec2 direction = normalize(pos - center);

            if(dist > radius)
                continue;
            float val = clamp(FieldLoad(i, j)[component], 0.0, 1.0);

            gradient += (val - center_val) * direction;
        }
    }

    return vec3(normalize(gradient), length(gradient));
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
    int field_x = int(floor(INDIVIDUAL.position.x / Params.field_size.x * imageSize(Evo_Field).x));
    int field_y = int(floor(INDIVIDUAL.position.y / Params.field_size.y * imageSize(Evo_Field).y));

    /* Start off by feeding, if possible. */
    FieldLock(field_x, field_y);

    vec4  feed  = imageLoad(Evo_Field, ivec2(field_x, field_y));
    float eat = min(feed.w, 1.0 - INDIVIDUAL.energy);

    INDIVIDUAL.energy += eat;
    feed.w -= eat;

    imageStore(Evo_Field, ivec2(field_x, field_y), feed);

    FieldUnlock(field_x, field_y);


    /* Create the input for the network. */
    vec4[4] nn_input;
    nn_input[0] = vec4(0.0);
    nn_input[1] = vec4(0.0);
    nn_input[2] = vec4(0.0);
    nn_input[3] = vec4(0.0);

    nn_input[0][0] = INDIVIDUAL.velocity.x;
    nn_input[0][1] = INDIVIDUAL.velocity.y;

    vec2 view = vec2(
        Params.herbivore_view_radius / Params.field_size.x * imageSize(Evo_Field).x,
        Params.herbivore_view_radius / Params.field_size.y * imageSize(Evo_Field).y
    );
    vec3 red   = GradientIntensityAt(field_x, field_y, 0, view);
    vec3 green = GradientIntensityAt(field_x, field_y, 1, view);
    vec3 blue  = GradientIntensityAt(field_x, field_y, 2, view);
    vec3 alpha = GradientIntensityAt(field_x, field_y, 3, view);

    nn_input[0][2] = red.x;
    nn_input[0][3] = red.y;
    nn_input[1][0] = red.z;
    nn_input[1][1] = green.x;
    nn_input[1][2] = green.y;
    nn_input[1][3] = green.z;
    nn_input[2][0] = blue.x;
    nn_input[2][1] = blue.y;
    nn_input[2][2] = blue.z;
    nn_input[2][3] = alpha.x;
    nn_input[3][0] = alpha.y;
    nn_input[3][1] = alpha.z;

    /* Calculate an output value. */
    vec4[4] nn_output = MatrixMultiplyByVec16(INDIVIDUAL.weights, nn_input);
    for(int i = 0; i < 8; ++i) {
        nn_output[i / 4][i % 4] += INDIVIDUAL.biases[i / 4][i % 4];
    }
    for(int i = 0; i < 16; ++i) {
        nn_output[i / 4][i % 4] = Sigmoid(nn_output[i / 4][i % 4]);
    }

    /* Perform the actions we got from the output. */
    vec2 movement = vec2(
        cos(nn_output[0][0] * 2 * 3.1415),
        sin(nn_output[0][0] * 2 * 3.1415));

    float speed = mix(0.0, Params.herbivore_max_speed, nn_output[0][1]);
    movement *= Params.delta * speed;

    float penalty = mix(
        Params.herbivore_penalty.x,
        Params.herbivore_penalty.y,
        nn_output[0][1]);
    penalty *= Params.delta;

    INDIVIDUAL.position += movement;
    INDIVIDUAL.velocity  = movement;
    INDIVIDUAL.energy   -= penalty;

    /* Coerce the individual back into bounds if necessary. */
    INDIVIDUAL.position.x = clamp(INDIVIDUAL.position.x, 0.0, Params.field_size.x - 0.01);
    INDIVIDUAL.position.y = clamp(INDIVIDUAL.position.y, 0.0, Params.field_size.y - 0.01);

    /* Update the tile. */
    FieldLock(field_x, field_y);
    vec4 tile = imageLoad(Evo_Field, ivec2(field_x, field_y));

    tile.x = clamp(tile.x + nn_output[0][2], 0.0, 1.0);
    tile.y = clamp(tile.y + nn_output[0][3], 0.0, 1.0);
    tile.z = clamp(tile.z + nn_output[1][0], 0.0, 1.0);

    FieldUnlock(field_x, field_y);

}
