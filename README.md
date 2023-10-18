# Pico Alarm
This is a uni project it gets it starting point from https://github.com/rp-rs/rp2040-project-template

## What will it do
It is a basic home alarm that will detect movement

## Componets used

* Raspberry pico W
* PIR sensor 
* Buzzer
* keypad
* oled display

<!-- TABLE OF CONTENTS -->
<details open="open">
  
  <summary><h2 style="display: inline-block">Table of Contents</h2></summary>
  <ol>
    <li><a href="#markdown-header-requirements">Requirements</a></li>
    <li><a href="#installation-of-development-dependencies">Installation of development dependencies</a></li>
    <li><a href="#running">Running</a></li>
    <li><a href="#alternative-runners">Alternative runners</a></li>
    <li><a href="#notes-on-using-rp2040_boot2">Notes on using rp2040_boot2</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#code-of-conduct">Code of conduct</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
  </ol>
</details>

<!-- Requirements -->
<details open="open">
  <summary><h2 style="display: inline-block" id="requirements">Requirements</h2></summary>
  
- The standard Rust tooling (cargo, rustup) which you can install from https://rustup.rs/

- Toolchain support for the cortex-m0+ processors in the rp2040 (thumbv6m-none-eabi)

- flip-link - this allows you to detect stack-overflows on the first core, which is the only supported target for now.

- probe-run. Upstream support for RP2040 was added with version 0.3.1.

