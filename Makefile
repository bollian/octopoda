CODENAME:=octopoda
TARGET:=aarch64-unknown-none
RELEASE_ARTIFACT:=target/$(TARGET)/release/$(CODENAME)
BIN_DIR:=bin
KERNEL_IMAGE:=$(BIN_DIR)/kernel8.img

DOCKER_UTILS:=rustembedded/osdev-utils
DOCKER_RUN:=docker run -it --rm
DOCKER_WORK_DIR=-v $(shell pwd):/work -w /work

.PHONY: all clean qemu-test clippy objdump nm

# Even though the cargo build artifact _isn't_ a phony target, it's not really
# possible to get a clean list of dependencies to be imported into make.
# So we just rely on cargo detecting updated files for us
.PHONY: $(RELEASE_ARTIFACT)

all: $(KERNEL_IMAGE)

clean:
	cargo clean
	rm $(KERNEL_IMAGE)

$(RELEASE_ARTIFACT):
	cargo xrustc --target=$(TARGET) --release

$(KERNEL_IMAGE): $(RELEASE_ARTIFACT)
	mkdir -p $(BIN_DIR) # the -p is so that mkdir doesn't fail if the directory exists
	cargo objcopy -- --strip-all -O binary $(RELEASE_ARTIFACT) $(KERNEL_IMAGE)

# this is tested with qemu 4.0, and seems to require it for the raspi3 support
qemu-test:
	$(DOCKER_RUN) $(DOCKER_WORK_DIR) $(DOCKER_UTILS) \
	qemu-system-aarch64 -M raspi3 -kernel $(KERNEL_IMAGE) -d in_asm -display none -serial null -serial stdio

clippy:
	cargo xclippy --target=$(TARGET)

objdump:
	cargo objdump --target $(TARGET) -- -disassemble -print-imm-hex $(RELEASE_ARTIFACT)

nm:
	cargo nm --target $(TARGET) -- $(RELEASE_ARTIFACT)

