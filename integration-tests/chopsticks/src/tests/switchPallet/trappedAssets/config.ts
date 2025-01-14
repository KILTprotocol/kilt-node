import type { EventFilter } from '@acala-network/chopsticks-testing'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { initialBalanceKILT, keysAlice, KILT } from '../../../helper/utils.js'
import { mainChains, testChains } from '../../../network/index.js'
import { tx, query } from '../../../helper/api.js'
import type { BasicConfig } from '../../types.js'
import { getXcmV4ReclaimMessage, getXcmMessageV4ToSendEkilt } from '../index.js'

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
	tx: ({ api }: { api: ApiPromise }, amount: string) => SubmittableExtrinsic<'promise'>
	// events to check after the transaction
	events: Events
	getXcmMessage: (amount: string, receiver: string) => object
	reclaimTx: ({ api }: { api: ApiPromise }, xcmMessage: object) => SubmittableExtrinsic<'promise'>
	// the relative location of the sender from the relay chain
	senderLocation: object
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

export const testCases: SwitchTestConfiguration[] = [
	{
		config: {
			desc: 'V4 LIVE',
			network: {
				// For this test, the relay chain is not important. By using the test chain, we can
				// dispatch calls with sudo rights. TODO: Scheduling the calls is somehow not possible.
				relay: testChains.polkadot.getConfig({}),
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
					...mainChains.kilt.storage.pauseSwitch(),
					...mainChains.kilt.storage.assignNativeTokensToAccounts([
						mainChains.assetHub.chainInfo.sovereignAccountOnSiblingChains,
					]),
				},
				relayStorage: {
					...testChains.polkadot.storage.assignNativeTokensToAccounts([keysAlice.address]),
					...testChains.polkadot.storage.assignSudoKey(keysAlice.address),
				},
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
			getXcmMessage: getXcmV4ReclaimMessage(mainChains.assetHub.chainInfo.eKiltLocation),
			reclaimTx: tx.xcmPallet.send(tx.xcmPallet.parachainV4(1, mainChains.kilt.chainInfo.paraId)),
			senderLocation: {
				parents: 0,
				interior: {
					X1: { Parachain: mainChains.assetHub.chainInfo.paraId },
				},
			},
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
						method: 'AssetsClaimed',
					},
					{
						section: 'assetSwitchPool1',
						method: 'RemoteToLocalSwitchExecuted',
					},
				],
			},
		},
	},
]
