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
	tx: ({ api }: { api: ApiPromise }, amount: string) => SubmittableExtrinsic<'promise'>
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
export const testCases: SwitchTestConfiguration[] = [
	{
		config: {
			desc: 'V4 LIVE',
			network: {
				relay: mainChains.polkadot.getConfig({}),
				parachains: [mainChains.assetHub.getConfig({}), mainChains.kilt.getConfig({})],
			},
			storage: {
				senderStorage: {
					...mainChains.assetHub.storage.assignNativeTokensToAccountsAsStorage([keysAlice.address]),
					...mainChains.assetHub.storage.assignForeignAssetToAccounts([
						[keysAlice.address, initialBalanceKILT],
					]),
				},
				receiverStorage: {
					...mainChains.kilt.storage.removeSwitchPair(),
				},
				relayStorage: {},
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
						section: 'messageQueue',
						method: 'Processed',
					},
				],
			},
		},
	},
]
