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
	sovereignAccount: SovereignAccount
}

// Test pairs for limited reserve transfers
export const testPairsSwitchFunds: SwitchTestConfiguration[] = [
	{
		config: {
			desc: 'Switch V4 LIVE: AssetHub -> Kilt',
			network: {
				relay: mainChains.polkadot.getConfig({}),
				parachains: [mainChains.assetHub.getConfig({}), mainChains.kilt.getConfig({})],
			},
			storage: {
				receiverStorage: mainChains.kilt.storage.assignNativeTokensToAccounts(
					[keysAlice.address],
					initialBalanceKILT
				),
				senderStorage: {
					// Assign some coins to create the account.
					...mainChains.assetHub.storage.assignNativeTokensToAccountsAsStorage([keysAlice.address]),
					// Assign the foreign asset to the account
					...mainChains.assetHub.storage.assignForeignAssetToAccounts([
						[keysAlice.address, initialBalanceKILT],
					]),
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
			tx: tx.xcmPallet.transferAssetsUsingTypeAndThenV4(
				tx.xcmPallet.parachainV4(1, mainChains.kilt.chainInfo.paraId),
				mainChains.assetHub.chainInfo.eKiltLocation,
				getXcmMessageV4ToSendEkilt(keysAlice.address)
			),
			events: {
				sender: [
					{ section: 'assetSwitchPool1', method: 'LocalToRemoteSwitchExecuted' },
					{ section: 'fungibles', method: 'Burned' },
				],

				receiver: [
					{ section: 'assetSwitchPool1', method: 'LocalToRemoteSwitchExecuted' },
					{ section: 'fungibles', method: 'Burned' },
				],
			},
			balanceToTransfer: BigInt(1e15),
		},
		sovereignAccount: {
			sender: mainChains.assetHub.chainInfo.sovereignAccountOnSiblingChains,
			receiver: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
		},
	},
] as const
