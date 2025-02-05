# e2e Chopsticks tests

This project is a set of end-to-end tests for the KILT protocol against other parachains.
Other functionalities such as a creation of DID can be easily added.

## Getting Started

These instructions will allow you to run the tests on your local machine for development and testing purposes.

### Prerequisites

- Node.js v20.11.0 (as specified in the [`.nvmrc`](https://github.com/nvm-sh/nvm)). With `nvm use` the right node version will be installed.
- npm (comes with Node.js)

### Installing

To install the node modules call:

```sh
yarn 
```

### Running the tests

In the package.json a script is provided.

By calling the command below, the test will be executed:

```sh
yarn test 
```

Please make sure an appropriate env is set.

### Env

The project uses environment variables for configuration.
You can find an example in the `.env-example` file. Copy this file to
.env and fill in the appropriate values.
Explanation for the values are in the `.env-example` file provided.

## Code Style

This project uses Prettier and ESLint for code formatting and linting.
The configuration for these tools can be found in the `.prettierrc` and `.eslintrc.json` files respectively.

To check your code for style issues, run:

```sh
yarn lint
```

To automatically fix style issues, run:

```sh
yarn lint:fix
```

## Adding a new test case

To add a new test case, you need to insert a new object into the list of test cases.
For example, if you want to add a new instance of `LimitedReserveTestConfiguration`, you would insert it into the `testPairsLimitedReserveTransfers` list.

The tests are configuration-driven, meaning they can be easily customized for different scenarios.
The test framework doesn't make assumptions about which parachain is sending which coin to which destination over which relay chain.
Fundamental events such as the moving of coins or the creation of a new account should be emitted during the test.

The test cases live in the tests folder and use mocks from the network directory.
The network directory contains helper functions to set the blockchain state and provides an abstraction over the chopsticks functionalities, such as creating a network.

For Adding a new test case scenario, a template folder is provided.

## Debugging Existence Tests

Each test case should have a unique ID. To execute a specific test case, run:

```sh
yarn test -t "REGULAR_EXPRESSION"
```

## UI

Vitest supports a UI to manage the test cases. To spin up the UI, call:

```sh
yarn ui
```

## CLI

The project provides a CLI to interact with the test framework.
To execute the cli run:

```sh
yarn cli [COMMAND]
```

Below are the available commands and their descriptions:

`spinUp`

Spins up the network using the definition in `./src/command/network.ts`.
The network configuration can be adjusted as needed.
A detailed step-by-step explanation of how to modify the network to a specific state is provided in the `network.ts` file.

```sh
yarn cli spinUp
```

`scheduleTx`

Executes a transaction on the network, creating a new Chopsticks instance.

```sh
yarn cli scheduleTx endpoint rawTx [options]
```

- endpoint: The endpoint of the network.
- rawTx: The raw transaction to execute.
- --origin: The origin of the transaction (default: Root).
- --port: The RPC port (default: 8888).

`stateTransition`

Shows the state transition of the network based on the latest block.
The command creates a preview folder containing an HTML file, which can be opened in a browser to inspect the state transition.

```sh
yarn cli stateTransition endpoint [option]
```

- endpoint: The endpoint of the network.
- --block: The block to do the state transition

## Built With

[TypeScript](https://www.typescriptlang.org/)
[Chopsticks](https://github.com/AcalaNetwork/chopsticks)
[Polkadot API](https://github.com/polkadot-js/api)
