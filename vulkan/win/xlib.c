/* win/xlib.c - Implementation of windowing facilities using Xlib. */
#include <Xlib.h>			/* For everything Xlib.			*/
#include <vulkan.h>			/* For everything Vulkan.		*/
#include <vulkan_xlib.h>	/* For everything Vulkan.		*/
#include <stddef.h>			/* For size_t.					*/
#include <stdint.h>			/* For standard integer types.	*/
#include "../win.h"			/* For base definitions.		*/
#include "../fail.h"		/* For panic().					*/
#include <stdlib.h>			/* For malloc().				*/
#include <string.h>			/* For memcpy().				*/

struct
window
{
	Display *d;
	GC       g;
	Window   w;
};

void
w_init(struct window **window)
{
	struct window w;

	if((w.d = XOpenDisplay(NULL) == NULL)
		panic("could not connect to the X display");
	w.w = XCreateWindow()
}


void 
w_exts(struct window *window, size_t *count, char **names)
{
	(void) window;

	if(count != NULL) *count = 1;
	if(names != NULL)
	{
		#define EXTENSIONS     (u8"VK_KHR_xlib_surface")
		#define EXTENSIONS_LEN (sizeof(EXTENSIONS) / sizeof(char))

		*names = malloc(EXTENSIONS_LEN);
		memcpy(*names, EXTENSIONS, EXTENSIONS_LEN);
	}
}

void
w_surface(struct window *window, VkInstance instance, VkSurfaceKHR *surface)
{
	VkXlibSurfaceCreateInfoKHR sc;
	sc.sType  = VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR;
	sc.pNext  = NULL;
	sc.flags  = 0;
	sc.dpy    = window->d;
	sc.window = window->w;

	int status;
	if((status = vkCreateXlibSurfaceKHR(instance, &sc, NULL, surface)) != VK_SUCCESS)
		panic("could not create xlib surface: 0x%08x", status);
}

