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

export const balances = {
	transferAllowDeath: ({ api }: { api: ApiPromise }, dest: string, amount: string) =>
		api.tx.balances.transferAllowDeath(dest, amount),
}

export const switchPallet = {
	switchV3:
		() =>
		({ api }: { api: ApiPromise }, acc: any, amount: string) =>
			api.tx.assetSwitchPool1.switch(amount, {
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
			}),
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
	resume:
		() =>
		({ api }: { api: ApiPromise }) =>
			api.tx.assetSwitchPool1.resumeSwitchPair(),
	pause:
		() =>
		({ api }: { api: ApiPromise }) =>
			api.tx.assetSwitchPool1.pauseSwitchPair(),
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
	limitedReserveTransferAssetsV4:
		(token: any, dest: any) =>
		({ api }: { api: ApiPromise }, acc: any, amount: any) =>
			(api.tx.xcmPallet || api.tx.polkadotXcm).limitedReserveTransferAssets(
				dest,
				{
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
				},
				{
					V4: [
						{
							id: token,
							fun: { Fungible: amount },
						},
					],
				},
				0,
				'Unlimited'
			),
	transferAssetsV3:
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
	transferAssetsV4:
		(dest: any, token: any) =>
		({ api }: { api: ApiPromise }, acc: any, amount: any) =>
			api.tx.polkadotXcm.transferAssets(
				dest,
				{
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
				},
				{
					V4: [
						{
							id: token,
							fun: { Fungible: amount },
						},
					],
				},
				0,
				'Unlimited'
			),
	transferAssetsUsingTypeAndThenV4:
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
	transferAssetsUsingTypeAndThenV3:
		(dest: any, token: any, xcmMessage: any) =>
		({ api }: { api: ApiPromise }, balanceToTransfer: string) =>
			api.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
				dest,
				{
					V3: [
						{
							id: token,
							fun: { Fungible: balanceToTransfer },
						},
					],
				},
				'LocalReserve',
				{ V3: token },
				'LocalReserve',
				xcmMessage,
				'Unlimited'
			),
	send:
		(destination: object) =>
		({ api }: { api: ApiPromise }, xcmMessage: object) =>
			(api.tx.xcmPallet || api.tx.polkadotXcm).send(destination, xcmMessage),
}

/**
 * Different pallets to submit tx
 */
export const tx = {
	xtokens,
	xcmPallet,
	switchPallet,
	balances,
}

/**
 * Query functions for different chains.
 * Native tokens are fetched via the system pallet, while other tokens are fetched via the tokens or assets pallet.
 *
 */
export const query = {
	balances: async ({ api }: { api: ApiPromise }, address: string) =>
		((await api.query.system.account(address)) as any).data.free.toBigInt(),
	foreignAssets:
		(assetId: any) =>
		async ({ api }: { api: ApiPromise }, address: string) => {
			const accountInfo: any = await api.query.foreignAssets.account(assetId, address)
			if (accountInfo.isNone) {
				return 0n
			}
			return accountInfo.unwrap().balance.toBigInt()
		},
	fungibles:
		(assetId: any) =>
		async ({ api }: { api: ApiPromise }, address: string) => {
			const accountInfo: any = await api.query.fungibles.account(assetId, address)
			if (accountInfo.isNone) {
				return 0n
			}
			return accountInfo.unwrap().balance.toBigInt()
		},
	tokens:
		(token: any) =>
		async ({ api }: { api: ApiPromise }, address: string) =>
			((await api.query.tokens.accounts(address, token)) as any).data.free.toBigInt(),
}
