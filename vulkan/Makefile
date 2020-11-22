CC=clang
LD=clang
SC=glslc

CFLAGS=-fsanitize=memory,undefined -Wall -Wextra -Werror -pedantic -std=c18 -g -Og
LFLAGS=-g
SFLAGS=

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

OBJS=fail.o $(WINDOW) vk.o main.o
evo: $(OBJS) Makefile
	$(LD) $(LFLAGS) -o $@ $(OBJS) $(LIBS)

.c.o:
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -rf evo
	find . -type f -name "*.o" -exec rm {} \;
