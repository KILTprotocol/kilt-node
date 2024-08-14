import { test } from 'vitest'

import * as SpiritnetConfig from '../../../../network/spiritnet.js'
import * as AssetHubConfig from '../../../../network/assetHub.js'
import { KILT, initialBalanceKILT, keysAlice } from '../../../../utils.js'
import { spiritnetContext, getFreeBalanceSpiritnet } from '../../../index.js'
import { getAccountLocationV3, getNativeAssetIdLocationV3, getSiblingLocationV3 } from '../../../../network/utils.js'
import { checkBalance, createBlock, hexAddress, setStorage } from '../../../utils.js'

test('Teleport assets from Spiritnet Account Alice -> AH Account Alice', async ({ expect }) => {
	// Assign alice some KILT tokens
	await setStorage(spiritnetContext, {
		...SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...SpiritnetConfig.XcmPalletSafeVersion3StorageEntry(),
	})

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalanceSpiritnet, keysAlice.address, expect, initialBalanceKILT)

	const aliceAddress = hexAddress(keysAlice.address)
	const assetHubDestination = { V3: getSiblingLocationV3(AssetHubConfig.paraId) }
	const beneficiary = getAccountLocationV3(aliceAddress)
	const kiltAsset = { V3: [getNativeAssetIdLocationV3(KILT)] }

	// Teleportation should exhaust resources. This is intended until isTeleport is enabled in the XCM config.
	await expect(async () => {
		await spiritnetContext.api.tx.polkadotXcm
			.teleportAssets(assetHubDestination, beneficiary, kiltAsset, 0)
			.signAndSend(keysAlice)
		await createBlock(spiritnetContext)
	}).rejects.toThrowErrorMatchingSnapshot()

	// Check sender state
}, 20_000)
