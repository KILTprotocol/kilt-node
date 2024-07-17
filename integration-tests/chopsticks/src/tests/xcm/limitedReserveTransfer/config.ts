import * as PolkadotChainConfigs from '../../../network/index.js'
import { initialBalanceKILT, keysAlice, keysBob } from '../../../helper/utils.js'
import * as SpiritnetConfig from '../../../network/spiritnet.js'
import * as HydraDxConfig from '../../../network/hydraDx.js'
import { tx, query } from '../../../helper/api.js'

import type { ApiPromise } from '@polkadot/api'
import type { BasicConfig, BasicXcmTestConfiguration, BasisTxContext } from '../types.js'
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
			desc: 'Kilt -> HydraDx live V2',
			precision: BigInt(96),
		},

		network: {
			sender: PolkadotChainConfigs.all.spiritnet.getConfig({}),
			receiver: PolkadotChainConfigs.all.hydraDx.getConfig({}),
			relay: PolkadotChainConfigs.all.polkadot.getConfig({}),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.tokens(HydraDxConfig.kiltTokenId),
		},
		txContext: {
			tx: tx.xcmPallet.limitedReserveTransferAssetsV2(
				SpiritnetConfig.KILT,
				tx.xcmPallet.parachainV2(1, HydraDxConfig.paraId)
			),
			pallets: {
				sender: ['xcmpQueue', 'polkadotXcm'],
				receiver: ['xcmpQueue'],
			},
			balanceToTransfer: BigInt(1e15),
		},
		storage: {
			senderStorage: SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
			receiverStorage: {},
			relayStorage: {},
		},
		sovereignAccount: {
			sender: HydraDxConfig.siblingSovereignAccount,
			receiver: SpiritnetConfig.siblingSovereignAccount,
		},
	},

	{
		config: {
			desc: 'Kilt -> HydraDx at block V2',
			precision: BigInt(99),
		},

		network: {
			sender: PolkadotChainConfigs.all.spiritnet.getConfig({
				blockNumber: PolkadotChainConfigs.all.spiritnet.parameters.blockNumber,
			}),
			receiver: PolkadotChainConfigs.all.hydraDx.getConfig({
				blockNumber: PolkadotChainConfigs.all.hydraDx.parameters.blockNumber,
			}),
			relay: PolkadotChainConfigs.all.polkadot.getConfig({
				blockNumber: PolkadotChainConfigs.all.polkadot.parameters.blockNumber,
			}),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.tokens(HydraDxConfig.kiltTokenId),
		},
		txContext: {
			tx: tx.xcmPallet.limitedReserveTransferAssetsV2(
				SpiritnetConfig.KILT,
				tx.xcmPallet.parachainV2(1, HydraDxConfig.paraId)
			),
			pallets: {
				sender: ['xcmpQueue', 'polkadotXcm', { section: 'balances', method: 'Withdraw' }],
				receiver: ['xcmpQueue', 'tokens', 'currencies'],
			},
			balanceToTransfer: BigInt(1e15),
		},
		storage: {
			senderStorage: SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
			receiverStorage: {},
			relayStorage: {},
		},
		sovereignAccount: {
			sender: HydraDxConfig.siblingSovereignAccount,
			receiver: SpiritnetConfig.siblingSovereignAccount,
		},
	},

	{
		config: {
			desc: 'Kilt -> HydraDx live V3',
			precision: BigInt(96),
		},

		network: {
			sender: PolkadotChainConfigs.all.spiritnet.getConfig({}),
			receiver: PolkadotChainConfigs.all.hydraDx.getConfig({}),
			relay: PolkadotChainConfigs.all.polkadot.getConfig({}),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.tokens(HydraDxConfig.kiltTokenId),
		},
		txContext: {
			tx: tx.xcmPallet.limitedReserveTransferAssetsV3(
				SpiritnetConfig.KILT,
				tx.xcmPallet.parachainV3(1, HydraDxConfig.paraId)
			),
			pallets: {
				sender: ['xcmpQueue', 'polkadotXcm'],
				receiver: ['xcmpQueue'],
			},
			balanceToTransfer: BigInt(1e15),
		},
		storage: {
			senderStorage: SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
			receiverStorage: {},
			relayStorage: {},
		},
		sovereignAccount: {
			sender: HydraDxConfig.siblingSovereignAccount,
			receiver: SpiritnetConfig.siblingSovereignAccount,
		},
	},

	{
		config: {
			desc: 'Kilt -> HydraDx at block V3',
			precision: BigInt(99),
		},

		network: {
			sender: PolkadotChainConfigs.all.spiritnet.getConfig({
				blockNumber: PolkadotChainConfigs.all.spiritnet.parameters.blockNumber,
			}),
			receiver: PolkadotChainConfigs.all.hydraDx.getConfig({
				blockNumber: PolkadotChainConfigs.all.hydraDx.parameters.blockNumber,
			}),
			relay: PolkadotChainConfigs.all.polkadot.getConfig({
				blockNumber: PolkadotChainConfigs.all.polkadot.parameters.blockNumber,
			}),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.tokens(HydraDxConfig.kiltTokenId),
		},
		txContext: {
			tx: tx.xcmPallet.limitedReserveTransferAssetsV3(
				SpiritnetConfig.KILT,
				tx.xcmPallet.parachainV3(1, HydraDxConfig.paraId)
			),
			pallets: {
				sender: ['xcmpQueue', 'polkadotXcm', { section: 'balances', method: 'Withdraw' }],
				receiver: ['xcmpQueue', 'tokens', 'currencies'],
			},
			balanceToTransfer: BigInt(1e15),
		},
		storage: {
			senderStorage: SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
			receiverStorage: {},
			relayStorage: {},
		},
		sovereignAccount: {
			sender: HydraDxConfig.siblingSovereignAccount,
			receiver: SpiritnetConfig.siblingSovereignAccount,
		},
	},

	{
		config: {
			desc: 'Kilt DEV -> HydraDx live v3',
			precision: BigInt(96),
		},

		network: {
			sender: PolkadotChainConfigs.all.spiritnet.getConfig(PolkadotChainConfigs.all.spiritnet.parameters),
			receiver: PolkadotChainConfigs.all.hydraDx.getConfig({}),
			relay: PolkadotChainConfigs.all.polkadot.getConfig({}),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.tokens(HydraDxConfig.kiltTokenId),
		},
		txContext: {
			tx: tx.xcmPallet.limitedReserveTransferAssetsV3(
				SpiritnetConfig.KILT,
				tx.xcmPallet.parachainV3(1, HydraDxConfig.paraId)
			),
			pallets: {
				sender: ['xcmpQueue', 'polkadotXcm'],
				receiver: ['xcmpQueue'],
			},
			balanceToTransfer: BigInt(1e15),
		},
		storage: {
			senderStorage: SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
			receiverStorage: {},
			relayStorage: {},
		},
		sovereignAccount: {
			sender: HydraDxConfig.siblingSovereignAccount,
			receiver: SpiritnetConfig.siblingSovereignAccount,
		},
	},

	{
		config: {
			desc: 'Kilt DEV -> HydraDx live v2',
			precision: BigInt(96),
		},

		network: {
			sender: PolkadotChainConfigs.all.spiritnet.getConfig(PolkadotChainConfigs.all.spiritnet.parameters),
			receiver: PolkadotChainConfigs.all.hydraDx.getConfig({}),
			relay: PolkadotChainConfigs.all.polkadot.getConfig({}),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.tokens(HydraDxConfig.kiltTokenId),
		},
		txContext: {
			tx: tx.xcmPallet.limitedReserveTransferAssetsV2(
				SpiritnetConfig.KILT,
				tx.xcmPallet.parachainV2(1, HydraDxConfig.paraId)
			),
			pallets: {
				sender: ['xcmpQueue', 'polkadotXcm'],
				receiver: ['xcmpQueue'],
			},
			balanceToTransfer: BigInt(1e15),
		},
		storage: {
			senderStorage: SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
			receiverStorage: {},
			relayStorage: {},
		},
		sovereignAccount: {
			sender: HydraDxConfig.siblingSovereignAccount,
			receiver: SpiritnetConfig.siblingSovereignAccount,
		},
	},
] as const
