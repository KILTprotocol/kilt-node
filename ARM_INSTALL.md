# How to cross compile to arm

download [cross compiler arm-linux-gnueabihf 8.3](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-a/downloads)

setup environment:

```fish
set -a PATH $HOME/gcc-arm-8.3-2019.03-x86_64-arm-linux-gnueabihf/bin/
set -a LIBRARY_PATH $HOME/gcc-arm-8.3-2019.03-x86_64-arm-linux-gnueabihf/arm-linux-gnueabihf/libc/lib/
set -x CXX_armv7_unknown_linux_gnueabihf arm-linux-gnueabihf-g++
set -x CC_armv7_unknown_linux_gnueabihf arm-linux-gnueabihf-gcc
``` 

edit cargo config `~/.cargo/config`

```toml
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

stuff i might have done that helped:

```
sudo apt-get install gcc-arm\*
sudo apt install libc6-armhf-cross libc6-dev-armhf-cross gcc-arm-linux-gnueabihf libdbus-1-dev libdbus-1-dev:armhf
apt-get install gcc-multilib
```
