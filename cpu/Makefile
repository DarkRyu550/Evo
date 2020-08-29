BIN_PATH=build/
SRC_PATH=src/
INCLUDE_PATH=include
override CXXFLAGS += \
	-fno-exceptions -fno-rtti -I $(INCLUDE_PATH) -std=c++20

PROJECT_NAME=$(shell basename $(shell pwd))

define override_default
  $(strip $(if $(findstring $(origin $(1)), default),
    $(2),
    $(if $($(1)),
	  $($(1)),
	  $(2)
	)
  ))
endef

override CXX :=     $(call override_default,CXX,clang++-10)
override LD :=      $(call override_default,LD,clang++-10)
override OBJDUMP := $(call override_default,OBJDUMP,llvm-objdump-10)

.PHONY: clean all release debug
.DEFAULT_GOAL := all

release: CXXFLAGS += -Ofast -march=native -mtune=native -flto
release: LDFLAGS += -flto
release: all

debug: CXXFLAGS += -g -DDEBUG
debug: all

clean:
	rm -rf $(BIN_PATH) ./$(PROJECT_NAME)

all: $(PROJECT_NAME)

run: all
	./$(PROJECT_NAME)

SOURCE_FILES=$(notdir $(wildcard ${SRC_PATH}*.cpp))
OBJ_FILES=$(SOURCE_FILES:=.o)
OBJ_PATHS=$(addprefix ${BIN_PATH}, $(OBJ_FILES))

$(BIN_PATH)%.cpp.o: $(SRC_PATH)%.cpp
	$(CXX) $(CXXFLAGS) -c -o $@ $<

setup:
	mkdir -p $(BIN_PATH)

$(PROJECT_NAME): setup $(OBJ_PATHS)
	$(LD) $(LDFLAGS) -o $(PROJECT_NAME) $(OBJ_PATHS)

