/* eslint-disable @typescript-eslint/no-explicit-any */

import type { KeyringPair } from '@polkadot/keyring/types'

import * as PolkadotChainConfigs from '../../../network/polkadot/index.js'
import { initialBalanceHDX, initialBalanceKILT, keysAlice, keysBob } from '../../../helper/utils.js'
import * as SpiritnetConfig from '../../../network/polkadot/spiritnet.js'
import * as HydraDxConfig from '../../../network/polkadot/hydraDx.js'
import { tx, query } from '../../../helper/api.js'
import { ApiPromise } from '@polkadot/api'
import { SubmittableExtrinsic } from '@polkadot/api/types'
import { EventFilter, SetupOption } from '@acala-network/chopsticks-testing'

interface LimitedReserveTestConfiguration {
	config: {
		desc: string
		precision: bigint
	}

	blockchain: {
		sender: SetupOption
		receiver: SetupOption
		relay: SetupOption
	}

	query: {
		sender: (
			{
				api,
			}: {
				api: ApiPromise
			},
			address: string
		) => Promise<bigint>
		receiver: (
			{
				api,
			}: {
				api: ApiPromise
			},
			address: string
		) => Promise<bigint>
	}

	test: {
		tx: (
			{
				api,
			}: {
				api: ApiPromise
			},
			acc: string,
			amount: number | string
		) => SubmittableExtrinsic<'promise'>

		pallets: {
			sender: EventFilter[]
			receiver: EventFilter[]
		}

		balanceToTransfer: bigint
	}
	accounts: {
		senderAccount: KeyringPair
		receiverAccount: KeyringPair
	}
	storage: {
		senderStorage: Record<string, Record<string, unknown>>
		receiverStorage: Record<string, Record<string, unknown>>
		relayStorage: Record<string, Record<string, unknown>>
	}

	sovereignAccount: {
		sender: string
		receiver: string
	}
}

// Test pairs for limited reserve transfers
export const testPairsLimitedReserveTransfers: LimitedReserveTestConfiguration[] = [
	// Kilt -> HydraDx
	{
		config: {
			desc: 'Kilt -> HydraDx live status',
			precision: BigInt(95),
		},

		blockchain: {
			sender: PolkadotChainConfigs.all.spiritnet.config(),
			receiver: PolkadotChainConfigs.all.hydraDx.config(),
			relay: PolkadotChainConfigs.all.polkadot.config(),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.tokens(HydraDxConfig.kiltTokenId),
		},
		test: {
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
			sender: SpiritnetConfig.hydraDxSovereignAccount,
			receiver: SpiritnetConfig.hydraDxSovereignAccount,
		},
	},

	{
		config: {
			desc: 'Kilt -> HydraDx at block',
			precision: BigInt(100),
		},

		blockchain: {
			sender: PolkadotChainConfigs.all.spiritnet.config(PolkadotChainConfigs.all.spiritnet.blockNumber),
			receiver: PolkadotChainConfigs.all.hydraDx.config(PolkadotChainConfigs.all.hydraDx.blockNumber),
			relay: PolkadotChainConfigs.all.polkadot.config(PolkadotChainConfigs.all.polkadot.blockNumber),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.tokens(HydraDxConfig.kiltTokenId),
		},
		test: {
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
			sender: SpiritnetConfig.hydraDxSovereignAccount,
			receiver: SpiritnetConfig.hydraDxSovereignAccount,
		},
	},

	// HydraDx -> Kilt
	{
		config: {
			desc: 'HydraDx -> KILT at live',
			precision: BigInt(95),
		},

		blockchain: {
			sender: PolkadotChainConfigs.all.spiritnet.config(),
			receiver: PolkadotChainConfigs.all.hydraDx.config(),
			relay: PolkadotChainConfigs.all.polkadot.config(),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.tokens(HydraDxConfig.kiltTokenId),
			receiver: query.balances,
		},
		test: {
			tx: tx.xtokens.transfer(HydraDxConfig.KILTLocation, tx.xtokens.parachainV3(SpiritnetConfig.paraId)),
			pallets: {
				sender: [],
				receiver: [],
			},
			balanceToTransfer: BigInt(1e15),
		},
		storage: {
			senderStorage: {
				...HydraDxConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceHDX),
				...HydraDxConfig.assignKiltTokensToAccounts([keysAlice.address], initialBalanceKILT),
			},
			receiverStorage: {},
			relayStorage: {},
		},
		sovereignAccount: {
			sender: SpiritnetConfig.hydraDxSovereignAccount,
			receiver: SpiritnetConfig.hydraDxSovereignAccount,
		},
	},
] as const

export type TestType = (typeof testPairsLimitedReserveTransfers)[number]
