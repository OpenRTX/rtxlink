# rtxlink
Command line utility to exchange data with radios running OpenRTX

## Usage
* Install the rust toolchain using [rustup](https://rustup.rs/)
* Clone this repository and enter the `rtxlink` folder
* Run `cargo run` to compile and run a debug build \
You can append rtxlink parameters to the `cargo run` command.

## Troubleshooting
* If you get this build error on Fedora
```
error: failed to run custom build command for `libudev-sys v0.1.4
```
You may be missing `libudev`, install it with
```
sudo dnf install systemd-devel
```
