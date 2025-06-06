import type { EventFilter } from '@acala-network/chopsticks-testing'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { keysAlice } from '../../../helper/utils.js'
import { mainChains } from '../../../network/index.js'
import { tx, query } from '../../../helper/api.js'
import type { BasicConfig, SovereignAccount } from '../../types.js'

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
	tx: (
		{
			api,
		}: {
			api: ApiPromise
		},
		acc: string,
		amount: string
	) => SubmittableExtrinsic<'promise'>
	// events to check after the transaction
	events: Events
}

interface TestConfiguration {
	config: BasicConfig
	query: Query
	txContext: TxContext
	account: KeyringPair
	sovereignAccount: SovereignAccount
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
							// Assign some coins to create the account.
							...mainChains.kilt.storage.assignNativeTokensToAccounts([keysAlice.address]),
							...mainChains.kilt.storage.assignRelayTokensToAccounts([keysAlice.address]),
						},
					},
					{
						option: mainChains.assetHub.getConfig({}),
						setUpTx: [],
						storage: mainChains.assetHub.storage.assignNativeTokensToAccountsAsStorage([keysAlice.address]),
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
			tx: tx.switchPallet.switchV4(),
			events: {
				receiver: ['foreignAssets'],
				sender: [{ section: 'assetSwitchPool1', method: 'LocalToRemoteSwitchExecuted' }],
			},
			balanceToTransfer: BigInt(1e15),
		},
		sovereignAccount: {
			sender: mainChains.assetHub.chainInfo.sovereignAccountOnSiblingChains,
			receiver: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
		},
	},
] as const
