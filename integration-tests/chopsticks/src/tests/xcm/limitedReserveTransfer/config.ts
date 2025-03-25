import { initialBalanceKILT, keysAlice, keysBob } from '../../../helper/utils.js'
import { mainChains } from '../../../network/index.js'
import { tx, query } from '../../../helper/api.js'

import type { ApiPromise } from '@polkadot/api'
import type { BasicConfig, BasicXcmTestConfiguration, BasicTxContext } from '../../types.js'
import type { SubmittableExtrinsic } from '@polkadot/api/types'

interface Config extends BasicConfig {
	// The received balance can be different in each block due to dynamic fee calculation.
	// Precision is used to compare the balances with a certain precision.
	precision: bigint
}

interface Query {
	sender: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
	receiver: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

interface TxContext extends BasicTxContext {
	balanceToTransfer: bigint
	tx: ({ api }: { api: ApiPromise }, submitter: string, amount: string | number) => SubmittableExtrinsic<'promise'>
}

/*
 * Configuration for the LimitedReserveTransfer test extends the BasicXcmTestConfiguration
 **/
interface LimitedReserveTestConfiguration extends BasicXcmTestConfiguration {
	config: Config
	query: Query
	txContext: TxContext
}

// Test pairs for limited reserve transfers
export const testPairsLimitedReserveTransfers: LimitedReserveTestConfiguration[] = [
	{
		config: {
			desc: 'Kilt -> Hydration live V2',
			precision: 96n,
			network: {
				// sender, receiver
				parachains: [
					{
						option: mainChains.kilt.getConfig({}),
						storage: mainChains.kilt.storage.assignNativeTokensToAccounts(
							[keysAlice.address],
							initialBalanceKILT
						),
						setUpTx: [],
					},
					{ option: mainChains.hydration.getConfig({}), setUpTx: [], storage: {} },
				],
				relay: { option: mainChains.polkadot.getConfig({}), storage: {}, setUpTx: [] },
			},
		},

		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.tokens(mainChains.hydration.chainInfo.kiltTokenId),
		},
		txContext: {
			tx: tx.xcmPallet.limitedReserveTransferAssetsV2(
				mainChains.kilt.chainInfo.KILT,
				tx.xcmPallet.parachainV2(1, mainChains.hydration.chainInfo.paraId)
			),
			pallets: {
				sender: [],
				receiver: [],
			},
			balanceToTransfer: BigInt(1e15),
		},

		sovereignAccount: {
			sender: mainChains.hydration.chainInfo.sovereignAccountOnSiblingChains,
			receiver: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
		},
	},

	{
		config: {
			desc: 'Kilt -> Hydration live V3',
			precision: 96n,
			network: {
				// sender, receiver
				parachains: [
					{
						option: mainChains.kilt.getConfig({}),
						storage: mainChains.kilt.storage.assignNativeTokensToAccounts(
							[keysAlice.address],
							initialBalanceKILT
						),
						setUpTx: [],
					},
					{ option: mainChains.hydration.getConfig({}), setUpTx: [], storage: {} },
				],
				relay: { option: mainChains.polkadot.getConfig({}), storage: {}, setUpTx: [] },
			},
		},

		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.tokens(mainChains.hydration.chainInfo.kiltTokenId),
		},
		txContext: {
			tx: tx.xcmPallet.limitedReserveTransferAssetsV3(
				mainChains.kilt.chainInfo.KILT,
				tx.xcmPallet.parachainV3(1, mainChains.hydration.chainInfo.paraId)
			),
			pallets: {
				sender: [],
				receiver: [],
			},
			balanceToTransfer: BigInt(1e15),
		},

		sovereignAccount: {
			sender: mainChains.hydration.chainInfo.sovereignAccountOnSiblingChains,
			receiver: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
		},
	},

	{
		config: {
			desc: 'Kilt DEV -> Hydration live v3',
			precision: 96n,
			network: {
				// sender, receiver
				parachains: [
					{
						option: mainChains.kilt.getConfig({}),
						storage: mainChains.kilt.storage.assignNativeTokensToAccounts(
							[keysAlice.address],
							initialBalanceKILT
						),
						setUpTx: [],
					},
					{ option: mainChains.hydration.getConfig({}), setUpTx: [], storage: {} },
				],
				relay: { option: mainChains.polkadot.getConfig({}), storage: {}, setUpTx: [] },
			},
		},

		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.tokens(mainChains.hydration.chainInfo.kiltTokenId),
		},
		txContext: {
			tx: tx.xcmPallet.limitedReserveTransferAssetsV3(
				mainChains.kilt.chainInfo.KILT,
				tx.xcmPallet.parachainV3(1, mainChains.hydration.chainInfo.paraId)
			),
			pallets: {
				sender: [],
				receiver: [],
			},
			balanceToTransfer: BigInt(1e15),
		},

		sovereignAccount: {
			sender: mainChains.hydration.chainInfo.sovereignAccountOnSiblingChains,
			receiver: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
		},
	},

	{
		config: {
			desc: 'Kilt DEV -> Hydration live v2',
			precision: 96n,
			network: {
				// sender, receiver
				parachains: [
					{
						option: mainChains.kilt.getConfig({}),
						storage: mainChains.kilt.storage.assignNativeTokensToAccounts(
							[keysAlice.address],
							initialBalanceKILT
						),
						setUpTx: [],
					},
					{ option: mainChains.hydration.getConfig({}), setUpTx: [], storage: {} },
				],
				relay: { option: mainChains.polkadot.getConfig({}), storage: {}, setUpTx: [] },
			},
		},

		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.tokens(mainChains.hydration.chainInfo.kiltTokenId),
		},
		txContext: {
			tx: tx.xcmPallet.limitedReserveTransferAssetsV2(
				mainChains.kilt.chainInfo.KILT,
				tx.xcmPallet.parachainV2(1, mainChains.hydration.chainInfo.paraId)
			),
			pallets: {
				sender: [],
				receiver: [],
			},
			balanceToTransfer: BigInt(1e15),
		},

		sovereignAccount: {
			sender: mainChains.hydration.chainInfo.sovereignAccountOnSiblingChains,
			receiver: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
		},
	},
] as const
