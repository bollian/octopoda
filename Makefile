CODENAME:=octopoda

# default to the RPi3
BSP ?=rpi3

# default device name to use when opening serial terminal/minipush
DEV_SERIAL ?= /dev/ttyUSB0

# bsp-specific arguments
ifeq ($(BSP),rpi3)
	TARGET:=aarch64-unknown-none-softfloat
	OUTPUT:=kernel8.img
	LINKER_FILE:=src/bsp/link.ld
	RUSTC_MISC_ARGS:=-C target-cpu=cortex-a53

	QEMU_BINARY:=qemu-system-aarch64
	QEMU_MACHINE_TYPE:=raspi3
	QEMU_RELEASE_ARGS:=-serial stdio -display none
else ifeq ($(BSP),rpi4)
	TARGET:=aarch64-unknown-none-softfloat
	OUTPUT:=kernel8.img
	LINKER_FILE:=src/bsp/link.ld
	RUSTC_MISC_ARGS:=-C target-cpu=cortex-a72

	QEMU_BINARY:=qemu-system-aarch64
	QEMU_MACHINE_TYPE:=raspi4
	QEMU_RELEASE_ARGS:=-serial stdio -display none
endif

# export for build.rs
export LINKER_FILE

ifdef PRINT_ASM
	QEMU_BINARY_EXTRA_FLAGS:=-d in_asm
else
	QEMU_BINARY_EXTRA_FLAGS:=
endif

RUSTFLAGS     := -C link-arg=-T$(LINKER_FILE) $(RUSTC_MISC_ARGS)
COMPILER_ARGS := --features bsp_$(BSP) --target=$(TARGET) --release
BIN_DIR       := bin
KERNEL_BIN    := $(BIN_DIR)/$(OUTPUT)
KERNEL_ELF    := target/$(TARGET)/release/$(CODENAME)

DOCKER_UTILS:=rustembedded/osdev-utils
DOCKER_RUN:=docker run -it --rm
DOCKER_WORK_DIR:=-v $(shell pwd):/work -w /work
SOURCES:=$(wildcard src/*.rs) $(wildcard src/**/*.rs) $(wildcard src/**/*.ld) \
	Cargo.toml Cargo.lock Makefile rust-toolchain.toml

.PHONY: all clean check qemu-test clippy objdump nm readelf chainboot doc $(KERNEL_ELF)

all: $(KERNEL_BIN)

clean:
	cargo clean
	rm -rf $(BIN_DIR)

# the -p is so that mkdir doesn't fail if the directory exists
$(KERNEL_BIN): $(KERNEL_ELF)
	mkdir -p $(BIN_DIR)
	rust-objcopy --strip-all -O binary $(KERNEL_ELF) $(KERNEL_BIN)

$(KERNEL_ELF): $(SOURCES)
	RUSTFLAGS="$(RUSTFLAGS)" cargo rustc $(COMPILER_ARGS)

ifeq ($(QEMU_MACHINE_TYPE),)
	@echo "This machine isn't currently supported by qemu"
else
qemu-test: $(KERNEL_BIN)
	$(DOCKER_RUN) $(DOCKER_WORK_DIR) $(DOCKER_UTILS) \
		$(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE) -kernel $(KERNEL_BIN) \
			$(QEMU_RELEASE_ARGS) $(QEMU_BINARY_EXTRA_FLAGS)
endif

check:
	RUSTFLAGS="$(RUSTFLAGS)" cargo check $(COMPILER_ARGS)

clippy:
	RUSTFLAGS="$(RUSTFLAGS)" cargo clippy $(COMPILER_ARGS)

objdump: $(KERNEL_ELF)
	rust-objdump --disassemble --demangle $(KERNEL_ELF) | rustfilt

nm: $(KERNEL_ELF)
	rust-nm --demangle --print-size $(KERNEL_ELF) | sort | rustfilt

readelf: $(KERNEL_ELF)
	readelf --headers $(KERNEL_ELF)

doc:
	cargo doc --target=$(TARGET) --features bsp_$(BSP) --document-private-items

chainboot: $(KERNEL_BIN)
	ruby utils/minipush.rb $(DEV_SERIAL) $(KERNEL_BIN)

miniterm:
	ruby utils/miniterm.rb $(DEV_SERIAL)
