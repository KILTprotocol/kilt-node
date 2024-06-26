WIP.

# TODO

Besides the TODOs in the code, the following points must be tackled (ordered by importance):
* [REQUIRED FOR V1] Add integrity tests to check invariant that total issuance - remote balance == local pool balance (circulating supply) -> especially useful when adding token ratios, to avoid rounding errors
* [REQUIRED FOR V2] Add hook to check the swap parameters (restricting where remote assets can be sent to).
* [REQUIRED FOR V2] Add constraints about which beneficiary people can send their tokens to.
* [REQUIRED FOR V2] Improve logging, also for cases other than errors. Check how XCM implements them.
* Add swap back event from eKILT -> KILT.
* [OPTIONAL] Add configurable ratio for local/remote swaps.
* [OPTIONAL] Delegate XCM message composition to a trait Config as well, depending on the destination (choosing which asset to use for payments, what amount, etc).
