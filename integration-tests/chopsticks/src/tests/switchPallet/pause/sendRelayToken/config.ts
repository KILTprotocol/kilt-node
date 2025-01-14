/* eslint-disable @typescript-eslint/no-unused-vars */
import type { EventFilter } from '@acala-network/chopsticks-testing'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { initialBalanceKILT, keysAlice, keysBob } from '../../../../helper/utils.js'
import { mainChains } from '../../../../network/index.js'
import { tx, query } from '../../../../helper/api.js'
import type { BasicConfig } from '../../../types.js'

interface Query {
	// Query options on the native chain
	sender: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
	// Query options on the foreign chain
	receiver: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

/**
 * All possible events to check after the transaction.
 */
interface Events {
	// events to check after the transaction on the native chain
	sender: EventFilter[]
	// events to check after the transaction on the foreign chain
	receiver: EventFilter[]
}

/**
 * Context for the transaction to switch funds between chains.
 */
interface TxContext {
	// amount of funds to transfer
	balanceToTransfer: bigint
	// transactions to execute
	tx: ({ api }: { api: ApiPromise }, submitter: string, amount: string | number) => SubmittableExtrinsic<'promise'>
	// events to check after the transaction
	events: Events
}

/*
 * Configuration for Swtichting coins.
 **/
interface SwitchTestConfiguration {
	config: BasicConfig
	query: Query
	txContext: TxContext
	account: KeyringPair
}

// Test pairs for limited reserve transfers
export const testPairsSwitchFunds: SwitchTestConfiguration[] = [
	{
		config: {
			desc: 'V4 LIVE: AssetHub -> KILT',
			network: {
				relay: mainChains.polkadot.getConfig({}),
				parachains: [mainChains.assetHub.getConfig({}), mainChains.kilt.getConfig({})],
			},
			storage: {
				senderStorage: mainChains.assetHub.storage.assignNativeTokensToAccountsAsStorage([keysAlice.address]),
				receiverStorage: {
					...mainChains.kilt.storage.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
					...mainChains.kilt.storage.pauseSwitch(),
				},

				relayStorage: {},
			},
		},
		account: keysAlice,
		query: {
			sender: query.balances,
			receiver: query.fungibles(mainChains.assetHub.chainInfo.nativeTokenLocation),
		},
		txContext: {
			balanceToTransfer: BigInt('10000000000'),
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
