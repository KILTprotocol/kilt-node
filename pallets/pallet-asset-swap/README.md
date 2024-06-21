WIP.

# TODO

Besides the TODOs in the code, the following points must be tackled (ordered by importance):
* Add hook to check the swap parameters (restricting where remote assets can be sent to).
* Add integrity tests to check invariant that total issuance - remote balance = local pool balance (circulating supply) -> especially useful when adding token ratios, to avoid rounding errors
* Add configurable ratio for local/remote swaps.
* Delegate XCM message composition to a trait Config as well, depending on the destination (choosing which asset to use for payments, what amount, etc).
