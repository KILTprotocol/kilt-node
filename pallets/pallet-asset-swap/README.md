WIP.

# TODO

Besides the TODOs in the code, the following points must be tackled (ordered by importance):
* [REQUIRED FOR V1] Allow for DOT transfers to KILT to be paid with those DOTs
* [REQUIRED FOR V1] Allow for eKILT transfers to KILT (to be swapped for KILTs) to be paid with those eKILTs
* [REQUIRED FOR V2] Add hook to check the swap parameters (restricting where remote assets can be sent to).
* [REQUIRED FOR V1] Add integrity tests to check invariant that total issuance - remote balance = local pool balance (circulating supply) -> especially useful when adding token ratios, to avoid rounding errors
* [OPTIONAL] Add configurable ratio for local/remote swaps.
* [OPTIONAL] Delegate XCM message composition to a trait Config as well, depending on the destination (choosing which asset to use for payments, what amount, etc).
