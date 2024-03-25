import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'

export const options: SetupOption = {
	endpoint: ['wss://rpc.ibp.network/polkadot', 'wss://rpc.polkadot.io', 'wss://polkadot-rpc.dwellir.com'],
	db: './db/polkadot.db.sqlite',
	port: 8000,
	runtimeLogLevel: 5,
}

export const defaultStorage = (addr: string) => ({
	// give addr some coins
	System: {
		Account: [[[addr], { providers: 1, data: { free: 1000 * 1e12 } }]],
	},
	ParasDisputes: {
		// those can makes block building super slow
		$removePrefix: ['disputes'],
	},
	Dmp: {
		// clear existing dmp to avoid impact test result
		$removePrefix: ['downwardMessageQueues'],
	},
})

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
