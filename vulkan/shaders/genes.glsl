#version 450
/* The number of genes in an individual's chromosome. */
const uint GENE_COUNT = 4;

struct Individual
{
	float Genes[GENE_COUNT];
};