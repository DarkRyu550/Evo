CC=clang
LD=clang
SC=glslc

CFLAGS=-fsanitize=memory,undefined -Wall -Wextra -Werror -pedantic -std=c18 -g -Og
LFLAGS=-g
SFLAGS=-g -O

#
# Setup specific settings for Linux and X11.
#
WINDOW=win/xlib.o
CFLAGS+=-I/usr/include/vulkan -I/usr/include/X11
LIBS+=-lvulkan -lx11

#
# Setup specific settings for Windows.
#
# WINDOW=win/win32.o
# CFLAGS+=
# LFLAGS+=
# LIBS+=-lvulkan

SHAD=shaders/mutate.spirv
OBJS=fail.o $(WINDOW) vk.o main.o
	
evo: $(OBJS) shaders.h Makefile
	$(LD) $(LFLAGS) -o $@ $(OBJS) $(LIBS)
clean:
	rm -rf evo shaders.h
	find . -type f -name "*.o"     -exec rm {} \;
	find . -type f -name "*.spirv" -exec rm {} \;
shaders.h: $(SHAD) Makefile
	echo "/* Static shader data from the compiled shader files. */" > $@
	echo >> $@	
	for shader in "$(SHAD)"; \
	do \
		echo -n "_Alignas(4) "; \
		xxd -i "$$shader"; \
	done >> $@
	sed -i 's/_spirv//' $@
	sed -i 's/unsigned/const unsigned/' $@
	sed -i 's/shaders_/SHADERS_/' $@
	awk -i inplace -F '_Alignas\\(4\\) const unsigned char SHADERS_|\\[' '{ sub($$2, toupper($$2), $$0); print $$0 }' $@
	awk -i inplace -F 'const unsigned int SHADERS_| =' '{ sub($$2, toupper($$2), $$0); print $$0 }' $@


.SUFFIXES: .c .o .glsl .spirv
.c.o:
	$(CC) $(CFLAGS) -c -o $@ $<
.glsl.spirv:
	$(SC) $(SFLAGS) -o $@ $<

