# Reset Spec

Reset chain specs only when we start a chain from block #0 again.

This script uses docker images.

example usage:

```
python3 .maintain/reset-spec/ -i kiltprotocol/peregrine:develop --peregrine --peregrine-stg --peregrine-dev
python3 .maintain/reset-spec/ -i parity/polkadot:v0.9.8 --peregrine-relay --peregrine-relay-stg
```
