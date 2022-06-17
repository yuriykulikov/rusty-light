cargo +nightly build --bin stm32-nucleo --target=thumbv6m-none-eabi &&
cargo +nightly objcopy --target=thumbv6m-none-eabi --bin stm32-nucleo -- -O ihex stm32-nucleo.hex &&
/usr/bin/JLinkExe -CommandFile command_file.jlink
