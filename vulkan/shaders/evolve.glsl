#version 450
#pragma shader_stage(compute)
#include "genes.glsl"

layout(location = 0) Individual Current;
layout(location = 1) vec2 Entropy;

/* The fields of the topmost individual are passed here. */
layout(push_constant) sutrct PushConstants {
	float MutationRate;
	Individual Winner;
} Push;

void main()
{
	float mutation = Entropy.r / Entropy.g * Push.MutationRate;
	for(uint i = 0; i < GENE_COUNT; ++i)
	{
		float bias = Entropy.r / Entropy.g;
		float wbias = 1.0 + sin(3.14159 * bias / 2.0) / 2.0;
		float lbias = 1.0 - sin(3.14159 * bias / 2.0) / 2.0;

		Current.Genes[i] = Push.Winner.Genes[i] * wbias + Current.Genes[i] * lbias + mutation;
	}
}
