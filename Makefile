RUSTC								= rustc

SRC_DIR							= src
DEPS_DIR            = deps

MAIN								= src/voyager/main.rs

BUILD_DIR						= $(PWD)/build
ASSETS_DIR					= $(BUILD_DIR)/assets

GL_VERSION					?= 3.3

DEPS								= -L$(DEPS_DIR)/glfw-rs/lib \
											-L$(DEPS_DIR)/gl-rs/lib \
											-L$(DEPS_DIR)/cgmath-rs/lib

all: voyager

submodule-update:
	@git submodule init
	@git submodule update

deps:
	make lib -C $(DEPS_DIR)/gl-rs GL_VERSION=$(GL_VERSION)
	make lib -C $(DEPS_DIR)/cgmath-rs
	make lib -C $(DEPS_DIR)/glfw-rs

clean:
	make clean -C $(DEPS_DIR)/gl-rs
	make clean -C $(DEPS_DIR)/cgmath-rs
	make clean -C $(DEPS_DIR)/glfw-rs
	@rm -rf $(BUILD_DIR)

assets:
	@mkdir -p $(ASSETS_DIR)
	make -C src/assets ASSETS_DIR=$(ASSETS_DIR)

voyager: assets
	$(RUSTC) $(DEPS) -Llib -O -o $(BUILD_DIR)/voyager $(MAIN)

test: voyager
	@cd $(BUILD_DIR) && ./voyager

.PHONY: \
	all \
	submodule-update \
	deps \
	clean
