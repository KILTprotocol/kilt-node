import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'

import type { Config } from './types.js'
import { initialBalanceKILT, toNumber } from '../utils.js'

/// Options used to create the Spiritnet context
export const getSetupOptions = ({
	blockNumber = undefined,
	wasmOverride = undefined,
}: {
	blockNumber?: number
	wasmOverride?: string
}) =>
	({
		endpoint: process.env.ASSETHUB_WSS || 'wss://rococo-asset-hub-rpc.polkadot.io',
		db: './db/assethub.db.sqlite',
		port: toNumber(process.env.ASSETHUB_PORT) || 9003,
		wasmOverride,
		blockNumber,
	}) as SetupOption

export function createForeignAsset(
	manager: string,
	addr: string[],
	balance: bigint = initialBalanceKILT * BigInt(1000000000000)
) {
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
									// Todo: replace with the actual address
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
						supply: BigInt(addr.length) * balance,
						deposit: 100000000000,
						minBalance: 100,
						isSufficient: false,
						accounts: addr.length,
						sufficients: 0,
						approvals: 0,
						status: 'Live',
					},
				],
			],

			account: addr.map((addr) => [
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
					addr,
				],
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

/// AssetHub has no own coin. Teleported dots are used as the native token.
export function assignDotTokensToAccounts(addr: string[], balance: bigint) {
	return {
		System: {
			Account: addr.map((address) => [[address], { providers: 1, data: { free: balance.toString() } }]),
		},
	}
}

/// AssetHub ParaId
export const paraId = 1000

/// Native token in AssetHub
export const ROC = { parents: 1, interior: 'Here' }

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

// Sibling Sovereign Account
export const siblingSovereignAccount = '5Ec4AhPZk8STuex8Wsi9TwDtJQxKqzPJRCH7348Xtcs9vZLJ'

export async function getContext(): Promise<Config> {
	const options = getSetupOptions({})
	return setupContext(options)
}
