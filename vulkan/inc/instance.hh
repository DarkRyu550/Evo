#ifndef __INSTANCE_H__
#define __INSTANCE_H__
#include <vulkan/vulkan.hpp>	/* For Vulkan.		*/
#include <atomic>				/* For std::atomic.	*/
#include <cstddef>				/* For size_t.		*/

namespace instance
{
	class FenceGuard;
	class SemaphoreGuard;
	
	class Instance
	{
	protected:
		vk::Instance       _instance;
		vk::PhysicalDevice _physical_device;
		vk::Device         _device;

		vk::SwapchainKHR _swapchain;
		vk::SurfaceKHR   _surface;

		vk::Fence     _fence_cache[32];
		vk::Semaphore _semaphore_cache[32];

		std::atomic<size_t> _fence_top;
		std::atomic<size_t> _semaphore_top;

		vk::Queue _compute;
		vk::Queue _graphics;

	public:
		 Instance();
		~Instance();

		Instance(Instance&)  = delete;
		Instance(Instance&&) = delete;

		Instance(const Instance&)  = delete;
		Instance(const Instance&&) = delete;

		FenceGuard     get_fence();
		SemaphoreGuard get_semaphore();

			  vk::Device& get_device()       noexcept;
		const vk::Device& get_device() const noexcept;
	};
	
	class FenceGuard
	{
		friend Instance;
	protected:
		Instance& _instance;
		vk::Fence _fence;
	public:
		FenceGuard(Instance& instance, vk::Fence fence);

		FenceGuard(const FenceGuard&)  = delete;
		FenceGuard(const FenceGuard&&) = delete;
		FenceGuard(FenceGuard&)  = delete;
		FenceGuard(FenceGuard&&) = delete;

		~FenceGuard();

		operator vk::Fence();
	};
	
	class SemaphoreGuard
	{
		friend Instance;
	protected:
		Instance& _instance;
		vk::Semaphore _semaphore;
	public:
		SemaphoreGuard(Instance& instance, vk::Fence fence);

		SemaphoreGuard(const SemaphoreGuard&)  = delete;
		SemaphoreGuard(const SemaphoreGuard&&) = delete;
		SemaphoreGuard(SemaphoreGuard&)  = delete;
		SemaphoreGuard(SemaphoreGuard&&) = delete;

		~SemaphoreGuard();

		operator vk::Semaphore();
	};
}

#endif
