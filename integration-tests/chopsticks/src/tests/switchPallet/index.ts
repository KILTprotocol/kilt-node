import { ExpectStatic } from 'vitest'
import { hexAddress } from '../../helper/utils.js'
import { ApiPromise } from '@polkadot/api'
import { SetupConfig } from '@acala-network/chopsticks-testing'

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

export async function checkSwitchPalletInvariant(
	expect: ExpectStatic,
	nativeContext: SetupConfig,
	foreignContext: SetupConfig,
	sovereignAccount: string,
	queryNativeBalance: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>,
	queryForeignBalance: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>,
	deltaStoredSovereignSupply = BigInt(0)
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

export async function getPoolAccount(context: SetupConfig) {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const switchPairInfo: any = await context.api.query.assetSwitchPool1.switchPair()
	if (switchPairInfo.isNone) {
		return
	}
	return switchPairInfo.unwrap().poolAccount
}

export async function getRemoteLockedSupply(context: SetupConfig): Promise<bigint> {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const switchPairInfo: any = await context.api.query.assetSwitchPool1.switchPair()

	if (switchPairInfo.isNone) {
		return BigInt(0)
	}

	return switchPairInfo.unwrap().remoteAssetSovereignTotalBalance.toBigInt()
}
