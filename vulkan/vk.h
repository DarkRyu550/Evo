
struct
vulkan
{
	VkInstance       instance;
	VkPhysicalDevice physical;
	VkDevice         device;
	
	VKSurfaceKHR     surface;
	VKSwapchainKHR   swapchain;
};

void vk_init(struct vulkan *vk);
void vk_stop(struct vulkan *vk);

