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
import { peregrineContext, getFreeRocPeregrine, getFreeEkiltAssetHub, assethubContext } from '../index.js'
import {
	checkBalance,
	createBlock,
	setStorage,
	hexAddress,
	getXcmMessageV4ToSendEkilt,
	checkSwitchPalletInvariant,
} from '../utils.js'
import { getAccountLocationV4, getRelayNativeAssetIdLocationV4, getSiblingLocationV4 } from '../../network/utils.js'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

/**
 * These test cases should primarily check the behavior of the switch pair when it is paused.
 * Similar to the full end-to-end tests, but after each step, the switch pair is paused.
 */

// Send ROCs while switch is paused
test('Send ROCs while switch paused', async ({ expect }) => {
	const { checkSystemEvents } = withExpect(expect)

	const switchParameters = getAssetSwitchParameters()

	// 10 % of relay tokens are used as fees
	const feeAmount = (ROC * BigInt(10)) / BigInt(100)

	const remoteAssetId = { V4: AssetHubConfig.eKiltLocation }
	const remoteXcmFeeId = { V4: { id: AssetHubConfig.nativeTokenLocation, fun: { Fungible: feeAmount } } }
	const remoteReserveLocation = getSiblingLocationV4(AssetHubConfig.paraId)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	await setStorage(
		peregrineContext,
		PeregrineConfig.setSwitchPair(
			switchParameters,
			remoteAssetId,
			remoteXcmFeeId,
			remoteReserveLocation,
			PeregrineConfig.initialPoolAccountId,
			'Paused'
		)
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

	const peregrineDestination = getSiblingLocationV4(PeregrineConfig.paraId)
	const beneficiary1 = getAccountLocationV4(hexAddress(keysAlice.address))
	const rocAsset = { V4: [getRelayNativeAssetIdLocationV4(ROC.toString())] }

	const signedTx1 = assethubContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(peregrineDestination, beneficiary1, rocAsset, 0, 'Unlimited')
		.signAsync(keysAlice)

	await sendTransaction(signedTx1)
	await createBlock(assethubContext)

	// ... But it should fail on Peregrine
	await createBlock(peregrineContext)
	const aliceRocBalance = await getFreeRocPeregrine(keysAlice.address)

	await checkSystemEvents(peregrineContext, { section: 'messageQueue', method: 'Processed' }).toMatchSnapshot(
		'receiver Peregrine::messageQueue::[Processed]'
	)

	await checkSystemEvents(peregrineContext, { section: 'polkadotXcm', method: 'AssetsTrapped' }).toMatchSnapshot(
		'receiver Peregrine::polkadotXcm::[AssetsTrapped]'
	)

	expect(aliceRocBalance).toBe(BigInt(0))

	await checkSwitchPalletInvariant(expect)
}, 30_000)

/**
 * 1. Send Rocs
 * 2. pause switch
 * 3. switch KILTs
 */
test('Switch PILTs against ePILTs while paused', async ({ expect }) => {
	const switchParameters = getAssetSwitchParameters()

	// 10 % of relay tokens are used as fees
	const feeAmount = (ROC * BigInt(10)) / BigInt(100)

	const remoteAssetId = { V4: AssetHubConfig.eKiltLocation }
	const remoteXcmFeeId = { V4: { id: AssetHubConfig.nativeTokenLocation, fun: { Fungible: feeAmount } } }
	const remoteReserveLocation = getSiblingLocationV4(AssetHubConfig.paraId)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion4(),
		...PeregrineConfig.setSudoKey(keysAlice.address),
	})

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
	const beneficiary1 = getAccountLocationV4(hexAddress(keysAlice.address))
	const rocAsset = { V4: [getRelayNativeAssetIdLocationV4(ROC.toString())] }

	const signedTx1 = assethubContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(peregrineDestination, beneficiary1, rocAsset, 0, 'Unlimited')
		.signAsync(keysAlice)

	await sendTransaction(signedTx1)
	await createBlock(assethubContext)

	// process msg.
	await createBlock(peregrineContext)
	const aliceRocBalance = await getFreeRocPeregrine(keysAlice.address)
	expect(aliceRocBalance).toBeGreaterThan(BigInt(0))

	// 2. Pause switch pair
	await peregrineContext.api.tx.sudo
		.sudo(peregrineContext.api.tx.assetSwitchPool1.pauseSwitchPair())
		.signAndSend(keysAlice)
	await createBlock(peregrineContext)

	// 3. switch KILTs
	const balanceToTransfer = initialBalanceKILT / BigInt(2)

	const beneficiary = getAccountLocationV4(hexAddress(keysAlice.address))

	let section: string = ''
	let errorName: string = ''

	// This should fail.
	await peregrineContext.api.tx.assetSwitchPool1
		.switch(balanceToTransfer.toString(), beneficiary)
		.signAndSend(keysAlice, ({ dispatchError }) => {
			if (dispatchError) {
				const decoded = peregrineContext.api.registry.findMetaError(dispatchError.asModule)
				section = decoded.section
				errorName = decoded.name
			}
		})

	await createBlock(peregrineContext)

	expect(section).toBe('assetSwitchPool1')
	expect(errorName).toBe('SwitchPairNotEnabled')

	await checkSwitchPalletInvariant(expect)
}, 30_000)

/**
 * 1. Send Rocs
 * 2. switch KILTs
 * 3. pause switch
 * 4. send eKILTs back
 */
