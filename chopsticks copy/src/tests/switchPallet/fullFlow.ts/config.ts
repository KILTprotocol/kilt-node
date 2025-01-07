import type { EventFilter } from '@acala-network/chopsticks-testing'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import * as PolkadotChainConfigs from '../../../network/index.js'
import { initialBalanceKILT, keysAlice, keysBob } from '../../../helper/utils.js'
import * as SpiritnetConfig from '../../../network/spiritnet.js'
import * as AssetHubContext from '../../../network/assethub.js'
import { tx, query } from '../../../helper/api.js'
import { getXcmMessageV4ToSendEkilt } from '../index.js'
import type { BasicConfig } from '../../types.js'

interface QueryFunds {
	// Query the native asset of the chain
	nativeFunds: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
	// Query the foreign asset of the chain
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	foreignFunds: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

interface Query {
	// Query options on the native chain
	native: QueryFunds
	// Query options on the foreign chain
	foreign: QueryFunds
}

/**
 * All possible transactions to switch funds between chains.
 */
interface Transactions {
	native: {
		// tx to send the funds from the native chain to the foreign chain
		transfer: ({ api }: { api: ApiPromise }, submitter: string, amount: string) => SubmittableExtrinsic<'promise'>
		// tx to withdraw the funds from the foreign chain to the native chain
		withdraw: ({ api }: { api: ApiPromise }, amount: string) => SubmittableExtrinsic<'promise'>
	}
	foreign: {
		// tx to send the funds from the foreign chain to the native chain
		transfer: (
			{ api }: { api: ApiPromise },
			beneficiary: string,
			amount: string | number
		) => SubmittableExtrinsic<'promise'>
		// tx to withdraw the funds from the native chain to the foreign chain
		withdraw: (
			{ api }: { api: ApiPromise },
			submitter: string,
			amount: string | number
		) => SubmittableExtrinsic<'promise'>
	}
}

/**
 * All possible events to check after the transaction.
 */
interface Events {
	// events to check after the transaction on the native chain
	native: {
		// events after transfering the native funds to the foreign chain
		transfer: EventFilter[]
		// events after transfering the foreign funds from the native chain to the foreign chain
		withdraw: EventFilter[]
		// events after receiving the native funds from the foreign chain
		receive: {
			// events after receiving the native funds from the foreign chain
			native: EventFilter[]
			// events after receiving the foreign funds from the foreign chain
			foreign: EventFilter[]
		}
	}
	// events to check after the transaction on the foreign chain
	foreign: {
		// events after transfering the foreign funds to the native chain
		transfer: EventFilter[]
		// events after transfering the native funds from the foreign chain to the native chain
		withdraw: EventFilter[]
		receive: {
			// events after receiving the native funds from the native chain on the foreign chain
			native: EventFilter[]
			// events after receiving the foreign funds from the native chain on the foreign chain
			foreign: EventFilter[]
		}
	}
}

/**
 * Context for the transaction to switch funds between chains.
 */
interface TxContext {
	// amount of funds to transfer
	balanceToTransfer: {
		// amount of native currency to transfer
		native: bigint
		// amount of foreign currency to transfer
		foreign: bigint
	}
	// transactions to execute
	tx: Transactions
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
	sovereignAccount: string
}

// Test pairs for limited reserve transfers
export const testPairsSwitchFunds: SwitchTestConfiguration[] = [
	{
		config: {
			desc: 'Switch V4 LIVE: Kilt -> AssetHub -> Kilt',
			network: {
				sender: PolkadotChainConfigs.all.spiritnet.getConfig({}),
				receiver: PolkadotChainConfigs.all.assetHub.getConfig({}),
				relay: PolkadotChainConfigs.all.polkadot.getConfig({}),
			},
			storage: {
				senderStorage: SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
				receiverStorage: {
					// Assign some coins to create the account.
					...AssetHubContext.assignDotTokensToAccountsAsStorage([keysAlice.address]),
					// Create the eKilts.
					...AssetHubContext.createForeignAsset(keysBob.address),
				},
				relayStorage: {},
			},
		},

		account: keysAlice,
		query: {
			native: { nativeFunds: query.balances, foreignFunds: query.fungibles(AssetHubContext.nativeTokenLocation) },
			foreign: {
				nativeFunds: query.balances,
				foreignFunds: query.foreignAssets(AssetHubContext.eKiltLocation),
			},
		},
		txContext: {
			tx: {
				native: {
					transfer: tx.switchPallet.switchV4(),
					withdraw: tx.xcmPallet.transferAssetsUsingTypeAndThenV4(
						tx.xcmPallet.parachainV4(1, SpiritnetConfig.paraId),
						AssetHubContext.eKiltLocation,
						getXcmMessageV4ToSendEkilt(keysAlice.address)
					),
				},
				foreign: {
					transfer: tx.xcmPallet.limitedReserveTransferAssetsV4(
						AssetHubContext.nativeTokenLocation,
						tx.xcmPallet.parachainV4(1, SpiritnetConfig.paraId)
					),
					withdraw: tx.xcmPallet.transferAssetsV4(
						tx.xcmPallet.parachainV4(1, AssetHubContext.paraId),
						AssetHubContext.nativeTokenLocation
					),
				},
			},
			events: {
				native: {
					transfer: [
						{ section: 'assetSwitchPool1', method: 'LocalToRemoteSwitchExecuted' },
						{ section: 'fungibles', method: 'Burned' },
					],

					receive: {
						native: [{ section: 'assetSwitchPool1', method: 'RemoteToLocalSwitchExecuted' }],
						foreign: [
							{ section: 'fungibles', method: 'Issued' },
							{ section: 'messageQueue', method: 'Processed' },
						],
					},
					withdraw: [{ section: 'polkadotXcm', method: 'Sent' }, 'fungibles'],
				},

				foreign: {
					transfer: [{ section: 'polkadotXcm', method: 'Sent' }],
					receive: {
						foreign: ['foreignAssets', { section: 'messageQueue', method: 'Processed' }],
						native: [
							{ section: 'messageQueue', method: 'Processed' },
							{ section: 'balances', method: 'burned' },
						],
					},
					withdraw: [{ section: 'polkadotXcm', method: 'Sent' }],
				},
			},
			balanceToTransfer: {
				native: BigInt(1e15),
				foreign: BigInt(1e10),
			},
		},
		sovereignAccount: SpiritnetConfig.sovereignAccountOnSiblingChains,
	},
] as const
