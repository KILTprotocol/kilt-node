import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'
import { initialBalanceDOT, toNumber } from '../utils.js'

/// Options used to create the HydraDx context
export const options: SetupOption = {
	endpoint: process.env.POLKADOT_WS || [
		'wss://rpc.polkadot.io',
		'wss://polkadot-rpc.dwellir.com',
		'wss://rpc.ibp.network/polkadot',
	],
	db: './db/polkadot.db.sqlite',
	port: toNumber(process.env.POLKADOT_PORT) || 9000,
}

/// Assigns the native tokens to an accounts
export function assignNativeTokensToAccounts(addr: string[], balance: bigint = initialBalanceDOT) {
	return {
		System: {
			Account: addr.map((address) => [[address], { providers: 1, data: { free: balance } }]),
		},
	}
}

export function removeDisputesAndMessageQueues() {
	return {
		ParasDisputes: {
			// those can makes block building super slow
			$removePrefix: ['disputes'],
		},
		Dmp: {
			// clear existing dmp to avoid impact test result
			$removePrefix: ['downwardMessageQueues'],
		},
	}
}

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
