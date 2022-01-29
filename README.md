# rusty-light
Awesome flashlight firmware written in rust

## Hardware
[Blackboard](./blackboard/README.md) targets
[STM32 Nucleo-32 boards (MB1180)](https://www.st.com/resource/en/user_manual/dm00231744-stm32-nucleo32-boards-mb1180-stmicroelectronics.pdf)
boards.

## LED drivers
One or two LED drivers are required to drive LEDs. Drivers are controlled with PWM.
You can use boost drivers with 3 LEDs or buck drivers with one LED.
We have some drivers of our own design using LT3477 as well as
[Led Senser V2](https://www.ledtreiber.de/shop/Led-Senser-V2-R-2-%E2%80%A2-100-1350mA-%E2%80%A2-2-6V~18V-p164952213)
and [Led Senser Xtreme R.2](https://www.ledtreiber.de/shop/Led-Senser-Xtreme-R-2-%E2%80%A2-200-3050mA-%E2%80%A2-6V~40V-p164951429).

## Building and running
### On the blackboard
See [Blackboard README.md](blackboard/README.md)

Prerequisites:
[JLink](https://www.segger.com/downloads/jlink/)

```
rustup target add thumbv6m-none-eabi
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

Build and flash:

```
cargo build --manifest-path=blackboard/Cargo.toml --target=thumbv6m-none-eabi
cargo objcopy --target=thumbv6m-none-eabi --bin blackboard -- -O ihex blackboard.hex
/usr/bin/JLinkExe -CommandFile command_file.jlink
```

or simply `./flash.sh`
### Console

`sudo apt install libx11-dev`

`cargo run --bin console_sim`