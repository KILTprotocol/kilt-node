import { SetupOption } from '@acala-network/chopsticks-testing'

import { initialBalanceDOT, initialBalanceKILT, toNumber } from '../helper/utils.js'

/// Options used to create the Spiritnet context
export const getSetupOptions = ({
	blockNumber = undefined,
	wasmOverride = undefined,
}: {
	blockNumber?: number
	wasmOverride?: string
}) =>
	({
		endpoint: process.env.ASSETHUB_WSS || 'wss://asset-hub-rococo-rpc.dwellir.com',
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
						supply: '4242424242424242424242',
						deposit: 100000000000,
						minBalance: 100,
						isSufficient: false,
						accounts: 1,
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
export function assignDotTokensToAccounts(
	addr: string[],
	balance: bigint = initialBalanceDOT * BigInt(100000000000000)
) {
	return {
		System: {
			Account: addr.map((address) => [[address], { providers: 1, data: { free: '1000000000000000000000000' } }]),
		},
	}
}

/// AssetHub ParaId
export const paraId = 1000

/// Native token in AssetHub
export const DOT = { Concrete: { parents: 0, interior: 'Here' } }

// Sibling Sovereign Account
export const siblingSovereignAccount = '5Ec4AhPZk8STuex8Wsi9TwDtJQxKqzPJRCH7348Xtcs9vZLJ'
