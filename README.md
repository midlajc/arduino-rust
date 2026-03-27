## Prerequisites

### 1. Install Rust

Install `rustup` if it is not already on your machine:

```sh
curl https://sh.rustup.rs -sSf | sh
```

Then install the toolchain this repo expects:

```sh
rustup toolchain install nightly-2025-04-27 --component rust-src
```

### 2. Install AVR tooling

Install the Arduino flashing tools for your OS.

macOS:

```sh
xcode-select --install
brew tap osx-cross/avr
brew install avr-gcc avrdude
```

Ubuntu / Debian:

```sh
sudo apt install avr-libc gcc-avr pkg-config avrdude libudev-dev build-essential
```

Windows:

```powershell
winget install AVRDudes.AVRDUDE ZakKemble.avr-gcc
```

### 3. Install `ravedude`

`ravedude` is the runner used by both projects, so `cargo run` will build, flash, and open the serial console.

```sh
cargo +stable install --locked ravedude
```

## Hardware Setup

### `hello-world`

- Connect an Arduino Uno over USB.
- No extra wiring is required because it toggles the built-in LED on `D13`.

### `color-wheel`

- Connect a common cathode RGB LED:
  - `D9` -> red channel
  - `D10` -> green channel
  - `D11` -> blue channel
- Add appropriate current-limiting resistors for each LED channel.
- Connect your Bluetooth serial module:
  - Bluetooth `TX` -> Arduino `D3`
  - Arduino `D4` is held high in software and reserved for transmit
- Connect the Arduino Uno over USB for flashing and serial logs.

## Build And Run

Run commands from the project you want to flash.

### `hello-world`

```sh
cd hello-world
cargo run
```

### `color-wheel`

```sh
cd color-wheel
cargo run
```