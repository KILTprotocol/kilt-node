# prototype-chain

Substrate node implementation for the KILT prototype

## Running a local node that connects to the KILT prototype test environment in AWS

There are 2 boot nodes running in the KILT test net:

* bootnode-alice
* bootnode-bob

To connect to the Alice node you can use the shell script `connect.sh`:

```
connect.sh --key Charly --name "CHARLY"
``` 

You can use any of the accounts declared in the chain spec to connect (Alice, Bob, Charly, Dave, Eve, Ferdie).
