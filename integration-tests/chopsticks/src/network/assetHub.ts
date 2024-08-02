import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'

import type { Config } from './types.js'
import { initialBalanceDOT, initialBalanceKILT, toNumber } from '../utils.js'

/// Options used to create the Spiritnet context
export const getSetupOptions = ({
	blockNumber = undefined,
	wasmOverride = undefined,
}: {
	blockNumber?: number
	wasmOverride?: string
}) =>
	({
		endpoint: process.env.ASSETHUB_WSS || 'wss://asset-hub-polkadot-rpc.dwellir.com',
		db: './db/assethub.db.sqlite',
		port: toNumber(process.env.ASSETHUB_PORT) || 9003,
		wasmOverride,
		blockNumber,
		runtimeLogLevel: 5,
	}) as SetupOption

/// AssetHub has no own coin. Teleported dots are used as the native token.
export function assignDotTokensToAccounts(addr: string[], balance: bigint = initialBalanceDOT) {
	return {
		System: {
			Account: addr.map((address) => [[address], { providers: 1, data: { free: balance.toString() } }]),
		},
	}
}

export function createForeignAsset(manager: string, accountInfo: [string, bigint][]) {
	const supply = accountInfo.map(([, balance]) => balance).reduce((acc, balance) => acc + balance, BigInt(0))
	return {
		foreignAssets: {
			asset: [
				[
					[
						{
							parents: 2,
							interior: {
								X2: [
									{ GlobalConsensus: { Ethereum: { chainId: 11155111 } } },
									{
										AccountKey20: {
											network: null,
											key: '0x06012c8cf97bead5deae237070f9587f8e7a266d',
										},
									},
								],
							},
						},
					],
					{
						// owner is set to relay chain sovereign account. Check out if this is correct.
						owner: '5Dt6dpkWPwLaH4BBCKJwjiWrFVAGyYk3tLUabvyn4v7KtESG',
						issuer: manager,
						admin: manager,
						freezer: manager,
						supply,
						deposit: 100000000000,
						minBalance: 100,
						isSufficient: false,
						accounts: accountInfo.length,
						sufficients: 0,
						approvals: 0,
						status: 'Live',
					},
				],
			],

			account: accountInfo.map((account) => [
				[
					{
						parents: 2,
						interior: {
							X2: [
								{
									GlobalConsensus: { Ethereum: { chainId: 11155111 } },
								},
								{
									AccountKey20: {
										network: null,
										key: '0x06012c8cf97bead5deae237070f9587f8e7a266d',
									},
								},
							],
						},
					},
					account[0],
				],
				{
					balance: account[1],
					status: 'Liquid',
					reason: 'Consumer',
					extra: null,
				},
			]),
		},
	}
}

/// Assigns KSM to an account
export function assignKSMtoAccounts(addr: string[], balance: bigint = initialBalanceDOT) {
	return {
		foreignAssets: {
			account: addr.map((addr) => [
				[KSMAssetLocation, addr],
				{
					balance: balance,
					status: 'Liquid',
					reason: 'Consumer',
					extra: null,
				},
			]),
		},
	}
}

/// Assigns the foreign asset to the accounts.
/// Does not check if supply is matching the sum of the account balances.
export function assignForeignAssetToAccounts(addr: string[], balance: bigint = initialBalanceKILT) {
	return {
		foreignAssets: {
			account: [
				addr.map(
					(addr) => [
						{
							parents: 2,
							interior: {
								X2: [
									{
										GlobalConsensus: { Ethereum: { chainId: 11155111 } },
									},
									{
										AccountKey20: {
											network: null,
											key: '0x06012c8cf97bead5deae237070f9587f8e7a266d',
										},
									},
								],
							},
						},
						addr,
					],
					{
						balance: balance,
						status: 'Liquid',
						reason: 'Consumer',
						extra: null,
					}
				),
			],
		},
	}
}

/// AssetHub ParaId
export const paraId = 1000

export const KSMAssetLocation = {
	parents: 2,
	interior: {
		X1: {
			GlobalConsensus: 'Kusama',
		},
	},
}

// Sibling Sovereign Account
export const sovereignAccountOnSiblingChains = '4qXPdpimHh8TR24RSk994yVzxx4TLfvKj5i1qH5puvWmfAqy'

/// Native token in AssetHub
export const native = { parents: 1, interior: 'Here' }

export const eKiltLocation = {
	parents: 2,
	interior: {
		X2: [
			{
				GlobalConsensus: { Ethereum: { chainId: 11155111 } },
			},
			{
				AccountKey20: {
					network: null,
					key: '0x06012c8cf97bead5deae237070f9587f8e7a266d',
				},
			},
		],
	},
}

export async function getContext(): Promise<Config> {
	const options = getSetupOptions({})
	return setupContext(options)
}
