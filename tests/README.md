# Testing

Make sure you have a correct configured kubectl.
In the [official documentation](https://paritytech.github.io/zombienet/) of Zombienet, more providers are shown.

To run the network do:

zombienet spawn tests/network-configuration.toml

## Known Issues

If you face this error:

 Error:          Error: Command failed with exit code 1: kubectl --kubeconfig /home/{USER}/.kube/config --namespace zombie-087286c76d4301bac2d39f96a4a97698 cp temp-collator:/cfg/genesis-state /tmp/zombie-087286c76d4301bac2d39f96a4a97698_-1512996-vZpBh73wdvtR/2000/genesis-state
error: unable to upgrade connection: container not found ("temp-collator")
Defaulted container "temp-collator" out of: temp-collator, transfer-files-container (init)

Restart your minikube instance.
