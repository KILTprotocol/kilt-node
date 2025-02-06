import { chains as KiltConfigs } from './kilt/index.js'
import { chains as HydrationConfigs } from './hydration/index.js'
import { chains as PolkadotConfigs } from './polkadot/index.js'
import { chains as AssetHubConfigs } from './assethub/index.js'
import { ChainConfigs } from './types.js'

/*
 * Get an environment variable and throw an error if it is not set.
 */
function getRequiredEnvVariable(name: string): string {
	const value = process.env[name]
	if (value === undefined) {
		if (process.env.NODE_ENV === 'test') {
			throw new Error(`Error: Environment variable ${name} is not set. Some tests might not work.`)
		}
		return ''
	}
	return value
}

/*
 * Object with all the chain configurations.
 */
export const testChains: ChainConfigs = {
	kilt: {
		getConfig: KiltConfigs.test!.getConfig,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('SPIRITNET_BLOCK_NUMBER')),
			wasmOverride: getRequiredEnvVariable('SPIRITNET_WASM_OVERRIDE'),
		},
		storage: KiltConfigs.test!.storage,
		chainInfo: KiltConfigs.test!.chainInfo,
	},
	hydration: {
		getConfig: HydrationConfigs.test!.getConfig,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('HYDRATION_BLOCK_NUMBER')),
		},
		storage: HydrationConfigs.test!.storage,
		chainInfo: HydrationConfigs.test!.chainInfo,
	},
	polkadot: {
		getConfig: PolkadotConfigs.test!.getConfig,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('POLKADOT_BLOCK_NUMBER')),
		},
		storage: PolkadotConfigs.test!.storage,
		chainInfo: PolkadotConfigs.test!.chainInfo,
	},
	assetHub: {
		getConfig: AssetHubConfigs.test!.getConfig,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('ASSETHUB_BLOCK_NUMBER')),
		},
		storage: AssetHubConfigs.test!.storage,
		chainInfo: AssetHubConfigs.test!.chainInfo,
	},
}

export const mainChains: ChainConfigs = {
	kilt: {
		getConfig: KiltConfigs.main!.getConfig,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('SPIRITNET_BLOCK_NUMBER')),
			wasmOverride: getRequiredEnvVariable('SPIRITNET_WASM_OVERRIDE'),
		},
		storage: KiltConfigs.main!.storage,
		chainInfo: KiltConfigs.main!.chainInfo,
	},
	hydration: {
		getConfig: HydrationConfigs.main!.getConfig,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('HYDRATION_BLOCK_NUMBER')),
		},
		storage: HydrationConfigs.main!.storage,
		chainInfo: HydrationConfigs.main!.chainInfo,
	},
	polkadot: {
		getConfig: PolkadotConfigs.main!.getConfig,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('POLKADOT_BLOCK_NUMBER')),
		},
		storage: PolkadotConfigs.main!.storage,
		chainInfo: PolkadotConfigs.main!.chainInfo,
	},
	assetHub: {
		getConfig: AssetHubConfigs.main!.getConfig,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('ASSETHUB_BLOCK_NUMBER')),
		},
		storage: AssetHubConfigs.main!.storage,
		chainInfo: AssetHubConfigs.main!.chainInfo,
	},
}
