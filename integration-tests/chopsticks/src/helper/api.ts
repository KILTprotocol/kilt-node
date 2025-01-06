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

export const switchPallet = {
	switchV4:
		() =>
		({ api }: { api: ApiPromise }, acc: any, amount: string) =>
			api.tx.assetSwitchPool1.switch(amount, {
				V4: {
					parents: 0,
					interior: {
						X1: [
							{
								AccountId32: {
									id: acc,
								},
							},
						],
					},
				},
			}),
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
	parachainV4: (parents: number, paraId: any) => ({
		V4: {
			parents,
			interior: { X1: [{ Parachain: paraId }] },
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
	transferAssets:
		(dest: any, token: any) =>
		({ api }: { api: ApiPromise }, acc: any, amount: any) =>
			api.tx.polkadotXcm.transferAssets(
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
	transferAssetsUsingTypeAndThen:
		(dest: any, token: any, xcmMessage: any) =>
		({ api }: { api: ApiPromise }, balanceToTransfer: string) =>
			api.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
				dest,
				{
					V4: [
						{
							id: token,
							fun: { Fungible: balanceToTransfer },
						},
					],
				},
				'LocalReserve',
				{ V4: token },
				'LocalReserve',
				xcmMessage,
				'Unlimited'
			),
}

/**
 * Different pallets to submit xcm messages.
 */
export const tx = {
	xtokens,
	xcmPallet,
	switchPallet,
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
