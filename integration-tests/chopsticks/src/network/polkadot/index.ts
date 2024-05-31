import * as SpiritnetConfig from './spiritnet.js'
import * as PolkadotConfig from './relay.js'
import * as HydraDxConfig from './hydraDx.js'
import { ChainConfigs } from '../types.js'

function getEnvVariable(name: string): number {
	const value = process.env[name]
	if (value === undefined) {
		throw new Error(`Environment variable ${name} is not set.`)
	}
	return Number(value)
}

export const all: ChainConfigs = {
	spiritnet: {
		config: SpiritnetConfig.getSetupOptions,
		blockNumber: getEnvVariable('SPIRITNET_BLOCK_NUMBER'),
		name: 'spiritnet',
	},
	hydraDx: {
		config: HydraDxConfig.getSetupOptions,
		blockNumber: getEnvVariable('HYDRADX_BLOCK_NUMBER'),
		name: 'hydradx',
	},
	polkadot: {
		config: PolkadotConfig.getSetupOptions,
		blockNumber: getEnvVariable('POLKADOT_BLOCK_NUMBER'),
		name: 'polkadot',
	},
}
