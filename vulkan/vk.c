#include <vulkan.h>	/* For Vulkan data types.	*/
#include <stdint.h>	/* For standard integers.	*/ 
#include "vk.h"		/* For base definitions.	*/
#include "fail.h"	/* For fallible functions.	*/
#include <string.h>	/* For strlen().			*/

/* Creates the Vulkan instance object. */
static void
mk_instance(struct vulkan *vk, struct window *window)
{
	/* Just use what we have already defined in `def.h` for these fields. */
	VkApplicationInfo ai;
	ai.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
	ai.pApplicationName   = EVO_NAME;
	ai.applicationVersion = EVO_VERSION;
	ai.pEngineName        = EVO_ENGINE_NAME;
	ai.engineVersion      = EVO_ENGINE_VERSION;
	ai.apiVersion         = VK_API_VERSION_1_0;

	VkInstanceCreateInfo ic;
	ai.sType = VK_STRUCTURE_INSTANCE_CREATE_INFO;
	ai.pApplicationInfo = &ai;

	/* Enumerate extensions, first by enumerating ones required by the 
	 * presentation target, then by prepending the ones we know we will
	 * need to use to the list. */
	size_t ext_count = 2;
	char **ext;
	{
		size_t  wext_count;
		char   *wext, *orig;

		w_exts(window, &wext_count, &wext);
		orig = wext;

		ext = allocate(sizeof(char*) * (ext_count + wext_count));
		for(size_t i = 0; i < wext_count; ++i)
		{
			size_t index = ext_count + i;
			size_t len   = strlen(wext) + 1;

			ext[index] = allocate(sizeof(char) * len);
			memcpy(ext[index], wext, len);

			/* Move to the next extension name. */
			wext += len;
		}

		free(orig);
	}
	ext[0] = u8"VK_KHR_surface";
	ext[1] = u8"VK_KHR_swapchain";

	ic.enabledExtensionCount   = ext_count; 
	ic.enabledLayerCount       = 0;
	ic.ppEnabledExtensionNames = ext;

	/* Make the call. */
	if(vkCreateInstance(&ic, NULL, &vk->instance) != VK_SUCCESS)
		panic("could not create vulkan instance: 0x%08x");
}

/* Create the swapchain and the presentation surfaces. */
static void
mk_surfaces(struct vulkan *vk)
{
	
}

void 
vk_init(struct vulkan *vk, struct window *window)
{
	mk_instance(vk, window);
}

