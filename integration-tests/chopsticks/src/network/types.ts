import type { SetupOption, setupContext } from '@acala-network/chopsticks-testing'

export type Config = Awaited<ReturnType<typeof setupContext>>

/**
 * `ChainConfig` is an interface that represents the configuration for a blockchain.
 *
 * @interface ChainConfig
 *
 * @property {Function} getConfig - A function that takes an object as an argument.
 * The object can have two optional properties: `blockNumber` and `wasmOverride`.
 * `blockNumber` is a number that represents the block number in the blockchain, which can be set as env variable.
 * `wasmOverride` is a string that can be used to override the WASM code for the blockchain, which can also be set as env variable.
 * The function returns a `SetupOption` object.
 *
 * @property {Object} parameters - An object that contains the parameters for the getConfig function.
 * It has two optional properties: `blockNumber` and `wasmOverride`.
 */
export interface ChainConfig {
	getConfig: ({ blockNumber, wasmOverride }: { blockNumber?: number; wasmOverride?: string }) => SetupOption
	parameters: {
		blockNumber?: number
		wasmOverride?: string
	}
}

/// A Record of all possible chain configurations.
export type ChainConfigs = Record<string, ChainConfig>
