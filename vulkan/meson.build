sources = ['src/fail.c', 'src/instance.cc', 'src/main.cc']

# Find the shader compiler and create the shader build dependency.
shaders = ['shaders/cross.glsl', 'shaders/mutate.glsl']

glslc_program = find_program('glslc')
glslc = generator(
	glslc_program,
	output:     '@BASENAME@.spv',
	arguments: ['-O', '-o', '@OUTPUT@', '@INPUT@']
)

spirv = glslc.process(shaders)

python3_program = import('python').find_installation('python3')
assets_target = custom_target(
	'assets.h',
	output:  'assets.h',
	input:   ['tools/assets', spirv],
	command: [python3_program, '@INPUT@', '-o', '@OUTPUT@'],
	install: true,
	install_dir: 'inc/'
)
assets = declare_dependency(
	sources: [assets_target]
)

# Find the dependencies for the actual C program.
vk   = dependency('vulkan')
glfw = dependency('glfw3')
thrd = dependency('threads')
inc  = include_directories('inc/')

executable(
	'vulkan',
	sources,
	deps: [
		vk,
		glfw,
		thrd,
		assets
	],
	include_directories: [
		inc
	],
	win_subsystem: 'windows'
)
