import type { SetupOption } from '@acala-network/chopsticks-testing'

import { initialBalanceHDX, initialBalanceKILT, toNumber } from '../../helper/utils.js'
import { SetupConfigParameters } from '../types.js'

/// Options used to create the Hydration context
export const getSetupOptions = ({ blockNumber, wasmOverride }: SetupConfigParameters) => {
	const random = (Math.random() + 1).toString(36).substring(7)
	return {
		endpoint: process.env.HYDRATION_TEST_WS || ['wss://paseo-rpc.play.hydration.cloud'],
		db: `./db/hydration_test_${random}.db.sqlite`,
		port: toNumber(process.env.HYDRATION_TEST_PORT),
		blockNumber,
		wasmOverride,
	} as SetupOption
}

export const storage = {
	/// Sets the [TechnicalCommittee] and [Council] governance to the given accounts
	setGovernance(addr: string[]) {
		return {
			TechnicalCommittee: { Members: addr },
			Council: { Members: addr },
		}
	},
	/// Assigns the native tokens to an accounts
	assignNativeTokensToAccounts(addr: string[], balance: bigint = initialBalanceHDX) {
		return {
			System: {
				Account: addr.map((address) => [[address], { providers: 1, data: { free: balance } }]),
			},
		}
	},

	/// Assigns KILT tokens to accounts
	assignKiltTokensToAccounts(addr: string[], balance: bigint = initialBalanceKILT) {
		return {
			Tokens: {
				Accounts: addr.map((address) => [[address, parachainInfo.kiltTokenId], { free: balance }]),
			},
		}
	},
}

export const parachainInfo = {
	// Sibling Sovereign Account
	sovereignAccountOnSiblingChains: '5Eg2fntQqFi3EvFWAf71G66Ecjjah26bmFzoANAeHFgj9Lia',
	kiltTokenId: 28,
	paraId: 2034,
	omnipoolAccount: '7L53bUTBbfuj14UpdCNPwmgzzHSsrsTWBHX5pys32mVWM3C1',
}