- A CMSIS-DAP probe. (J-Link and other probes will not work with probe-run)

  You can use a second
  [Pico as a CMSIS-DAP debug probe](debug_probes.md#raspberry-pi-pico). Details
  on other supported debug probes can be found in
  [debug_probes.md](debug_probes.md)

</details>

<!-- Installation of development dependencies -->
<details open="open">
  <summary><h2 style="display: inline-block" id="installation-of-development-dependencies">Installation of development dependencies</h2></summary>

```sh
rustup target install thumbv6m-none-eabi
cargo install flip-link
# This is our suggested default 'runner'
cargo install probe-run --locked
# If you want to use elf2uf2-rs instead of probe-run, instead do...
cargo install elf2uf2-rs --locked
# If you want to use any of the probe-rs tools (probe-rs run, cargo-embed, probe-rs-debugger)
cargo install probe-rs --features=cli --locked
```
If you get the error ``binary `cargo-embed` already exists`` during installation of probe-rs, run `cargo uninstall cargo-embed` to uninstall your older version of cargo-embed before trying again.

</details>


<!-- Running -->
<details open="open">
  <summary><h2 style="display: inline-block" id="running">Running</h2></summary>
  
For a debug build
```sh
cargo run
```
For a release build
```sh
cargo run --release
```

If you do not specify a DEFMT_LOG level, it will be set to `debug`.
That means `println!("")`, `info!("")` and `debug!("")` statements will be printed.
If you wish to override this, you can change it in `.cargo/config.toml` 
```toml
[env]
DEFMT_LOG = "off"
```
You can also set this inline (on Linux/MacOS)  
```sh
DEFMT_LOG=trace cargo run
```

or set the _environment variable_ so that it applies to every `cargo run` call that follows:
#### Linux/MacOS/unix
```sh
export DEFMT_LOG=trace
```

Setting the DEFMT_LOG level for the current session  
for bash
```sh
export DEFMT_LOG=trace
```

#### Windows
Windows users can only override DEFMT_LOG through `config.toml`
or by setting the environment variable as a separate step before calling `cargo run`
- cmd
```cmd
set DEFMT_LOG=trace
```
- powershell
```ps1
$Env:DEFMT_LOG = trace
```

```cmd
cargo run
```

</details>
<!-- ALTERNATIVE RUNNERS -->
<details open="open">
  <summary><h2 style="display: inline-block" id="alternative-runners">Alternative runners</h2></summary>

If you don't have a debug probe or if you want to do interactive debugging you can set up an alternative runner for cargo.  

Some of the options for your `runner` are listed below:

* **cargo embed**  
  *Step 1* - Install cargo-embed. This is part of the [`probe-rs`](https://crates.io/crates/probe-rs) crate:

  ```console
  $ cargo install probe-rs --features=cli --locked
  ```

  *Step 2* - Update settings in [Embed.toml](./Embed.toml)  
  - The defaults are to flash, reset, and start a defmt logging session
  You can find all the settings and their meanings [in the probe-rs repo](https://github.com/probe-rs/probe-rs/blob/c0610e98008cbb34d0dc056fcddff0f2d4f50ad5/probe-rs/src/bin/probe-rs/cmd/cargo_embed/config/default.toml)

  *Step 3* - Use the command `cargo embed`, which will compile the code, flash the device
  and start running the configuration specified in Embed.toml

  ```console
  $ cargo embed --release
  ```

* **probe-rs-debugger**
  *Step 1* - Install Visual Studio Code from https://code.visualstudio.com/

  *Step 2* - Install `probe-rs`
  ```console
  $ cargo install probe-rs --features=cli --locked
  ```

  *Step 3* - Open this project in VSCode

  *Step 4* - Install `debugger for probe-rs` via the VSCode extensions menu (View > Extensions)

  *Step 5* - Launch a debug session by choosing `Run`>`Start Debugging` (or press F5)

* **probe-rs run**
  *Step 1* - Install [`probe-rs`](https://crates.io/crates/probe-rs-cli):

  ```console
  $ cargo install probe-rs --features=cli --locked
  ```

  *Step 2* - Make sure your .cargo/config contains the following

  ```toml
  [target.thumbv6m-none-eabi]
  runner = "probe-rs run --chip RP2040 --protocol swd"
  ```

  *Step 3* - Use `cargo run`, which will compile the code and start the
  specified 'runner'. As the 'runner' is cargo embed, it will flash the device
  and start running immediately

  ```console
  $ cargo run --release
  ```

* **Loading a UF2 over USB**  
  *Step 1* - Install [`elf2uf2-rs`](https://github.com/JoNil/elf2uf2-rs):

  ```console
  $ cargo install elf2uf2-rs --locked
  ```

  *Step 2* - Make sure your .cargo/config contains the following

  ```toml
  [target.thumbv6m-none-eabi]
  runner = "elf2uf2-rs -d"
  ```

  The `thumbv6m-none-eabi` target may be replaced by the all-Arm wildcard
  `'cfg(all(target_arch = "arm", target_os = "none"))'`.

  *Step 3* - Boot your RP2040 into "USB Bootloader mode", typically by rebooting
  whilst holding some kind of "Boot Select" button. On Linux, you will also need
  to 'mount' the device, like you would a USB Thumb Drive.

  *Step 4* - Use `cargo run`, which will compile the code and start the
  specified 'runner'. As the 'runner' is the elf2uf2-rs tool, it will build a UF2
  file and copy it to your RP2040.

  ```console
  $ cargo run --release
  ```

* **Loading with picotool**  
  As ELF files produced by compiling Rust code are completely compatible with ELF
  files produced by compiling C or C++ code, you can also use the Raspberry Pi
  tool [picotool](https://github.com/raspberrypi/picotool). The only thing to be
  aware of is that picotool expects your ELF files to have a `.elf` extension, and
  by default Rust does not give the ELF files any extension. You can fix this by
  simply renaming the file.

  This means you can't easily use it as a cargo runner - yet.

  Also of note is that the special
  [pico-sdk](https://github.com/raspberrypi/pico-sdk) macros which hide
  information in the ELF file in a way that `picotool info` can read it out, are
  not supported in Rust. An alternative is TBC.

</details>
<!-- Notes on using rp2040_hal and rp2040_boot2 -->
<details open="open">
  <summary><h2 style="display: inline-block" id="notes-on-using-rp2040_boot2">Notes on using rp2040_boot2</h2></summary>

  The second-stage boot loader must be written to the .boot2 section. That
  is usually handled by the board support package (e.g.`rp-pico`). If you don't use
  one, you should initialize the boot loader manually. This can be done by adding the
  following to the beginning of main.rs:
  ```rust
  use rp2040_boot2;
  #[link_section = ".boot2"]
  #[used]
  pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;
  ```

</details>

## Code of Conduct

Contribution to this crate is organized under the terms of the [Rust Code of
Conduct][CoC], and the maintainer of this crate, the [rp-rs team], promises
to intervene to uphold that code of conduct.

[CoC]: CODE_OF_CONDUCT.md
[rp-rs team]: https://github.com/orgs/rp-rs/teams/rp-rs

## License

The contents of this repository are dual-licensed under the _MIT OR Apache
2.0_ License. That means you can chose either the MIT licence or the
Apache-2.0 licence when you re-use this code. See `MIT` or `APACHE2.0` for more
information on each specific licence.

