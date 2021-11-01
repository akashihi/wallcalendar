#!/usr/bin/env sh

cargo size --bin fw --release
cargo objcopy --bin fw --target thumbv7em-none-eabihf --release -- -O binary fw.bin
st-flash erase
st-flash write fw.bin 0x8000000
st-flash reset
