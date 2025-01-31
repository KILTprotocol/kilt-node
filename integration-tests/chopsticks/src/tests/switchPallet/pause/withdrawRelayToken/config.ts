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
				relay: { option: mainChains.polkadot.getConfig({}), setUpTx: [], storage: {} },
				parachains: [
					{
						option: mainChains.kilt.getConfig({}),
						storage: {
							...mainChains.kilt.storage.assignNativeTokensToAccounts(
								[keysAlice.address],
								initialBalanceKILT
							),
							...mainChains.kilt.storage.assignRelayTokensToAccounts([keysAlice.address]),
						},
						setUpTx: [tx.switchPallet.pause()],
					},

					{ option: mainChains.assetHub.getConfig({}), storage: {}, setUpTx: [] },
				],
			},
		},
		account: keysAlice,
		query: {
			sender: query.fungibles(mainChains.assetHub.chainInfo.nativeTokenLocation),
			receiver: query.balances,
		},
		txContext: {
			balanceToTransfer: DOT,
			tx: tx.xcmPallet.transferAssetsV4(
				tx.xcmPallet.parachainV4(1, mainChains.assetHub.chainInfo.paraId),
				mainChains.assetHub.chainInfo.nativeTokenLocation
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
						section: 'balances',
						method: 'Burned',
					},
					{
						section: 'balances',
						method: 'Minted',
					},
				],
			},
		},
	},
]
