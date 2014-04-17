RUSTC								= rustc

SRC_DIR							= src
DEPS_DIR            = deps

MAIN								= src/voyager/main.rs

ROOT_DIR						= build
ASSETS_DIR					= $(ROOT_DIR)/assets

GL_VERSION					?= 3.1

DEPS								= -L$(DEPS_DIR)/glfw-rs/lib \
											-L$(DEPS_DIR)/gl-rs/lib

all: voyager

submodule-update:
	@git submodule init
	@git submodule update

deps: submodule-update
	make lib -C $(DEPS_DIR)/gl-rs GL_VERSION=$(GL_VERSION)
	make lib -C $(DEPS_DIR)/glfw-rs

clean:
	make clean -C $(DEPS_DIR)/gl-rs
	make clean -C $(DEPS_DIR)/glfw-rs

assets:
	@mkdir -p $(ASSETS_DIR)

voyager: assets $(MAIN)
	$(RUSTC) $(DEPS) -Llib -O -o $(ROOT_DIR)/voyager $(MAIN)

test: voyager
	@cd $(ROOT_DIR) && ./voyager

.PHONY: \
	all \
	submodule-update \
	deps \
	clean
