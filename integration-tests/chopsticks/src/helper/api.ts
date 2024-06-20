/* eslint-disable @typescript-eslint/no-explicit-any */
import { ApiPromise } from '@polkadot/api'

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

export const tx = {
	xtokens,
	xcmPallet,
}

export const query = {
	balances: async ({ api }: { api: ApiPromise }, address: string) =>
		BigInt(((await api.query.system.account(address)) as any).data.free),
	tokens:
		(token: any) =>
		async ({ api }: { api: ApiPromise }, address: string) =>
			BigInt(((await api.query.tokens.accounts(address, token)) as any).free),
}