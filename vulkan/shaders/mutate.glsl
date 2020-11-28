#version 450
#pragma shader_stage(compute)

/* Include common structures. */
#include "common.glsl"

void main()
{
	uint i = gl_GlobalInvocationID.x;
	uint j = gl_GlobalInvocationID.y;
	if(!Validate(i, j)) return;

	float change = Factor * Entropy[i][j];
	float range  = Maximum[j] - Minimum[j];

	/* Apply the change and clamp the results. */
	Genes[i][j] += change * range;
	Genes[i][j]  = clamp(Genes[i][j], Minimum[j], Maximum[j]);

	/* Snap the result to the granularity of the gene. */
	if(Granularity[j] > 0.0)
	{
		Genes[i][j] /= Granularity[j];
		Genes[i][j] *= round(Genes[i][j]) * Granularity[j];
	}
}

