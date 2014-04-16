DEPS_DIR            = deps

all:

submodule-update:
	@git submodule init
	@git submodule update

deps: submodule-update
	make lib -C $(DEPS_DIR)/gl-rs
	make lib -C $(DEPS_DIR)/glfw-rs

clean:
	make clean -C $(DEPS_DIR)/gl-rs
	make clean -C $(DEPS_DIR)/glfw-rs

.PHONY: \
	all \
	submodule-update \
	deps \
	clean
