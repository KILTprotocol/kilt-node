/* eslint-disable @typescript-eslint/no-explicit-any */
import { ApiPromise } from '@polkadot/api'

/**
 * All possible ways to submit an XCM message for the xtokens pallet.
 * different structs for the xcm versions are provided
 */
export const xtokens = {
	parachainV2: (paraId: number) => (acc: any) => ({
		V1: {
			parents: 1,
			interior: {
				X2: [
					{ Parachain: paraId },
					{
						AccountId32: {
							network: 'Any',
							id: acc,
						},
					},
				],
			},
		},
	}),
	parachainV3: (paraId: number) => (acc: any) => ({
		V3: {
			parents: 1,
			interior: {
				X2: [
					{ Parachain: paraId },
					{
						AccountId32: {
							id: acc,
						},
					},
				],
			},
		},
	}),
	transfer:
		(token: any, dest: (dest: any) => any, weight: any = 'Unlimited') =>
		({ api }: { api: ApiPromise }, acc: any, amount: any) =>
			api.tx.xTokens.transfer(token, amount, dest(acc), weight),
}

/**
 * All possible ways to submit an XCM message for the xcmPallet.
 * different structs for the xcm versions are provided
 */
export const xcmPallet = {
	parachainV2: (parents: number, paraId: number) => ({
		V2: {
			parents,
			interior: {
				X1: { Parachain: paraId },
			},
		},
	}),
	parachainV3: (parents: number, paraId: any) => ({
		V3: {
			parents,
			interior: {
				X1: { Parachain: paraId },
			},
		},
	}),
	limitedTeleportAssets:
		(token: any, amount: any, dest: any) =>
		({ api }: { api: ApiPromise }, acc: any) =>
			(api.tx.xcmPallet || api.tx.polkadotXcm).limitedTeleportAssets(
				dest,
				{
					V3: {
						parents: 0,
						interior: {
							X1: {
								AccountId32: {
									// network: 'Any',
									id: acc,
								},
							},
						},
					},
				},
				{
					V3: [
						{
							id: token,
							fun: { Fungible: amount },
						},
					],
				},
				0,
				'Unlimited'
			),
	limitedReserveTransferAssetsV2:
		(token: any, dest: any) =>
		({ api }: { api: ApiPromise }, acc: any, amount: any) =>
			(api.tx.xcmPallet || api.tx.polkadotXcm).limitedReserveTransferAssets(
				dest,
				{
					V2: {
						parents: 0,
						interior: {
							X1: {
								AccountId32: {
									network: 'Any',
									id: acc,
								},
							},
						},
					},
				},
				{
					V2: [
						{
							id: token,
							fun: { Fungible: amount },
						},
					],
				},
				0,
				'Unlimited'
			),
	limitedReserveTransferAssetsV3:
		(token: any, dest: any) =>
		({ api }: { api: ApiPromise }, acc: any, amount: any) =>
			(api.tx.xcmPallet || api.tx.polkadotXcm).limitedReserveTransferAssets(
				dest,
				{
					V3: {
						parents: 0,
						interior: {
							X1: {
								AccountId32: {
									id: acc,
								},
							},
						},
					},
				},
				{
					V3: [
						{
							id: token,
							fun: { Fungible: amount },
						},
					],
				},
				0,
				'Unlimited'
			),
}

const assetSwap = {
	beneficiaryV3: (account: string) => ({
		V3: {
			parents: 0,
			interior: {
				X1: {
					AccountId32: {
						id: account,
					},
				},
			},
		},
	}),

	asset: {
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

	swap:
		() =>
		({ api }: { api: ApiPromise }, beneficiary: any, amount: number | string) =>
			api.tx.assetSwap.swap(amount, beneficiary),

	transferAssetsUsingTypeAndThen: (dest: any, remoteFeeId: any) => {
		return ({ api }: { api: ApiPromise }, acc: any, funds: any) => {
			const copyRemoteFeeId = { ...remoteFeeId.V3 }

			console.log('THESE ARE THE PROVIDED FUNDS', JSON.stringify(copyRemoteFeeId))

			copyRemoteFeeId.fun = { Fungible: 1 }
			copyRemoteFeeId.Concrete.parents = 10

			const assets = {
				V3: [copyRemoteFeeId],
			}

			console.log('Updated funds', JSON.stringify(assets))

			const tx = api.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
				dest,
				{
					V3: [
						{
							Concrete: {
								parents: '2',
								interior: {
									X2: [
										{ GlobalConsensus: { Ethereum: { chainId: '11155111' } } },
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
							fun: { Fungible: 8888e9 },
						},
					],
				},
				'LocalReserve',
				remoteFeeId,
				'LocalReserve',
				{
					V3: [
						{
							DepositAsset: {
								assets: { Wild: 'All' },
								beneficiary: {
									parents: 0,
									interior: {
										X1: {
											AccountId32: {
												id: acc,
											},
										},
									},
								},
							},
						},
					],
				},
				'Unlimited'
			)

			console.log('DATA OF THE CALL', tx.data)
			return tx
		}
	},
}

/**
 * Different pallets to submit xcm messages.
 */
export const tx = {
	xtokens,
	xcmPallet,
	assetSwap,
}

/**
 * Query functions for different chains.
 * Native tokens are fetched via the system pallet, while other tokens are fetched via the tokens pallet.
 *
 */

export const query = {
	balances: async ({ api }: { api: ApiPromise }, address: string) =>
		BigInt(((await api.query.system.account(address)) as any).data.free),
	tokens:
		(token: any) =>
		async ({ api }: { api: ApiPromise }, address: string) =>
			BigInt(((await api.query.tokens.accounts(address, token)) as any).free),
}
