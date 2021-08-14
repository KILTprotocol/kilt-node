# Building .deb
Packages for debian are created using the [cargo-deb] plugin for cargo.

## Install plugin cargo-deb
Start by installing [cargo-deb]:

    cargo install cargo-deb

## Build packages
Build the packages:

    cargo deb 

## Maintainers
The packages are maintained by [Dwellir] and contributed upstream.


## Attributions
[Erik Lönroth] - Creating the initial deb versions, part of the [Dwellir] team.

[Joakim Nyman] - Additions to the debs, part of the [Dwellir] team.

[Michael Murphy] - Maintainer of the [cargo-deb]



[cargo-deb]: https://github.com/mmstick/cargo-deb
[Dwellir]: https://dwellir.com
[Erik Lönroth]: https://eriklonroth.com
[Joakim Nyman]: https://github.com/Maharacha
[Michael Murphy]: https://github.com/mmstick
