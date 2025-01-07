import * as SpiritnetConfig from './spiritnet.js'
import * as PolkadotConfig from './relay.js'
import * as HydraDxConfig from './hydraDx.js'
import * as AssetHubConfig from './assethub.js'
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
export const all: ChainConfigs = {
	spiritnet: {
		getConfig: SpiritnetConfig.getSetupOptions,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('SPIRITNET_BLOCK_NUMBER')),
			wasmOverride: getRequiredEnvVariable('SPIRITNET_WASM_OVERRIDE'),
		},
	},
	hydraDx: {
		getConfig: HydraDxConfig.getSetupOptions,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('HYDRADX_BLOCK_NUMBER')),
		},
	},
	polkadot: {
		getConfig: PolkadotConfig.getSetupOptions,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('POLKADOT_BLOCK_NUMBER')),
		},
	},
	assetHub: {
		getConfig: AssetHubConfig.getSetupOptions,
		parameters: {
			blockNumber: Number(getRequiredEnvVariable('ASSETHUB_BLOCK_NUMBER')),
		},
	},
}
