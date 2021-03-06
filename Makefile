#
# MIT License
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#

TARGET = aarch64-unknown-none

SOURCES = $(wildcard **/*.rs) $(wildcard **/*.S) link.ld


XRUSTC_CMD   = cargo xrustc --target=$(TARGET) --release
CARGO_OUTPUT = target/$(TARGET)/release/kernel8

OBJCOPY        = cargo objcopy --
OBJCOPY_PARAMS = --strip-all -O binary

CONTAINER_UTILS   = andrerichter/raspi3-utils

DOCKER_CMD        = docker run -it --rm
DOCKER_ARG_CURDIR = -v $(shell pwd):/work -w /work

DOCKER_EXEC_QEMU     = qemu-system-aarch64 -M raspi3 -kernel kernel8.img

.PHONY: all qemu clippy clean objdump nm

all: clean kernel8.img

$(CARGO_OUTPUT): $(SOURCES)
	$(XRUSTC_CMD)

kernel8.img: $(CARGO_OUTPUT)
	cp $< .
	$(OBJCOPY) $(OBJCOPY_PARAMS) $< kernel8.img

qemu: all
	$(DOCKER_CMD) $(DOCKER_ARG_CURDIR) $(CONTAINER_UTILS) \
	$(DOCKER_EXEC_QEMU) -serial stdio

clippy:
	cargo xclippy --target=$(TARGET)

clean:
	cargo clean

objdump:
	cargo objdump --target $(TARGET) -- -disassemble -print-imm-hex kernel8_debug

llvm:
	cargo llvm-ir --target $(TARGET)  kernel8_debug

nm:
	cargo nm --target $(TARGET) -- kernel8_debug | sort

gdb: clean $(SOURCES)
	 cargo xrustc --target=$(TARGET) --release -- -C debuginfo=2
	 cp target/aarch64-unknown-none/release/kernel8 kernel8_debug

gdb-opt0: clean $(SOURCES)
	$(call gen_gdb,-C debuginfo=2 -C opt-level=0)

dump: clean $(SOURCES)
	# cargo xrustc --target=$(TARGET) --release -- -C debuginfo=2 -C opt-level=0
	cargo xrustc --target=$(TARGET) --release -- -C debuginfo=2
	cp target/aarch64-unknown-none/release/kernel8 kernel8_debug
	cargo objdump --target $(TARGET) -- -disassemble -print-imm-hex kernel8_debug > dump.s

expand:
	cargo expand