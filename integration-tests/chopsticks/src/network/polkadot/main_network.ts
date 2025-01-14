import type { SetupOption } from '@acala-network/chopsticks-testing'

import { initialBalanceDOT, toNumber } from '../../helper/utils.js'

/// Options used to create the HydraDx context
export const getSetupOptions = ({
	blockNumber = undefined,
	wasmOverride = undefined,
}: {
	blockNumber?: number
	wasmOverride?: string
}) => {
	const random = (Math.random() + 1).toString(36).substring(7)
	return {
		endpoint: process.env.POLKADOT_WS || [
			'wss://rpc.polkadot.io',
			'wss://polkadot-rpc.dwellir.com',
			'wss://rpc.ibp.network/polkadot',
		],
		db: `./db/polkadot_main_${random}.db.sqlite`,
		port: toNumber(process.env.POLKADOT_PORT),
		blockNumber,
		wasmOverride,
	} as SetupOption
}

export const storage = {
	/// Assigns the native tokens to an accounts
	assignNativeTokensToAccounts(addr: string[], balance: bigint = initialBalanceDOT) {
		return {
			System: {
				Account: addr.map((address) => [[address], { providers: 1, data: { free: balance } }]),
			},
		}
	},

	removeDisputesAndMessageQueues() {
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
	},
}
