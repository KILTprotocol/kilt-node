import type { EventFilter } from '@acala-network/chopsticks-testing'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { initialBalanceKILT, keysAlice, keysBob } from '../../../helper/utils.js'
import { mainChains } from '../../../network/index.js'
import { tx, query } from '../../../helper/api.js'
import { getXcmMessageV4ToSendEkilt } from '../index.js'
import type { BasicConfig } from '../../types.js'

interface QueryFunds {
	nativeFunds: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	foreignFunds: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

interface Query {
	native: QueryFunds
	foreign: QueryFunds
}

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

interface Events {
	// events to check after the transaction on the native chain
	native: {
		// events after transferring the native funds to the foreign chain
		transfer: EventFilter[]
		// events after transferring the foreign funds from the native chain to the foreign chain
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
		// events after transferring the foreign funds to the native chain
		transfer: EventFilter[]
		// events after transferring the native funds from the foreign chain to the native chain
		withdraw: EventFilter[]
		receive: {
			// events after receiving the native funds from the native chain on the foreign chain
			native: EventFilter[]
			// events after receiving the foreign funds from the native chain on the foreign chain
			foreign: EventFilter[]
		}
	}
}

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
				relay: mainChains.polkadot.getConfig({}),
				parachains: [mainChains.kilt.getConfig({}), mainChains.assetHub.getConfig({})],
			},
			storage: {
				senderStorage: mainChains.kilt.storage.assignNativeTokensToAccounts(
					[keysAlice.address],
					initialBalanceKILT
				),
				receiverStorage: {
					// Assign some coins to create the account.
					...mainChains.assetHub.storage.assignNativeTokensToAccountsAsStorage([keysAlice.address]),
					// Create the eKilts.
					...mainChains.assetHub.storage.createForeignAsset(keysBob.address),
				},
				relayStorage: {},
			},
		},
		account: keysAlice,
		query: {
			native: {
				nativeFunds: query.balances,
				foreignFunds: query.fungibles(mainChains.assetHub.chainInfo.nativeTokenLocation),
			},
			foreign: {
				nativeFunds: query.balances,
				foreignFunds: query.foreignAssets(mainChains.assetHub.chainInfo.eKiltLocation),
			},
		},
		txContext: {
			tx: {
				native: {
					transfer: tx.switchPallet.switchV4(),
					withdraw: tx.xcmPallet.transferAssetsUsingTypeAndThenV4(
						tx.xcmPallet.parachainV4(1, mainChains.kilt.chainInfo.paraId),
						mainChains.assetHub.chainInfo.eKiltLocation,
						getXcmMessageV4ToSendEkilt(keysAlice.address)
					),
				},
				foreign: {
					transfer: tx.xcmPallet.limitedReserveTransferAssetsV4(
						mainChains.assetHub.chainInfo.nativeTokenLocation,
						tx.xcmPallet.parachainV4(1, mainChains.kilt.chainInfo.paraId)
					),
					withdraw: tx.xcmPallet.transferAssetsV4(
						tx.xcmPallet.parachainV4(1, mainChains.assetHub.chainInfo.paraId),
						mainChains.assetHub.chainInfo.nativeTokenLocation
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
						foreign: [{ section: 'fungibles', method: 'Issued' }],
					},
					withdraw: [{ section: 'polkadotXcm', method: 'Sent' }, 'fungibles'],
				},

				foreign: {
					transfer: [{ section: 'polkadotXcm', method: 'Sent' }],
					receive: {
						foreign: ['foreignAssets', { section: 'messageQueue', method: 'Processed' }],
						native: [
							{ section: 'balances', method: 'Burned' },
							{ section: 'balances', method: 'Minted' },
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
		sovereignAccount: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
	},
] as const
