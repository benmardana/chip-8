# Chip-8 Emulator

A Chip-8 Emulator written in Rust. This project is just for fun to give me an interesting way to learn Rust and learn more about emulators/interpreters.

My primary resource was this high level guide: https://tobiasvl.github.io/blog/write-a-chip-8-emulator/

I used the test roms from this repository to get up and running and then later on to iron out quirks: https://github.com/Timendus/chip8-test-suite/tree/main

## Requirements

### SDL 2

Install via your favourite package manager, or the website: https://www.libsdl.org/

## Usage

```bash

cargo run PATH [--hertz=NUM]
```

A selection of useful roms are included in the [/roms](/roms/) folder.
