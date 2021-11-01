cargo size --bin fw --release
cargo objcopy --bin fw --target thumbv7em-none-eabihf --release -- -O binary fw.bin
ST-LINK_CLI.EXE -ME
ST-LINK_CLI.exe -P fw.bin 0x08000000
ST-LINK_CLI.exe -Rst
