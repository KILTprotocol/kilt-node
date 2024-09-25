# KILT Chopsticks utilities

This project contains setup code to spawn [Chopsticks](https://github.com/AcalaNetwork/chopsticks)-based deployments for both Spiritnet and Peregrine runtimes.

## How to spawn

First, `cd` into the `chipsticks` folder and run first `nvm use` to configure the right Node version, and then `yarn` to install any Node dependencies.

To spin up a deployment using the actual state currently on Spiritnet or Peregrine, run either `yarn peregrine:spawn` or `yarn spiritnet:spawn`.

### Add customizations

By default, the config for each chain specifies only the endpoint to fetch the state from and the path to store the db folder.

Any additional information, such as storage or WASM overrides, or port number specifications, can be specified using the same YAML format in each folder's `extra.yaml` file.
These files are not tracked by git and are specific for each user's machine.
If a chain does not include an `extra.yaml` file, the default config specified in the chain's `config.yaml` file will be used.

A list of example configurations is given in the [examples](./examples/) folder.

So, if for example the sudo key for the Peregrine network must be overridden, follow these steps:

1. Create the file `runtimes/peregrine/kilt/extra.yaml`.
2. Copy the content of [`storage.example.yaml`](./examples/storage.example.yaml) into `runtimes/peregrine/kilt/extra.yaml`.
3. From within the `chopsticks` directory run `yarn peregrine:spawn`.

This process can be applied to every chain folder, also for multiple within the same environment, e.g., the sudo key for PILT and the sudo key for Paseo can be overridden by having two `extra.yaml` files within each folder.

## How it works

When spawning a network, a temporary file called `.tmp.yaml` will be generated within each chain's folder.
**THIS DOES NOT HAVE TO BE MANUALLY EDITED**, and it will be automatically cleaned up whenever the network spawning process is stopped.
If, for any reason, this should not be the case, you can run `peregrine:cleanup` or `spiritnet:cleanup` to clean up these temporary files.
