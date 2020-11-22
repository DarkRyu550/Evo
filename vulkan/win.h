/* win.h - Windowing and display facilities. */
struct window;

void w_init(struct window **window);
void w_stop(struct window **window);


/* Enumerate the required Vulkan instance extensions.
 *
 * Extension names are stored as sequential null-terminated slices of UTF-8 
 * encoded string data.
 */
void w_exts(
	struct window *window, 
	size_t *count, 
	char **target);

/* Creates the Vulkan surface for this window. */
void w_surface(
	struct window *window, 
	VkInstance instance, 
	VkSurfaceKHR *surface);

