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

