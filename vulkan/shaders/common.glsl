
/* Maximum number of genes  */
const uint MAX_GENES = 32;

layout(std430, set = 0, binding = 0)
buffer Individuals
{
	/* Buffer index to the individual to be preserved. */
	int Preserve;

	/* Mutation factor, from zero to one. */
	float Factor;

	/* Granularity values for every gene. 
	 * 
	 * A gene's granularity is defined as the smallest meaningful step that can
	 * be taken while changing its value. A granularity of 0 means that the 
	 * value of the gene maps continuously to the output. */
	float Granularity[MAX_GENES];

	/* The minimum meaningful values of every gene. */
	float Minimum[MAX_GENES];

	/* The maximum meaningful values of every gene. */
	float Maximum[MAX_GENES];

	/* The gene data for every individual.  */
	float Genes[][MAX_GENES];
};

layout(std430, set = 0, binding = 1)
buffer EntropyBuffer
{
	/* The entropy buffer for every individual, in a range from minus one to 
	 * one, inclusive. Each gene gets its own entropy value. */
	float Entropy[][MAX_GENES];
};


bool Validate(uint i, uint j)
{
	if(i >= Genes.length() || j >= MAX_GENES)
		/* Kill off extra invocations to the shader that don't map to any. 
		 * entity data directly. */
		return false;
	
	if(i == Preserve)
		/* We were invoked to do perform on the preserved entity. Kill this
		 * invocation off so that we guarantee the preserved entity is kept. */
		return false;
	return true;
}
