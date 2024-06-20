WIP.

# TODO

Besides the TODOs in the code, the following points must be tackled:
* Delegate XCM message composition to a trait Config as well, depending on the destination (choosing which asset to use for payments, what amount, etc).
* Add hook to check the swap parameters (restricting where remote assets can be sent to).
* Add configurable ratio for local/remote swaps.
* Adjust error types and logs
