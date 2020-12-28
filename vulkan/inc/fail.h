/* fail.h - Panic and falible operations. */
#ifndef __FAIL_H__
#define __FAIL_H__
#include <stddef.h> /* For size_t. */

/* Displays a given message and aborts the program. */
void panic(const char *fmt, ...);

/* Performs an allocation of memory with the requested size.
 * 
 * Unlike with malloc(), this operation will always either return a valid 
 * allocation or immediately abort the program in case of an allocation failure.
 *
 * This is what you will want in most cases, as malloc() errors will generally
 * lead to general errors in the program anyway, this will at least make it 
 * easier to isolate and debug. */
void * allocate(size_t size);

#endif
