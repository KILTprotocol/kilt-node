# substrate-poc
A new SRML-based Substrate node, ready for hacking

## Run inside docker container

```
docker build -t substrate-poc . 
docker run -p 9933:9933 -p 9944:9944 -p 30333:30333 --publish-all=true -it substrate-poc
```

## Running a local node that connects to the KILT prototype test net

There are 2 boot nodes running in the KILT test net:

* bootnode-alice
* bootnode-bob

To connect to the Alice node you can use the shell script `connect.sh`:

```
connect.sh --key Charly --name "CHARLY"
``` 

You can use any of the accounts declared in the chain spec to connect.
