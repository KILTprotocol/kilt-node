import type { EventFilter } from '@acala-network/chopsticks-testing'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { keysAlice, KILT } from '../../../../helper/utils.js'
import { mainChains } from '../../../../network/index.js'
import { tx, query } from '../../../../helper/api.js'
import type { BasicConfig } from '../../../types.js'

interface Query {
	sender: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>

	receiver: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

interface Events {
	// events to check after the transaction on the native chain
	sender: EventFilter[]
	// events to check after the transaction on the foreign chain
	receiver: EventFilter[]
}

interface TxContext {
	// amount of funds to transfer
	balanceToTransfer: bigint
	// transactions to execute
	tx: ({ api }: { api: ApiPromise }, acc: string, amount: string) => SubmittableExtrinsic<'promise'>
	// events to check after the transaction
	events: Events
}

interface TestConfiguration {
	config: BasicConfig
	query: Query
	txContext: TxContext
	account: KeyringPair
	sovereignAccount: string
}

export const testCases: TestConfiguration[] = [
	{
		config: {
			desc: 'V4 LIVE',
			network: {
				relay: { option: mainChains.polkadot.getConfig({}), storage: {}, setUpTx: [] },
				parachains: [
					{
						option: mainChains.kilt.getConfig({}),
						setUpTx: [],
						storage: {
							...mainChains.kilt.storage.assignNativeTokensToAccounts([keysAlice.address]),
							...mainChains.kilt.storage.assignRelayTokensToAccounts([keysAlice.address]),
						},
					},
					{
						option: mainChains.assetHub.getConfig({}),
						setUpTx: [],
						storage: {},
					},
				],
			},
		},
		account: keysAlice,
		query: {
			sender: query.balances,
			receiver: query.foreignAssets(mainChains.assetHub.chainInfo.eKiltLocation),
		},
		txContext: {
			balanceToTransfer: KILT,
			tx: tx.switchPallet.switchV4(),
			events: {
				sender: ['assetSwitchPool1'],
				receiver: [],
			},
		},
		sovereignAccount: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
	},
]
