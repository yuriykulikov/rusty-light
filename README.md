# rusty-light
Awesome flashlight firmware written in rust

## What is it all about
In better times programmers had to write a lot of code themselves.
Need a hashmap? Implement one! Linked list? - better start hacking.
OS? Send an email and get going.

It was fun! Now these days are over. There are libraries for everything.
We get it. We use libraries at work. It makes sense, but, here we can have some fun!
This is all this project is about - get creative!

## Crazy? Yes, please!
There are no limits on over engineering here.

## The IKEA effect
To keep the motivation up, we are going to use these devices for our daily commute
and, occasionally, a nice bike trip.

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