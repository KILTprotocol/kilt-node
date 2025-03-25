import type { EventFilter } from '@acala-network/chopsticks-testing'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { initialBalanceKILT, keysAlice } from '../../../helper/utils.js'
import { mainChains } from '../../../network/index.js'
import { tx, query } from '../../../helper/api.js'
import { getXcmMessageV4ToSendEkilt } from '../index.js'
import type { BasicConfig, SovereignAccount } from '../../types.js'

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
				relay: { option: mainChains.polkadot.getConfig({}), setUpTx: [], storage: {} },
				parachains: [
					{
						option: mainChains.assetHub.getConfig({}),
						storage: {
							// Assign some coins to create the account.
							...mainChains.assetHub.storage.assignNativeTokensToAccountsAsStorage([keysAlice.address]),
							// Assign the foreign asset to the account
							...mainChains.assetHub.storage.assignForeignAssetToAccounts([
								[keysAlice.address, initialBalanceKILT],
							]),
						},
						setUpTx: [],
					},
					{
						option: mainChains.kilt.getConfig({}),
						storage: {},
						setUpTx: [],
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
			tx: tx.xcmPallet.transferAssetsUsingTypeAndThenV4(
				tx.xcmPallet.parachainV4(1, mainChains.kilt.chainInfo.paraId),
				mainChains.assetHub.chainInfo.eKiltLocation,
				getXcmMessageV4ToSendEkilt(keysAlice.address)
			),
			events: {
				sender: [
					{ section: 'foreignAssets', method: 'Transferred' },
					{ section: 'polkadotXcm', method: 'Sent' },
				],

				receiver: [{ section: 'assetSwitchPool1', method: 'RemoteToLocalSwitchExecuted' }],
			},
			balanceToTransfer: BigInt(1e15),
		},
		sovereignAccount: {
			sender: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
			receiver: mainChains.assetHub.chainInfo.sovereignAccountOnSiblingChains,
		},
	},
] as const
