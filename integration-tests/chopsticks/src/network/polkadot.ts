import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'
import { toNumber } from './utils.js'

export const options: SetupOption = {
	endpoint: process.env.POLKADOT_WS || [
		'wss://rpc.polkadot.io',
		'wss://polkadot-rpc.dwellir.com',
		'wss://rpc.ibp.network/polkadot',
	],
	db: './db/polkadot.db.sqlite',
	port: toNumber(process.env.POLKADOT_PORT) || 8000,
}

export const defaultStorage = (addr: string) => ({
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
