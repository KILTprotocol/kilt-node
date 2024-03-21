import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types'

export const options: SetupOption = {
	endpoint: 'wss://polkadot-rpc.dwellir.com',
	db: './db/polkadot.db.sqlite',
	port: 8000,
}

export const defaultStorage = {
	// give charlie some coins
	System: {
		Account: [
			[['4opXEdE6gvsx2Dsw3uisuP8reFCutip5WNZWkdtzFpHLHE8V'], { providers: 1, data: { free: 1000 * 1e12 } }],
		],
	},
	ParasDisputes: {
		// those can makes block building super slow
		$removePrefix: ['disputes'],
	},
	Dmp: {
		// clear existing dmp to avoid impact test result
		$removePrefix: ['downwardMessageQueues'],
	},
}

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