test('Switch ePILTs against PILTs while paused', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	const switchParameters = getAssetSwitchParameters()

	// 10 % of relay tokens are used as fees
	const feeAmount = (ROC * BigInt(10)) / BigInt(100)

	const remoteAssetId = { V4: AssetHubConfig.eKiltLocation }
	const remoteXcmFeeId = { V4: { id: AssetHubConfig.nativeTokenLocation, fun: { Fungible: feeAmount } } }
	const remoteReserveLocation = getSiblingLocationV4(AssetHubConfig.paraId)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion4(),
		...PeregrineConfig.setSudoKey(keysAlice.address),
	})

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
	const beneficiary1 = getAccountLocationV4(hexAddress(keysAlice.address))
	const rocAsset = { V4: [getRelayNativeAssetIdLocationV4(ROC.toString())] }

	const signedTx1 = assethubContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(peregrineDestination, beneficiary1, rocAsset, 0, 'Unlimited')
		.signAsync(keysAlice)
	await sendTransaction(signedTx1)

	// send msg
	await createBlock(assethubContext)
	// process msg.
	await createBlock(peregrineContext)

	const aliceRocBalance = await getFreeRocPeregrine(keysAlice.address)
	expect(aliceRocBalance).toBeGreaterThan(BigInt(0))

	// 2. switch KILTs
	const balanceToTransfer = initialBalanceKILT / BigInt(2)

	const beneficiary = getAccountLocationV4(hexAddress(keysAlice.address))

	const signedTx2 = peregrineContext.api.tx.assetSwitchPool1
		.switch(balanceToTransfer.toString(), beneficiary)
		.signAsync(keysAlice)

	const events2 = await sendTransaction(signedTx2)

	await createBlock(peregrineContext)

	await checkEvents(events2, 'assetSwitchPool1').toMatchSnapshot(
		'sender Peregrine::assetSwitchPool1::[LocalToRemoteSwitchExecuted]'
	)
	await createBlock(assethubContext)

	// only check here, if alice received the funds
	const balanceAliceEkilt = await getFreeEkiltAssetHub(keysAlice.address)
	expect(balanceAliceEkilt).toBe(balanceToTransfer)

	// 3. Pause swap pairs
	await peregrineContext.api.tx.sudo
		.sudo(peregrineContext.api.tx.assetSwitchPool1.pauseSwitchPair())
		.signAndSend(keysAlice)
	await createBlock(peregrineContext)

	// 4. send eKILTs back
	const balanceToTransferBack = balanceToTransfer / BigInt(2)
	const dest = getSiblingLocationV4(PeregrineConfig.paraId)
	const remoteFeeId = { V4: AssetHubConfig.eKiltLocation }
	const funds = {
		V4: [
			{
				id: AssetHubConfig.eKiltLocation,
				fun: { Fungible: balanceToTransferBack.toString() },
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

	await sendTransaction(signedTx3)

	await createBlock(assethubContext)

	// Tx should not fail on AH.
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, KILT * BigInt(25))

	// prcess msg
	await createBlock(peregrineContext)

	// ... but MSG execution should fail on Peregrine
	await checkSystemEvents(peregrineContext, { section: 'messageQueue', method: 'Processed' }).toMatchSnapshot(
		'receiver Peregrine::messageQueue::[Processed]'
	)
	await checkSystemEvents(peregrineContext, { section: 'polkadotXcm', method: 'AssetsTrapped' }).toMatchSnapshot(
		'receiver Peregrine::polkadotXcm::[AssetsTrapped]'
	)

	// The msg will not be processed. Therefore, some assets are not moved. We can not do strict checks here.
	await checkSwitchPalletInvariant(expect, balanceToTransferBack)
}, 30_000)

test('Withdraw ROCs while switch is paused', async ({ expect }) => {
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address]),
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	const switchParameters = getAssetSwitchParameters()
	// 10 % of relay tokens are used as fees
	const feeAmount = (ROC * BigInt(10)) / BigInt(100)

	const remoteAssetId = { V4: AssetHubConfig.eKiltLocation }
	const remoteXcmFeeId = { V4: { id: AssetHubConfig.nativeTokenLocation, fun: { Fungible: feeAmount } } }
	const remoteReserveLocation = getSiblingLocationV4(AssetHubConfig.paraId)

	await setStorage(
		peregrineContext,
		PeregrineConfig.setSwitchPair(
			switchParameters,
			remoteAssetId,
			remoteXcmFeeId,
			remoteReserveLocation,
			PeregrineConfig.initialPoolAccountId,
			'Paused'
		)
	)

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccountsAsStorage(
			[PeregrineConfig.sovereignAccountAsSibling],
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

	let section: string = ''
	let errorName: string = ''

	const assetHubDestination = getSiblingLocationV4(AssetHubConfig.paraId)
	const assets = { V4: [getRelayNativeAssetIdLocationV4(ROC.toString())] }
	const beneficiary = getAccountLocationV4(hexAddress(keysAlice.address))

	await peregrineContext.api.tx.polkadotXcm
		.transferAssets(assetHubDestination, beneficiary, assets, 0, 'Unlimited')
		.signAndSend(keysAlice, ({ dispatchError }) => {
			if (dispatchError) {
				const decoded = peregrineContext.api.registry.findMetaError(dispatchError.asModule)
				section = decoded.section
				errorName = decoded.name
			}
		})

	await createBlock(peregrineContext)

	expect(section).toBe('polkadotXcm')
	expect(errorName).toBe('LocalExecutionIncomplete')

	await checkSwitchPalletInvariant(expect)
}, 30_000)
