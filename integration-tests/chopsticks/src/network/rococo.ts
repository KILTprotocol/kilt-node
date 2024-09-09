import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'
import { initialBalanceROC, toNumber } from '../utils.js'

/// Options used to create the HydraDx context
export const options: SetupOption = {
	endpoint: process.env.ROCOCO_WS || ['wss://rococo-rpc.polkadot.io'],
	db: './db/rococo.db.sqlite',
	port: toNumber(process.env.ROCOCO_PORT) || 8999,
}

/// Assigns the native tokens to an accounts
export function assignNativeTokensToAccounts(addr: string[], balance: bigint = initialBalanceROC) {
	return {
		System: {
			Account: addr.map((address) => [[address], { providers: 1, data: { free: balance } }]),
		},
	}
}

export function setSudoKey(sudo: string) {
	return {
		Sudo: {
			key: sudo,
		},
	}
}

export function removeDisputesAndMessageQueues() {
	return {
		ParasDisputes: {
			// those can makes block building super slow
			$removePrefix: ['disputes'],
		}
	}
}

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
