import { test } from 'vitest'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assetHub.js'
import {
	KILT,
	ROC,
	getAssetSwitchParameters,
	initialBalanceKILT,
	initialBalanceROC,
	keysAlice,
	keysCharlie,
} from '../../utils.js'
import {
	peregrineContext,
	getFreeBalancePeregrine,
	getFreeRocPeregrine,
	getFreeEkiltAssetHub,
	assethubContext,
} from '../index.js'
import {
	checkBalance,
	createBlock,
	setStorage,
	hexAddress,
	checkBalanceInRange,
	getXcmMessageV4ToSendEkilt,
	checkSwitchPalletInvariant,
	checkBalanceMovementIncomingSwitch,
} from '../utils.js'
import { getAccountLocationV4, getRelayNativeAssetIdLocationV4, getSiblingLocationV4 } from '../../network/utils.js'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

/**
 * Full e2e flow between Peregrine and AssetHub. More checks are provided in individual test cases.
 *
 * 1. Send ROCs from AssetHub to Peregrine
 * 2. Switch KILTs on Peregrine
 * 3. Send eKILTs back to AssetHub
 * 4. Send ROCs back to AssetHub
 */
test('Full e2e tests', async ({ expect }) => {
	const { checkEvents } = withExpect(expect)

	// 10 % of relay tokens are used as fees
	const feeAmount = (ROC * BigInt(10)) / BigInt(100)

	const remoteAssetId = { V4: AssetHubConfig.eKiltLocation }
	const remoteXcmFeeId = { V4: { id: AssetHubConfig.nativeTokenLocation, fun: { Fungible: feeAmount } } }
	const remoteReserveLocation = getSiblingLocationV4(AssetHubConfig.paraId)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	const switchParameters = getAssetSwitchParameters()

	await setStorage(
		peregrineContext,
		PeregrineConfig.setSwitchPair(switchParameters, remoteAssetId, remoteXcmFeeId, remoteReserveLocation)
	)

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccountsAsStorage(
			[keysAlice.address, PeregrineConfig.sovereignAccountAsSibling],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address),
	})
	await setStorage(
		assethubContext,
		AssetHubConfig.assignForeignAssetToAccounts([
			[PeregrineConfig.sovereignAccountAsSibling, switchParameters.sovereignSupply],
		])
	)
	// 1. send ROCs 2 Peregrine

	const peregrineDestination = getSiblingLocationV4(PeregrineConfig.paraId)
	const beneficiary = getAccountLocationV4(hexAddress(keysAlice.address))
	const rocAsset = { V4: [getRelayNativeAssetIdLocationV4((ROC * BigInt(2)).toString())] }

	const signedTx1 = assethubContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(peregrineDestination, beneficiary, rocAsset, 0, 'Unlimited')
		.signAsync(keysAlice)

	// Send the transaction and create a block0
	await sendTransaction(signedTx1)
	await createBlock(assethubContext)

	// process msg.
	await createBlock(peregrineContext)
	// Alice should have some Rocs on Peregrine
	const aliceRocBalance = await getFreeRocPeregrine(keysAlice.address)
	expect(aliceRocBalance).toBeGreaterThan(BigInt(0))

	await checkSwitchPalletInvariant(expect)

	// 2. switch KILTs
	const balanceToTransfer = initialBalanceKILT / BigInt(2)
	const signedTx2 = peregrineContext.api.tx.assetSwitchPool1
		.switch(balanceToTransfer.toString(), beneficiary)
		.signAsync(keysAlice)

	const events1 = await sendTransaction(signedTx2)

	await createBlock(peregrineContext)
	await checkEvents(events1, 'assetSwitchPool1').toMatchSnapshot(
		'receiver Peregrine::assetSwitchPool1::[LocalToRemoteSwitchExecuted]'
	)

	await createBlock(assethubContext)
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, balanceToTransfer)

	await checkSwitchPalletInvariant(expect)

	// 3. send eKILTs back
	const dest = getSiblingLocationV4(PeregrineConfig.paraId)
	const remoteFeeId = { V4: AssetHubConfig.eKiltLocation }
	const funds = {
		V4: [
			{
				id: AssetHubConfig.eKiltLocation,
				fun: { Fungible: balanceToTransfer },
			},
		],
	}

	const signedTx3 = assethubContext.api.tx.polkadotXcm
		.transferAssetsUsingTypeAndThen(
			dest,
			funds,
			'LocalReserve',
			remoteFeeId,
			'LocalReserve',
			getXcmMessageV4ToSendEkilt(keysAlice.address),
			'Unlimited'
		)
		.signAsync(keysAlice)

	const events2 = await sendTransaction(signedTx3)

	await createBlock(assethubContext)
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, BigInt(0))
	// assets should move from Sovereign account to user.
	await checkEvents(events2, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'sender AssetHub::foreignAssets::[Transferred]'
	)

	await createBlock(peregrineContext)
	await checkBalanceInRange(getFreeBalancePeregrine, keysAlice.address, expect, [
		BigInt(74) * KILT,
		BigInt(75) * KILT,
	])

	await checkSwitchPalletInvariant(expect)
	await checkBalanceMovementIncomingSwitch(balanceToTransfer, expect, keysAlice.address)

	// 4. send ROCs back

	const assetHubDestination = getSiblingLocationV4(AssetHubConfig.paraId)
	const assets = { V4: [getRelayNativeAssetIdLocationV4(ROC.toString())] }

	const signedTx4 = peregrineContext.api.tx.polkadotXcm
		.transferAssets(assetHubDestination, beneficiary, assets, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events3 = await sendTransaction(signedTx4)
	await createBlock(peregrineContext)

	// The xcm message should be send to AH and the funds should be burned from user.
	await checkEvents(events3, 'fungibles').toMatchSnapshot('sender Peregrine::fungibles::[Burned]')

	expect(await getFreeRocPeregrine(keysAlice.address)).toBe(BigInt(899999965317))

	// Process the message on AH
	await createBlock(assethubContext)

	await checkSwitchPalletInvariant(expect)
}, 20_000)
