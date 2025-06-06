import type { EventFilter } from '@acala-network/chopsticks-testing'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { DOT, initialBalanceKILT, keysAlice } from '../../../../helper/utils.js'
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
	tx: ({ api }: { api: ApiPromise }, submitter: string, amount: string | number) => SubmittableExtrinsic<'promise'>
	// events to check after the transaction
	events: Events
}

interface TestConfiguration {
	config: BasicConfig
	query: Query
	txContext: TxContext
	account: KeyringPair
}

export const testCases: TestConfiguration[] = [
	{
		config: {
			desc: 'V4 LIVE',
			network: {
				relay: { option: mainChains.polkadot.getConfig({}), storage: {}, setUpTx: [] },
				parachains: [
					{
						option: mainChains.assetHub.getConfig({}),
						storage: mainChains.assetHub.storage.assignNativeTokensToAccountsAsStorage([keysAlice.address]),
						setUpTx: [],
					},
					{
						option: mainChains.kilt.getConfig({}),

						storage: mainChains.kilt.storage.assignNativeTokensToAccounts(
							[keysAlice.address],
							initialBalanceKILT
						),
						setUpTx: [tx.switchPallet.pause()],
					},
				],
			},
		},
		account: keysAlice,
		query: {
			sender: query.balances,
			receiver: query.fungibles(mainChains.assetHub.chainInfo.nativeTokenLocation),
		},
		txContext: {
			balanceToTransfer: DOT,
			tx: tx.xcmPallet.limitedReserveTransferAssetsV4(
				mainChains.assetHub.chainInfo.nativeTokenLocation,
				tx.xcmPallet.parachainV4(1, mainChains.kilt.chainInfo.paraId)
			),
			events: {
				sender: [
					{
						section: 'polkadotXcm',
						method: 'Sent',
					},
				],
				receiver: [
					{
						section: 'polkadotXcm',
						method: 'AssetsTrapped',
					},
				],
			},
		},
	},
]
