cargo build --manifest-path=blackboard/Cargo.toml --target=thumbv6m-none-eabi &&
cargo objcopy --target=thumbv6m-none-eabi --bin blackboard -- -O ihex blackboard.hex &&
/usr/bin/JLinkExe -CommandFile command_file.jlink
