import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { BasicConfig } from '../types.js'
import type { KeyringPair } from '@polkadot/keyring/types'
import { mainChains } from '../../network/index.js'
import { keysAlice } from '../../helper/utils.js'
import { tx } from '../../helper/api.js'

interface TxContext {
	// transactions to execute
	tx: ({ api }: { api: ApiPromise }, submitter: string, amount: string) => SubmittableExtrinsic<'promise'>
}

interface TestConfiguration {
	config: BasicConfig
	txContext: TxContext
	account: KeyringPair
}

export const testCases: TestConfiguration[] = [
	{
		config: {
			desc: 'TEMPLATE',
			network: {
				parachains: [],
				relay: mainChains.polkadot.getConfig({}),
			},
			storage: {
				senderStorage: {},
				receiverStorage: {},
				relayStorage: {},
			},
		},
		account: keysAlice,
		txContext: {
			tx: tx.balances.transferAllowDeath,
		},
	},
]
