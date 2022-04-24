OUTDIR=build
OPT_PROFILE=speed
CARGO_CMD=cargo
CARGO_BUILD_CMD=cargo
# not actually needed right now, but here for reference
RASPBERRY_PI_TOOLCHAIN_PATH=toolchain/raspberry-pi-cross-compilers
RASPBERRY_PI_TOOLCHAIN_TYPE=32# 32 or 64 bit
RASPBERRY_PI_TOOLCHAIN_GCC_VER="10.2.0"
RASPBERRY_PI_TOOLCHAIN_PI_TYPE="3+"
RASPBERRY_PI_TOOLCHAIN_PI_OS_TYPE="bullseye"

RASPBERRY_PI_TOOLCHAIN_BIN_PATH=/toolchain/cross-pi-gcc-10.2.0-2/bin

makeFileDir := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
export PATH := $(makeFileDir)$(RASPBERRY_PI_TOOLCHAIN_BIN_PATH):$(PATH)

# TODO make this actually run the build process (the existing script provided SUCKS)
build-toolchain:
	[ -d $(RASPBERRY_PI_TOOLCHAIN_PATH) ] || git clone https://github.com/abhiTronix/raspberry-pi-cross-compilers.git $(RASPBERRY_PI_TOOLCHAIN_PATH)
	@echo " > this script is incomplete, see the README for more information"

clear-toolchain:
	rm -rf $(RASPBERRY_PI_TOOLCHAIN_PATH)

out_dir:
	[ -d $(OUTDIR) ] || mkdir -p $(OUTDIR)
	[ -d $(OUTDIR)/debug ] || mkdir -p $(OUTDIR)/debug
	[ -d $(OUTDIR)/release ] || mkdir -p $(OUTDIR)/release

clean_out_dir_dbg:
	rm -f $(OUTDIR)/debug/aareocams-dash
	rm -f $(OUTDIR)/debug/aareocams-bot

clean_out_dir_rel:
	rm -f $(OUTDIR)/release/aareocams-dash
	rm -f $(OUTDIR)/release/aareocams-bot

debug: out_dir clean_out_dir_dbg
	$(CARGO_BUILD_CMD) build --bin aareocams-dash
	$(CARGO_BUILD_CMD) build --target armv7-unknown-linux-gnueabihf --bin aareocams-bot
	mv target/debug/aareocams-dash $(OUTDIR)/debug
	mv target/armv7-unknown-linux-gnueabihf/debug/aareocams-bot $(OUTDIR)/debug

release: out_dir clean_out_dir_rel
	$(CARGO_BUILD_CMD) build --profile $(OPT_PROFILE) --bin aareocams-dash
	$(CARGO_BUILD_CMD) build --profile $(OPT_PROFILE) --target armv7-unknown-linux-gnueabihf --bin aareocams-bot
	mv target/$(OPT_PROFILE)/aareocams-dash $(OUTDIR)/release
	mv target/armv7-unknown-linux-gnueabihf/$(OPT_PROFILE)/aareocams-bot $(OUTDIR)/release

deploy_r: release
	export AAREOCAMS_DEPLOY_BUILD_MODE=release && ./tools/deploy.sh

deploy_d: debug
	export AAREOCAMS_DEPLOY_BUILD_MODE=debug && ./tools/deploy.sh

clean:
	$(CARGO_CMD) clean
	rm -rf --preserve-root=all $(OUTDIR)
