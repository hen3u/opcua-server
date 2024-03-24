# Build rust opcua server

## Installation
```sh
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ sudo apt install libssl-dev
`̀``

## Build
```sh
$ cargo build --release
`̀``

## Read CPU temperature
```sh
$ sudo apt install lm-sensors
$ cat /sys/class/thermal/thermal_zone0/temp
```

## Generate bitbake recipe from Cargo based project
```sh
$ cargo bitbake
$ sudo apt install librust-cargo+openssl-dev
```

# Cross-Compilation for debug
```
#sudo apt-get install gcc-aarch64-linux-gnu
rustup target add armv7-unknown-linux-gnueabihf
#rustup target add aarch64-unknown-linux-gnu
cargo build --target armv7-unknown-linux-gnueabihf --features static_ssl
scp target/armv7-unknown-linux-gnueabihf/debug/rust-server hostname@ipaddress:~
#PKG_CONFIG_SYSROOT_DIR=${HOME}/sysroot-arm64 cargo build --release --target=aarch64-unknown-linux-gnu
```

# Sources
https://stackoverflow.com/questions/37375712/cross-compile-rust-openssl-for-raspberry-pi-2
https://github.com/cross-rs/cross/issues/229#issuecomment-597898074
