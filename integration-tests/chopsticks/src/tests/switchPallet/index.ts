import { ExpectStatic } from 'vitest'
import { ApiPromise } from '@polkadot/api'
import { SetupConfig } from '@acala-network/chopsticks-testing'

import { hexAddress } from '../../helper/utils.js'

export function getXcmMessageV4ToSendEkilt(address: string) {
	return {
		V4: [
			{
				DepositAsset: {
					assets: { Wild: 'All' },
					beneficiary: {
						parents: 0,
						interior: {
							X1: [
								{
									AccountId32: {
										id: hexAddress(address),
									},
								},
							],
						},
					},
				},
			},
		],
	}
}

export function getDepositXcmMessageV3(assetId: object) {
	return (balanceToTransfer: string, receiver: string) => ({
		V3: [
			{
				ReserveAssetDeposited: [
					{
						id: { Concrete: assetId },
						fun: { Fungible: balanceToTransfer },
					},
				],
			},
			'ClearOrigin',
			{
				BuyExecution: {
					fees: {
						id: { Concrete: assetId },
						fun: { Fungible: balanceToTransfer },
					},
					weightLimit: 'Unlimited',
				},
			},
			{
				DepositAsset: {
					assets: { Wild: 'All' },
					beneficiary: {
						parents: 0,
						interior: {
							X1: {
								AccountId32: {
									id: hexAddress(receiver),
								},
							},
						},
					},
				},
			},
		],
	})
}

export function getXcmV4ReclaimMessage(assetId: object) {
	return (amount: string, receiver: string) => ({
		V4: [
			{ WithdrawAsset: [{ id: { parents: 0, interior: 'Here' }, fun: { Fungible: amount } }] },
			{
				BuyExecution: {
					weightLimit: 'Unlimited',
					fees: { id: { parents: 0, interior: 'Here' }, fun: { Fungible: amount } },
				},
			},
			{
				ClaimAsset: {
					// Specify xcm version 4
					ticket: { parents: 0, interior: { X1: [{ GeneralIndex: 4 }] } },
					assets: [
						{
							id: assetId,
							fun: { Fungible: amount },
						},
					],
				},
			},
			{
				DepositAsset: {
					assets: { Wild: 'All' },
					beneficiary: {
						parents: 0,
						interior: {
							X1: [
								{
									AccountId32: {
										id: hexAddress(receiver),
									},
								},
							],
						},
					},
				},
			},
		],
	})
}

export async function checkSwitchPalletInvariant(
	expect: ExpectStatic,
	nativeContext: SetupConfig,
	foreignContext: SetupConfig,
	sovereignAccount: string,
	queryNativeBalance: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>,
	queryForeignBalance: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>,
	deltaStoredSovereignSupply = 0n
) {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const switchPairInfo: any = await nativeContext.api.query.assetSwitchPool1.switchPair()
	if (switchPairInfo.isNone) {
		return
	}

	// check pool account balance
	const switchPoolAccount = switchPairInfo.unwrap().poolAccount

	const poolAccountBalance = await queryNativeBalance(nativeContext, switchPoolAccount)

	const sovereignEKiltSupply = await queryForeignBalance(foreignContext, sovereignAccount)

	const remoteAssetSovereignTotalBalance = switchPairInfo.unwrap().remoteAssetSovereignTotalBalance.toBigInt()
	const remoteAssetCirculatingSupply = switchPairInfo.unwrap().remoteAssetCirculatingSupply.toBigInt()
	const remoteAssetTotalSupply = switchPairInfo.unwrap().remoteAssetTotalSupply.toBigInt()

	const lockedBalanceFromTotalAndCirculating = remoteAssetTotalSupply - remoteAssetCirculatingSupply

	// Check pool account has enough funds to cover the circulating supply

	// everybody can send funds to the pool account. The pool account should have at least the circulating supply
	expect(poolAccountBalance).toBeGreaterThanOrEqual(remoteAssetCirculatingSupply)
	expect(remoteAssetSovereignTotalBalance).toBe(lockedBalanceFromTotalAndCirculating)
	expect(sovereignEKiltSupply).toBe(remoteAssetSovereignTotalBalance + deltaStoredSovereignSupply)
}

export async function getPoolAccount({ api }: { api: ApiPromise }) {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const switchPairInfo: any = await api.query.assetSwitchPool1.switchPair()
	if (switchPairInfo.isNone) {
		return
	}
	return switchPairInfo.unwrap().poolAccount
}

export async function getRemoteLockedSupply({ api }: { api: ApiPromise }): Promise<bigint> {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const switchPairInfo: any = await api.query.assetSwitchPool1.switchPair()

	if (switchPairInfo.isNone) {
		return 0n
	}

	return switchPairInfo.unwrap().remoteAssetSovereignTotalBalance.toBigInt()
}

export async function getReceivedNativeTokens({ api }: { api: ApiPromise }): Promise<bigint> {
	const events = await api.query.system.events()

	const polkadotFees = events.filter(
		(event) =>
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			(event as any).event.data.section === 'assetSwitchPool1' &&
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			(event as any).event.data.method === 'RemoteToLocalSwitchExecuted'
	)[0]

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	return BigInt((polkadotFees as any).event.data.amount.toString())
}

export async function isSwitchPaused({ api }: { api: ApiPromise }): Promise<boolean> {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const switchPairInfo: any = await api.query.assetSwitchPool1.switchPair()
	if (switchPairInfo.isNone) {
		return false
	}

	return JSON.parse(switchPairInfo.unwrap().toString()).status === 'Paused'
}
