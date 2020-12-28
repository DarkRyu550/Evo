#include "instance.hh"		/* For the base instance definition. */
#include <stdint.h>			/* For standard integers.	*/ 
#include "def.h"			/* For definitions.			*/
#include "fail.h"			/* For fallible functions.	*/
#include <string.h>			/* For strlen().			*/

/* Creates the Vulkan instance object. */
static void
mk_instance(Instance& inst)
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
	ic.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
	ic.pApplicationInfo = &ai;

	
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

