# Embassy on XIAO

## Prerequisites

```
pip3 install --user adafruit-nrfutil
cargo install cargo-binutils
```

## Bootloader

https://github.com/adafruit/Adafruit_nRF52_Bootloader

https://github.com/adafruit/Adafruit_nRF52_nrfutil

https://github.com/NordicSemiconductor/nrf-udev

## Before you start
```
rustup target add thumbv7m-none-eabi
```

```bash
cargo build --bin blinky_cli && 
cargo objcopy --target=thumbv7m-none-eabi --bin blinky_cli -- -O ihex blinky_cli.hex && 
adafruit-nrfutil dfu genpkg --dev-type 0x0052 --application blinky_cli.hex blinky_cli.zip &&
echo reset >> /dev/ttyACM0 &&
sleep 5 &&
adafruit-nrfutil dfu serial --package blinky_cli.zip -p /dev/ttyACM0 -b 115200 --singlebank
```

## Datasheets

https://files.seeedstudio.com/wiki/XIAO-BLE/Seeed-Studio-XIAO-nRF52840-Sense-v1.1.pdf
