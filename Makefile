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

SOURCES = $(wildcard **/*.rs) $(wildcard **/*.S) link.ld

CARGO_OUTPUT = target/aarch64-unknown-none/release/kernel8

OBJCOPY_ARGS = --strip-all -O binary

.PHONY: all clippy clean objdump nm

all: clean kernel8.img

$(CARGO_OUTPUT): $(SOURCES)
	cargo build --release

kernel8.img: $(CARGO_OUTPUT)
	cargo objcopy --release -- $(OBJCOPY_ARGS) kernel8.img

clippy:
	cargo clippy

clean:
	cargo clean

objdump:
	cargo objdump -- --disassemble --print-imm-hex

llvm:
	cargo rustc --release -- --emit=llvm-ir
# => target/aarch64-unknown-none/release/deps/*.ll

nm:
	cargo nm

gdb: clean $(SOURCES)
	 cargo rustc --release -- -C debuginfo=2
	 cp $(CARGO_OUTPUT) kernel8_debug

gdb-opt0: clean $(SOURCES)
	$(call gen_gdb,-C debuginfo=2 -C opt-level=0)

dump: $(SOURCES)
	cargo objdump --release -- --disassemble --print-imm-hex > dump.s

expand:
	cargo expand
# cargo-expand is required: cargo install cargo-expand

