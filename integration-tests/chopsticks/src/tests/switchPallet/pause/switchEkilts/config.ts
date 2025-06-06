import type { EventFilter } from '@acala-network/chopsticks-testing'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { initialBalanceKILT, keysAlice, KILT } from '../../../../helper/utils.js'
import { mainChains } from '../../../../network/index.js'
import { tx, query } from '../../../../helper/api.js'
import type { BasicConfig } from '../../../types.js'
import { getXcmMessageV4ToSendEkilt } from '../../index.js'

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
	tx: ({ api }: { api: ApiPromise }, amount: string) => SubmittableExtrinsic<'promise'>
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
				relay: { option: mainChains.polkadot.getConfig({}), setUpTx: [], storage: {} },
				parachains: [
					{
						option: mainChains.assetHub.getConfig({}),
						storage: {
							...mainChains.assetHub.storage.assignNativeTokensToAccountsAsStorage([keysAlice.address]),
							...mainChains.assetHub.storage.assignForeignAssetToAccounts([
								[keysAlice.address, initialBalanceKILT],
							]),
						},
						setUpTx: [],
					},
					{ option: mainChains.kilt.getConfig({}), storage: {}, setUpTx: [tx.switchPallet.pause()] },
				],
			},
		},
		account: keysAlice,
		query: {
			sender: query.foreignAssets(mainChains.assetHub.chainInfo.eKiltLocation),
			receiver: query.balances,
		},
		txContext: {
			balanceToTransfer: KILT,
			tx: tx.xcmPallet.transferAssetsUsingTypeAndThenV4(
				tx.xcmPallet.parachainV4(1, mainChains.kilt.chainInfo.paraId),
				mainChains.assetHub.chainInfo.eKiltLocation,
				getXcmMessageV4ToSendEkilt(keysAlice.address)
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
		sovereignAccount: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
	},
]
