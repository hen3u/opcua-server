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
$ cat /sys/class/thermal/thermal_zone0/temp
``` 
