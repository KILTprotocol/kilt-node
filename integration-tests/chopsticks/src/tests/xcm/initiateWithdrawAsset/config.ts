import { initialBalanceHDX, initialBalanceKILT, keysAlice, keysBob } from '../../../helper/utils.js'
import { tx, query } from '../../../helper/api.js'
import { mainChains } from '../../../network/index.js'

import type { ApiPromise } from '@polkadot/api'
import type { BasicConfig, BasicXcmTestConfiguration, BasisTxContext } from '../../types.js'
import type { SubmittableExtrinsic } from '@polkadot/api/types'

interface Config extends BasicConfig {
	precision: bigint
}

interface Query {
	sender: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
	receiver: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

interface TxContext extends BasisTxContext {
	balanceToTransfer: bigint
	tx: ({ api }: { api: ApiPromise }, submitter: string, amount: string | number) => SubmittableExtrinsic<'promise'>
}

/*
 * Configuration for the WithdrawAssets test extends the BasicXcmTestConfiguration
 **/
interface WithdrawAssetTestConfiguration extends BasicXcmTestConfiguration {
	config: Config
	query: Query
	txContext: TxContext
}
// Test pairs for WithdrawAssets
export const testPairsWithdrawAssets: WithdrawAssetTestConfiguration[] = [
	{
		config: {
			desc: 'HydraDx -> KILT live',
			precision: BigInt(96),
			network: {
				relay: mainChains.polkadot.getConfig({}),
				// sender, receiver
				parachains: [mainChains.hydration.getConfig({}), mainChains.kilt.getConfig({})],
			},
			storage: {
				senderStorage: {
					...mainChains.hydration.storage.assignKiltTokensToAccounts([keysAlice.address], initialBalanceKILT),
					...mainChains.hydration.storage.assignNativeTokensToAccounts(
						[keysAlice.address],
						initialBalanceHDX
					),
				},
				receiverStorage: {},
				relayStorage: {},
			},
		},

		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.tokens(mainChains.hydration.chainInfo.kiltTokenId),
			receiver: query.balances,
		},
		txContext: {
			tx: tx.xtokens.transfer(
				mainChains.hydration.chainInfo.kiltTokenId,
				tx.xtokens.parachainV3(mainChains.kilt.chainInfo.paraId)
			),
			pallets: {
				sender: ['xcmpQueue'],
				receiver: ['xcmpQueue', { section: 'system', method: 'NewAccount' }],
			},
			balanceToTransfer: BigInt(1e15),
		},

		sovereignAccount: {
			sender: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
			receiver: mainChains.hydration.chainInfo.sovereignAccountOnSiblingChains,
		},
	},

	// {
	// 	config: {
	// 		desc: 'HydraDx -> KILT DEV',
	// 		precision: BigInt(96),
	// 		network: {
	// 			sender: PolkadotChainConfigs.all.hydraDx.getConfig({}),
	// 			receiver: PolkadotChainConfigs.all.spiritnet.getConfig(PolkadotChainConfigs.all.spiritnet.parameters),
	// 			relay: PolkadotChainConfigs.all.polkadot.getConfig({}),
	// 		},
	// 		storage: {
	// 			senderStorage: {
	// 				...HydraDxConfig.assignKiltTokensToAccounts([keysAlice.address], initialBalanceKILT),
	// 				...HydraDxConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceHDX),
	// 			},
	// 			receiverStorage: {},
	// 			relayStorage: {},
	// 		},
	// 	},

	// 	accounts: {
	// 		senderAccount: keysAlice,
	// 		receiverAccount: keysBob,
	// 	},
	// 	query: {
	// 		sender: query.tokens(HydraDxConfig.kiltTokenId),
	// 		receiver: query.balances,
	// 	},
	// 	txContext: {
	// 		tx: tx.xtokens.transfer(HydraDxConfig.kiltTokenId, tx.xtokens.parachainV3(SpiritnetConfig.paraId)),
	// 		pallets: {
	// 			sender: ['xcmpQueue'],
	// 			receiver: ['xcmpQueue', { section: 'system', method: 'NewAccount' }],
	// 		},
	// 		balanceToTransfer: BigInt(1e15),
	// 	},

	// 	sovereignAccount: {
	// 		sender: SpiritnetConfig.siblingSovereignAccount,
	// 		receiver: HydraDxConfig.siblingSovereignAccount,
	// 	},
	// },

	{
		config: {
			desc: 'HydraDx -> KILT at Block',
			precision: BigInt(99),
			network: {
				relay: mainChains.polkadot.getConfig({
					blockNumber: mainChains.polkadot.parameters.blockNumber,
				}),
				// sender, receiver
				parachains: [mainChains.hydration.getConfig({}), mainChains.kilt.getConfig({})],
			},
			storage: {
				senderStorage: {
					...mainChains.hydration.storage.assignKiltTokensToAccounts([keysAlice.address], initialBalanceKILT),
					...mainChains.hydration.storage.assignNativeTokensToAccounts(
						[keysAlice.address],
						initialBalanceHDX
					),
				},
				receiverStorage: {},
				relayStorage: {},
			},
		},

		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.tokens(mainChains.hydration.chainInfo.kiltTokenId),
			receiver: query.balances,
		},
		txContext: {
			tx: tx.xtokens.transfer(
				mainChains.hydration.chainInfo.kiltTokenId,
				tx.xtokens.parachainV3(mainChains.kilt.chainInfo.paraId)
			),
			pallets: {
				sender: ['xcmpQueue', { section: 'currencies', method: 'Withdrawn' }],
				receiver: [
					'xcmpQueue',
					{ section: 'balances', method: 'Withdraw' },
					{ section: 'system', method: 'NewAccount' },
				],
			},
			balanceToTransfer: BigInt(1e15),
		},

		sovereignAccount: {
			sender: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
			receiver: mainChains.hydration.chainInfo.sovereignAccountOnSiblingChains,
		},
	},
] as const
