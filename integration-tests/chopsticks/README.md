# e2e Chopsticks tests

This project is a set of end-to-end tests for the KILT protocol against other parachains. 
Other functionalities such as a creation of DID can be easy inserted.

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

The project uses environment variables for configuration. You can find an example in the `.env-example` file. Copy this file to 
.env and fill in the appropriate values. Explanation for the values are in the `.env-example` file provided.


## Built With 

[TypeScript](https://www.typescriptlang.org/)
[Chopsticks](https://github.com/AcalaNetwork/chopsticks)
[Polkadot API](https://github.com/polkadot-js/api)

## Code Style 

This project uses Prettier and ESLint for code formatting and linting. The configuration for these tools can be found in the `.prettierrc` and `.eslintrc.json` files respectively.

To check your code for style issues, run:

```sh
yarn lint
```

To automatically fix style issues, run:

```sh
yarn lint:fix
```

## Adding a new test case

To add a new test case, you need to insert a new object into the list of test cases. For example, if you want to add a new instance of `LimitedReserveTestConfiguration`, you would insert it into the `testPairsLimitedReserveTransfers` list. Here's a step-by-step guide:


- Create a new instance of LimitedReserveTestConfiguration. Make sure to fill in all the necessary parameters for the test.
- Insert this new instance into the testPairsLimitedReserveTransfers list.

The tests are configuration-driven, meaning they can be easily customized for different scenarios. The test framework doesn't make assumptions about which parachain is sending which coin to which destination over which relay chain. Fundamental events such as the moving of coins or the creation of a new account should be emitted during the test.

The test cases live in the tests folder and use mocks from the network directory. The network directory contains helper functions to set the blockchain state and provides an abstraction over the chopsticks functionalities, such as creating a network.
