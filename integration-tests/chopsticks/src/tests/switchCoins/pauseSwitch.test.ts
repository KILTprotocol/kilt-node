import { test } from 'vitest'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assethub.js'
import { KILT, ROC, initialBalanceKILT, initialBalanceROC, keysAlice, keysBob, keysCharlie } from '../../utils.js'
import {
	peregrineContext,
	getFreeBalancePeregrine,
	getFreeRocPeregrine,
	getFreeEkiltAssetHub,
	assethubContext,
} from '../index.js'
import { checkBalance, createBlock, setStorage, hexAddress, checkBalanceInRange } from '../utils.js'
import { getAccountLocationV3, getRelayNativeAssetIdLocation, getSiblingLocation } from '../../network/utils.js'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

test.skip('Full e2e tests', async ({ expect }) => {
	const { checkEvents } = withExpect(expect)

	// Assign alice some KILT and ROC tokens
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair())

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	// 1. send ROCs 2 Peregrine

	const peregrineDestination = { V3: getSiblingLocation(PeregrineConfig.paraId) }
	const beneficiary1 = getAccountLocationV3(hexAddress(keysAlice.address))
	const rocAsset = { V3: [getRelayNativeAssetIdLocation(ROC)] }

	const signedTx1 = assethubContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(peregrineDestination, beneficiary1, rocAsset, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events1 = await sendTransaction(signedTx1)

	await createBlock(assethubContext)

	// Check events sender
	await checkEvents(events1, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	await checkEvents(events1, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	await checkEvents(events1, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	await createBlock(peregrineContext)

	const aliceRocBalance = await getFreeRocPeregrine(keysAlice.address)

	// just check if the balance is greater than 0
	expect(aliceRocBalance).toBeGreaterThan(BigInt(0))

	// 2. switch KILTs

	const balanceToTransfer = initialBalanceKILT / BigInt(2)

	const beneficiary = getAccountLocationV3(hexAddress(keysAlice.address))

	const signedTx2 = peregrineContext.api.tx.assetSwitchPool1
		.switch(balanceToTransfer.toString(), beneficiary)
		.signAsync(keysAlice)

	const events2 = await sendTransaction(signedTx2)

	await createBlock(peregrineContext)

	// Just check if switch is executed
	await checkEvents(events2, 'assetSwitchPool1').toMatchSnapshot('Switch events assetSwitchPool pallet')

	await createBlock(assethubContext)

	// Just check if Alice has some eKILTs now
	const balanceAliceEkilt = await getFreeEkiltAssetHub(keysAlice.address)
	expect(balanceAliceEkilt).toBe(balanceToTransfer)

	// 3. send eKILTs back

	const dest = { V3: getSiblingLocation(PeregrineConfig.paraId) }

	const remoteFeeId = { V3: { Concrete: AssetHubConfig.eKiltLocation } }

	const funds = {
		V3: [
			{
				id: { Concrete: AssetHubConfig.eKiltLocation },
				fun: { Fungible: balanceToTransfer / BigInt(2) },
			},
		],
	}

	const xcmMessage = {
		V3: [
			{
				DepositAsset: {
					assets: { Wild: 'All' },
					beneficiary: {
						parents: 0,
						interior: {
							X1: {
								AccountId32: {
									id: hexAddress(keysAlice.address),
								},
							},
						},
					},
				},
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
			xcmMessage,
			'Unlimited'
		)
		.signAsync(keysAlice)

	const events3 = await sendTransaction(signedTx3)

	await createBlock(assethubContext)

	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, KILT * BigInt(25))

	// Just check if assets are transferred back
	await checkEvents(events3, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'Sending eKILTs back'
	)

	await createBlock(peregrineContext)

	// Alice should have her KILTs back
	await checkBalanceInRange(getFreeBalancePeregrine, keysAlice.address, expect, [
		BigInt(74) * KILT,
		BigInt(75) * KILT,
	])

	// 4. send ROCs back TODO: implement
}, 20_000)

test('full e2e. Send DOTs while switch paused', async ({ expect }) => {
	const { checkEvents } = withExpect(expect)

	// Assign alice some KILT and ROC tokens
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	await setStorage(
		peregrineContext,
		PeregrineConfig.setSwitchPair(
			PeregrineConfig.initialRemoteAssetBalance,
			PeregrineConfig.initialPoolAccountId,
			'Paused'
		)
	)

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	const peregrineDestination = { V3: getSiblingLocation(PeregrineConfig.paraId) }
	const beneficiary1 = getAccountLocationV3(hexAddress(keysAlice.address))
	const rocAsset = { V3: [getRelayNativeAssetIdLocation(ROC)] }

	const signedTx1 = assethubContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(peregrineDestination, beneficiary1, rocAsset, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events1 = await sendTransaction(signedTx1)

	await createBlock(assethubContext)

	// Check events sender
	await checkEvents(events1, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	await checkEvents(events1, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	await checkEvents(events1, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	await createBlock(peregrineContext)

	const aliceRocBalance = await getFreeRocPeregrine(keysAlice.address)

	// just check if the balance is greater than 0. Paused switch should not effect sending ROCs
	expect(aliceRocBalance).toBeGreaterThan(BigInt(0))
}, 20_000)

test.skip('Full e2e. Switch PILTs against ePILTs while paused', async ({ expect }) => {
	const { checkEvents } = withExpect(expect)

	// Assign alice some KILT and ROC tokens
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion3(),
		...PeregrineConfig.setSudoKey(keysAlice.address),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair())

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	// 1. send ROCs 2 Peregrine

	const peregrineDestination = { V3: getSiblingLocation(PeregrineConfig.paraId) }
	const beneficiary1 = getAccountLocationV3(hexAddress(keysAlice.address))
	const rocAsset = { V3: [getRelayNativeAssetIdLocation(ROC)] }

	const signedTx1 = assethubContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(peregrineDestination, beneficiary1, rocAsset, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events1 = await sendTransaction(signedTx1)

	await createBlock(assethubContext)

	// Check events sender
	await checkEvents(events1, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	await checkEvents(events1, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	await checkEvents(events1, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	await createBlock(peregrineContext)

	const aliceRocBalance = await getFreeRocPeregrine(keysAlice.address)

	// just check if the balance is greater than 0
	expect(aliceRocBalance).toBeGreaterThan(BigInt(0))

	// Pause switch pair
	await peregrineContext.api.tx.sudo
		.sudo(peregrineContext.api.tx.assetSwitchPool1.pauseSwitchPair())
		.signAndSend(keysAlice)
	await createBlock(peregrineContext)

	// 2. switch KILTs

	const balanceToTransfer = initialBalanceKILT / BigInt(2)

	const beneficiary = getAccountLocationV3(hexAddress(keysAlice.address))

	let section: string = ''
	let errorName: string = ''

	await peregrineContext.api.tx.assetSwitchPool1
		.switch(balanceToTransfer.toString(), beneficiary)
		.signAndSend(keysAlice, ({ dispatchError }) => {
			if (dispatchError) {
				const decoded = peregrineContext.api.registry.findMetaError(dispatchError.asModule)
				section = decoded.section
				errorName = decoded.name
			}
		})

	// After creating a new block, the tx should be finalized
	await createBlock(peregrineContext)

	expect(section).toBe('assetSwitchPool1')
	expect(errorName).toBe('SwitchPairNotEnabled')
}, 20_000)

test('full e2e. Switch ePILTs agains PILTs while paused', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Assign alice some KILT and ROC tokens
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion3(),
		...PeregrineConfig.setSudoKey(keysAlice.address),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair())

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	// 1. send ROCs 2 Peregrine

	const peregrineDestination = { V3: getSiblingLocation(PeregrineConfig.paraId) }
	const beneficiary1 = getAccountLocationV3(hexAddress(keysAlice.address))
	const rocAsset = { V3: [getRelayNativeAssetIdLocation(ROC)] }

	const signedTx1 = assethubContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(peregrineDestination, beneficiary1, rocAsset, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events1 = await sendTransaction(signedTx1)

	await createBlock(assethubContext)

	// Check events sender
	await checkEvents(events1, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	await checkEvents(events1, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	await checkEvents(events1, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	await createBlock(peregrineContext)

	const aliceRocBalance = await getFreeRocPeregrine(keysAlice.address)

	// just check if the balance is greater than 0
	expect(aliceRocBalance).toBeGreaterThan(BigInt(0))

	// 2. switch KILTs

	const balanceToTransfer = initialBalanceKILT / BigInt(2)

	const beneficiary = getAccountLocationV3(hexAddress(keysAlice.address))

	const signedTx2 = peregrineContext.api.tx.assetSwitchPool1
		.switch(balanceToTransfer.toString(), beneficiary)
		.signAsync(keysAlice)

	const events2 = await sendTransaction(signedTx2)

	await createBlock(peregrineContext)

	// Just check if switch is executed
	await checkEvents(events2, 'assetSwitchPool1').toMatchSnapshot('Switch events assetSwitchPool pallet')

	await createBlock(assethubContext)

	// Just check if Alice has some eKILTs now
	const balanceAliceEkilt = await getFreeEkiltAssetHub(keysAlice.address)
	expect(balanceAliceEkilt).toBe(balanceToTransfer)

	// Pause swap pairs
	await peregrineContext.api.tx.sudo
		.sudo(peregrineContext.api.tx.assetSwitchPool1.pauseSwitchPair())
		.signAndSend(keysAlice)
	await createBlock(peregrineContext)

	// 3. send eKILTs back

	const dest = { V3: getSiblingLocation(PeregrineConfig.paraId) }

	const remoteFeeId = { V3: { Concrete: AssetHubConfig.eKiltLocation } }

	const funds = {
		V3: [
			{
				id: { Concrete: AssetHubConfig.eKiltLocation },
				fun: { Fungible: balanceToTransfer / BigInt(2) },
			},
		],
	}

	const xcmMessage = {
		V3: [
			{
				DepositAsset: {
					assets: { Wild: 'All' },
					beneficiary: {
						parents: 0,
						interior: {
							X1: {
								AccountId32: {
									id: hexAddress(keysAlice.address),
								},
							},
						},
					},
				},
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
			xcmMessage,
			'Unlimited'
		)
		.signAsync(keysAlice)

	const events3 = await sendTransaction(signedTx3)

	await createBlock(assethubContext)

	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, KILT * BigInt(25))

	// Just check if assets are transferred back
	await checkEvents(events3, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'Sending eKILTs back'
	)

	await createBlock(peregrineContext)

	await checkSystemEvents(peregrineContext, { section: 'xcmpQueue', method: 'Fail' }).toMatchSnapshot(
		'xcmpQueue Sending eKILTs back while switch is paused'
	)

	await checkSystemEvents(peregrineContext, { section: 'polkadotXcm', method: 'AssetsTrapped' }).toMatchSnapshot(
		'PolkadotXCM Sending eKILTs back while switch is paused'
	)
}, 20_000)
