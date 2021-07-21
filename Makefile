CODENAME := octopoda

# default to building for the RPi3
BSP ?= rpi3

# default device name to use when opening serial terminal/minipush
DEV_SERIAL ?= /dev/ttyUSB0

# bsp-specific arguments
ifeq ($(BSP),rpi3)
	TARGET            := aarch64-unknown-none-softfloat
	OUTPUT            := kernel8.img
	LINKER_FILE       := src/bsp/aarch64.ld
	RUSTC_MISC_ARGS   := -C target-cpu=cortex-a53

	QEMU_BINARY       := qemu-system-aarch64
	QEMU_MACHINE_TYPE := raspi3
	QEMU_RELEASE_ARGS := -serial stdio -display none
else ifeq ($(BSP),rpi4)
	TARGET            := aarch64-unknown-none-softfloat
	OUTPUT            := kernel8.img
	LINKER_FILE       := src/bsp/aarch64.ld
	RUSTC_MISC_ARGS   := -C target-cpu=cortex-a72

	QEMU_BINARY       := qemu-system-aarch64
	QEMU_MACHINE_TYPE :=
	QEMU_RELEASE_ARGS := -serial stdio -display none
else ifeq ($(BSP),x86_64)
	TARGET            := targets/x86_64-unknown-none-softfloat.json
	OUTPUT            := bootimage-$(CODENAME).bin
	RUSTC_MISC_ARGS   :=
	BIN_DIR           := target/x86_64-unknown-none-softfloat/release

	QEMU_BINARY       := qemu-system-x86_64
	QEMU_MACHINE_TYPE :=
	QEMU_RELEASE_ARGS := -serial stdio -display none
endif

ifdef LINKER_FILE
	# export for build.rs
	export LINKER_FILE
	LINKER_ARGS := -C link-arg=-T$(LINKER_FILE)
else
	LINKER_ARGS :=
endif

ifdef PRINT_ASM
	QEMU_BINARY_EXTRA_FLAGS := -d in_asm
else
	QEMU_BINARY_EXTRA_FLAGS :=
endif

RUSTFLAGS     := $(LINKER_ARGS) $(RUSTC_MISC_ARGS)
COMPILER_ARGS := --no-default-features --features bsp_$(BSP) --target=$(TARGET) --release
ifndef BIN_DIR
	BIN_DIR   := target/$(TARGET)/release
endif
KERNEL_BIN    := $(BIN_DIR)/$(OUTPUT)
KERNEL_ELF    := $(BIN_DIR)/$(CODENAME)

SOURCES := $(wildcard src/*.rs) $(wildcard src/**/*.rs) \
	$(wildcard src/**/*.ld) Cargo.toml Cargo.lock Makefile rust-toolchain.toml

.PHONY: all clean check qemu-test clippy objdump nm readelf chainboot doc $(KERNEL_ELF)

all: $(KERNEL_BIN)

clean:
	cargo clean

ifeq ($(BSP),x86_64)
$(KERNEL_BIN): $(KERNEL_ELF)
	RUSTFLAGS="$(RUSTFLAGS)" cargo bootimage $(COMPILER_ARGS)
else
$(KERNEL_BIN): $(KERNEL_ELF)
	rust-objcopy --strip-all -O binary $(KERNEL_ELF) $(KERNEL_BIN)
endif

$(KERNEL_ELF): $(SOURCES)
	RUSTFLAGS="$(RUSTFLAGS)" cargo rustc $(COMPILER_ARGS)

ifeq ($(BSP),x86_64)
qemu-test: $(KERNEL_BIN)
	qemu-system-x86_64 -drive format=raw,file=$(KERNEL_BIN)
else ifeq ($(QEMU_MACHINE_TYPE),)
	@echo "This machine isn't currently supported by qemu"
else
qemu-test: $(KERNEL_BIN)
	qemu-system-aarch64 -M $(QEMU_MACHINE_TYPE) -kernel $(KERNEL_BIN) \
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
