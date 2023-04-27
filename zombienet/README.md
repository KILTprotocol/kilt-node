# Setup

Download Zombienet version [1.3.34](https://github.com/paritytech/zombienet/releases/download/v1.3.34/zombienet-linux-x64) and make the binary executable. Zombienet is currently under heavy development. Other versions might not work with the current defined network configuration.
Make sure you have a correct configured kubectl and all env variable, which are described in the tests/.env-example file, are set.
To set up another provider, have a look at the [official documentation](https://paritytech.github.io/zombienet/).

The provided network configuration spawns 3 parachain nodes and 2 validator nodes.

To run the network:
```
zombienet spawn network-config.toml
```

There are two ways, to execute a test. If you do not have a spawned network, you can simply execute:

```
zombienet tests FILENAME.zndsl
```

This will create the network and perform the tests. After all tests are finished, the network is destroyed.

If you already have a spawned network, you have to look up the ´runningNetworkSpec´ which is typically in  /tmp/zombie-{HASH}/zombie.json located for the k8s provider.

An example call would be:


```
zombienet test {FILENAME}.zndsl  /tmp/zombie-{HASH}/zombie.json
```

## Known Issues

### Kubernetes provider

1. If you face this error:

```
Error:          Error: Command failed with exit code 1: kubectl --kubeconfig /home/{USER}/.kube/config --namespace zombie-087286c76d4301bac2d39f96a4a97698 cp temp-collator:/cfg/genesis-state /tmp/zombie-087286c76d4301bac2d39f96a4a97698_-1512996-vZpBh73wdvtR/2000/genesis-state
error: unable to upgrade connection: container not found ("temp-collator")
Defaulted container "temp-collator" out of: temp-collator, transfer-files-container (init)
```

Make sure, you have export all env variable. The env variables defines the used docker images. An example is provided in tests/.env-example.

2. If prometheus is not working and you use minikube, delete your minikube instance and start the process again.

### Docker

Zombienet requires to have bash. Docker images without bash can not be tested or spawned.
