#include "instance.hh"		/* For the base instance definition.	*/
#include "def.h"			/* For definitions.						*/
#include "fail.c"			/* For fast failing.					*/
#include <vector>

namespace instance
{
	/* Creates the Vulkan instance object. */
	Instance::Instance()
	{
		/* Just use what we have already defined in `def.h` for these fields. */
		vk::ApplicationInfo ai {};
		ai.pApplicationName   = EVO_NAME;
		ai.applicationVersion = EVO_VERSION;
		ai.pEngineName        = EVO_ENGINE_NAME;
		ai.apiVersion         = VK_API_VERSION_1_0;
		ai.engineVersion      = EVO_ENGINE_VERSION;

		vk::InstanceCreateInfo ic {};
		ic.pApplicationInfo = &ai;

		std::vector<const char*> extensions;
		extensions.push_back("VK_KHR_surface");
		extensions.push_back("VK_KHR_swapchain");

		ic.enabledExtensionCount   = extensions.size();
		ic.enabledLayerCount       = 0;
		ic.ppEnabledExtensionNames = extensions.data();

		/* Make the call. */
		if(vk::createInstance(&ic, NULL, &_instance) != vk::Result::eSuccess)
			panic("could not create vulkan instance: 0x%08x");

	}

	Instance::~Instance()
	{

	}

	FenceGuard Instance::get_fence()
	{

	}

	FenceGuard::FenceGuard(Instance& instance, vk::Fence fence)
		: _instance(instance), _fence(fence)
	{ }

	FenceGuard::operator vk::Fence()
	{
		return this->_fence;
	}
}
