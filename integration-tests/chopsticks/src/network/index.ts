import KiltConfigs from './kilt/index.js'
import HydrationConfigs from './hydration/index.js'
import PolkadotConfigs from './polkadot/index.js'
import AssetHubConfigs from './assethub/index.js'
import { ChainConfigs } from './types.js'

/*
 * Get an environment variable and throw an error if it is not set.
 */
function getRequiredEnvVariable(name: string): string {
	const value = process.env[name]
	if (value === undefined) {
		throw new Error(`Environment variable ${name} is not set.`)
	}
	return value
}

/*
 * Object with all the chain configurations.
 */
export const testChains: ChainConfigs = {
	kilt: {
		getConfig: KiltConfigs.test.getSetupOptions,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('SPIRITNET_BLOCK_NUMBER')),
			wasmOverride: getRequiredEnvVariable('SPIRITNET_WASM_OVERRIDE'),
		},
		storage: KiltConfigs.test.storage,
		chainInfo: KiltConfigs.test.parachainInfo,
	},
	hydration: {
		getConfig: HydrationConfigs.test.getSetupOptions,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('HYDRADX_BLOCK_NUMBER')),
		},
		storage: HydrationConfigs.test.storage,
		chainInfo: HydrationConfigs.test.parachainInfo,
	},
	polkadot: {
		getConfig: PolkadotConfigs.test.getSetupOptions,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('POLKADOT_BLOCK_NUMBER')),
		},
		storage: PolkadotConfigs.test.storage,
		chainInfo: {},
	},
	assetHub: {
		getConfig: AssetHubConfigs.test.getSetupOptions,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('ASSETHUB_BLOCK_NUMBER')),
		},
		storage: AssetHubConfigs.test.storage,
		chainInfo: AssetHubConfigs.test.parachainInfo,
	},
}

export const mainChains: ChainConfigs = {
	kilt: {
		getConfig: KiltConfigs.main.getSetupOptions,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('SPIRITNET_BLOCK_NUMBER')),
			wasmOverride: getRequiredEnvVariable('SPIRITNET_WASM_OVERRIDE'),
		},
		storage: KiltConfigs.main.storage,
		chainInfo: KiltConfigs.main.parachainInfo,
	},
	hydration: {
		getConfig: HydrationConfigs.main.getSetupOptions,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('HYDRADX_BLOCK_NUMBER')),
		},
		storage: HydrationConfigs.main.storage,
		chainInfo: HydrationConfigs.main.parachainInfo,
	},
	polkadot: {
		getConfig: PolkadotConfigs.main.getSetupOptions,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('POLKADOT_BLOCK_NUMBER')),
		},
		storage: PolkadotConfigs.main.storage,
		chainInfo: {},
	},
	assetHub: {
		getConfig: AssetHubConfigs.main.getSetupOptions,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('ASSETHUB_BLOCK_NUMBER')),
		},
		storage: AssetHubConfigs.main.storage,
		chainInfo: AssetHubConfigs.main.parachainInfo,
	},
}
