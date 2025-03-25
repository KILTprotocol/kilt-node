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
	sender: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
	receiver: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

interface Events {
	sender: EventFilter[]
	receiver: EventFilter[]
}

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
				// For this test, the relay chain is not important. By using the test chain, we can
				// dispatch calls with sudo rights. TODO: Scheduling the calls is somehow not possible.
				relay: {
					option: testChains.polkadot.getConfig({}),
					setUpTx: [],
					storage: {
						...testChains.polkadot.storage.assignNativeTokensToAccounts([keysAlice.address]),
						...testChains.polkadot.storage.assignSudoKey(keysAlice.address),
					},
				},
				parachains: [
					{
						option: mainChains.assetHub.getConfig({}),
						setUpTx: [],
						storage: {
							...mainChains.assetHub.storage.assignNativeTokensToAccountsAsStorage([keysAlice.address]),
							...mainChains.assetHub.storage.assignForeignAssetToAccounts([
								[keysAlice.address, initialBalanceKILT],
							]),
						},
					},
					{
						option: mainChains.kilt.getConfig({}),
						setUpTx: [tx.switchPallet.pause()],
						storage: mainChains.kilt.storage.assignNativeTokensToAccounts([
							mainChains.assetHub.chainInfo.sovereignAccountOnSiblingChains,
						]),
					},
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
		sovereignAccount: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
	},
]
