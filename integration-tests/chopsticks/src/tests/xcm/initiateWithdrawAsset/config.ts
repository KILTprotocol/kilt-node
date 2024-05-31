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

interface Config {
	desc: string
	precision: bigint
}

interface Blockchain {
	sender: SetupOption
	receiver: SetupOption
	relay: SetupOption
}

interface Query {
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

interface Test {
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

interface Accounts {
	senderAccount: KeyringPair
	receiverAccount: KeyringPair
}

interface Storage {
	senderStorage: Record<string, Record<string, unknown>>
	receiverStorage: Record<string, Record<string, unknown>>
	relayStorage: Record<string, Record<string, unknown>>
}

interface SovereignAccount {
	sender: string
	receiver: string
}

interface LimitedReserveTestConfiguration {
	config: Config
	blockchain: Blockchain
	query: Query
	test: Test
	accounts: Accounts
	storage: Storage
	sovereignAccount: SovereignAccount
}

// Test pairs for limited reserve transfers
export const testPairsLimitedReserveTransfers: LimitedReserveTestConfiguration[] = [
	// Kilt -> HydraDx
	{
		config: {
			desc: 'HydraDx -> KILT live',
			precision: BigInt(95),
		},

		blockchain: {
			sender: PolkadotChainConfigs.all.hydraDx.config(),
			receiver: PolkadotChainConfigs.all.spiritnet.config(),
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
			tx: tx.xtokens.transfer(HydraDxConfig.kiltTokenId, tx.xtokens.parachainV3(SpiritnetConfig.paraId)),
			pallets: {
				sender: [],
				receiver: [],
			},
			balanceToTransfer: BigInt(1e15),
		},
		storage: {
			senderStorage: {
				...HydraDxConfig.assignKiltTokensToAccounts([keysAlice.address], initialBalanceKILT),
				...HydraDxConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceHDX),
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
