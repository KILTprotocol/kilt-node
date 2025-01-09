import type { EventFilter } from '@acala-network/chopsticks-testing'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { keysAlice } from '../../../helper/utils.js'
import { mainChains } from '../../../network/index.js'
import { tx, query } from '../../../helper/api.js'
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
			desc: 'Switch V4 LIVE: KILT -> AH',
			network: {
				relay: mainChains.polkadot.getConfig({}),
				parachains: [mainChains.kilt.getConfig({}), mainChains.assetHub.getConfig({})],
			},
			storage: {
				receiverStorage: {
					// Assign some coins to create the account.
					...mainChains.assetHub.storage.assignNativeTokensToAccountsAsStorage([keysAlice.address]),
				},
				senderStorage: {
					// Assign some coins to create the account.
					...mainChains.kilt.storage.assignNativeTokensToAccounts([keysAlice.address]),
					...mainChains.kilt.storage.assignRelayTokensToAccounts([keysAlice.address]),
				},
				relayStorage: {},
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
				receiver: [{ section: 'foreignAssets', method: 'Transferred' }],
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
