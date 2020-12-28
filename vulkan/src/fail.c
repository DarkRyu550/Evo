#include <stddef.h>	/* For size_t.					*/
#include "fail.h"	/* For base definitions.		*/
#include <stdio.h>	/* For vfprintf().				*/
#include <stdlib.h>	/* For abort() and malloc().	*/
#include <stdarg.h>	/* For variadic arguments.		*/

void
panic(const char *fmt, ...)
{
	fprintf(stderr, "\n========== PANIC ==========\n");

	va_list va;
	va_start(va, fmt);
	vfprintf(stderr, fmt, va);
	va_end(va);

	fprintf(stderr, "\n===========================\n");

	/* Abort the process and pray for a core dump. */
	abort();
}

void *
allocate(size_t size)
{
	void *data;
	data = malloc(size);

	if(data == NULL)
		panic("An allocation made through allocate() failed.");
	return data;
}

