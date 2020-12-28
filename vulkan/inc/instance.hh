#ifndef __INSTANCE_H__
#define __INSTANCE_H__
#include <vulkan/vulkan.h>

class Instance
{
protected:
	VkInstance       _instance;
	VkPhysicalDevice _physical_device;
	VkDevice         _device;
	
	VkSwapchainKHR _swapchain;
	VkSurfaceKHR   _surface;
	
	VkFence     _fence_cache[32];
	VkSemaphore _semaphore_cache[32];
	
	VkQueue _compute;
	
public:
	 Instance();
	~Instance();
	
	VkFence     get_fence();
	VkSemaphore get_semaphore();
	
	VkCommandPool get_command_pool()
};

#endif
