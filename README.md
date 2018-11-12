# substrate-poc
A new SRML-based Substrate node, ready for hacking

## Run inside docker container

```
docker build -t substrate-poc . 
docker run -p 9933:9933 -p 9944:9944 -p 30333:30333 --publish-all=true -it substrate-poc
```
