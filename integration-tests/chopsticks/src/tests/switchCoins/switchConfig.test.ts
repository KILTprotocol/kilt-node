import { test } from 'vitest'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assethub.js'
import { initialBalanceKILT, initialBalanceROC, keysAlice, keysBob, keysCharlie } from '../../utils.js'
import {
	peregrineContext,
	getFreeBalancePeregrine,
	getFreeEkiltAssetHub,
	assethubContext,
	getFreeRocAssetHub,
} from '../index.js'
import { checkBalance, createBlock, setStorage, hexAddress } from '../utils.js'
import { getAccountLocationV3, getSiblingLocation } from '../../network/utils.js'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

test('Switch KILTs against EKILTs not same user', async ({ expect }) => {
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
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

	const balanceToTransfer = initialBalanceKILT / BigInt(2)
	const beneficiary = getAccountLocationV3(hexAddress(keysBob.address))

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

	await createBlock(peregrineContext)

	expect(section).toBe('assetSwitchPool1')
	expect(errorName).toBe('Hook')
}, 20_000)

test('Switch KILTs against EKILTs user has not enough balance', async ({ expect }) => {
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
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

	const balanceToTransfer = initialBalanceKILT * BigInt(2)
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

	await createBlock(peregrineContext)

	expect(section).toBe('assetSwitchPool1')
	expect(errorName).toBe('UserSwitchBalance')
}, 20_000)

test('Switch KILTs against EKILTs not enough pool account balance', async ({ expect }) => {
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT * BigInt(1000)),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	// assign the pool account only 100 KILTs
	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(initialBalanceKILT))

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	const balanceToTransfer = initialBalanceKILT * BigInt(2)

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

	await createBlock(peregrineContext)

	expect(section).toBe('assetSwitchPool1')
	expect(errorName).toBe('Liquidity')
}, 20_000)

test('Switch KILTs against EKILTs user has no DOTs', async ({ expect }) => {
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

	await createBlock(peregrineContext)

	expect(section).toBe('assetSwitchPool1')
	expect(errorName).toBe('UserXcmBalance')
}, 20_000)

test('Switch KILTs against EKILTs no SwitchPair', async ({ expect }) => {
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address]),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

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

	await createBlock(peregrineContext)

	expect(section).toBe('assetSwitchPool1')
	expect(errorName).toBe('SwitchPairNotFound')
}, 20_000)

test('Switch KILTs against EKILTs no enough DOTs on AH', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address]),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair())

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts([PeregrineConfig.siblingSovereignAccount], initialBalanceROC),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	const balanceToTransfer = initialBalanceKILT / BigInt(2)

	const beneficiary = getAccountLocationV3(hexAddress(keysAlice.address))

	const signedTx = peregrineContext.api.tx.assetSwitchPool1
		.switch(balanceToTransfer.toString(), beneficiary)
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	await createBlock(peregrineContext)

	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'assetSwitchPool1').toMatchSnapshot('Switch events assetSwitchPool1 pallet')
	checkEvents(events, { section: 'balances', method: 'Transfer' }).toMatchSnapshot('sender events Balances')

	await createBlock(assethubContext)

	// messageQueue should not successfully execute the msg
	checkSystemEvents(assethubContext, 'messageQueue').toMatchSnapshot('receiver events xcm queue pallet')
	// Refunded fees should be trapped
	checkSystemEvents(assethubContext, 'polkadotXcm').toMatchSnapshot('receiver events polkadotXcm')

	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, BigInt(0))
	await checkBalance(getFreeRocAssetHub, keysAlice.address, expect, BigInt(0))
}, 20_000)

//FIX ME: This test is failing
test('Pool accounts funds goes to zero', async ({ expect }) => {
	const { checkEvents } = withExpect(expect)

	// Setup switch Pair.
	await setStorage(peregrineContext, {
		...PeregrineConfig.setSwitchPair(initialBalanceKILT),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	// create foreign asset on assethub and assign Alice more eKILTs then existing
	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(
			keysCharlie.address,
			[keysAlice.address],
			initialBalanceKILT * BigInt(1000)
		),
	})

	// Check initial state
	checkBalance(getFreeBalancePeregrine, PeregrineConfig.initialPoolAccountId, expect, initialBalanceKILT)
	checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, initialBalanceKILT * BigInt(1000))

	// try to dry out the pool account
	const balanceToTransfer = initialBalanceKILT

	const dest = { V3: getSiblingLocation(PeregrineConfig.paraId) }

	const remoteFeeId = { V3: { Concrete: AssetHubConfig.eKiltLocation } }

	const funds = {
		V3: [
			{
				id: { Concrete: AssetHubConfig.eKiltLocation },
				fun: { Fungible: balanceToTransfer },
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

	const signedTx = assethubContext.api.tx.polkadotXcm
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

	const events = await sendTransaction(signedTx)

	await createBlock(assethubContext)

	checkEvents(events, 'xcmpQueue').toMatchSnapshot('assetHubs events xcm queue pallet')
	checkEvents(events, { section: 'polkadotXcm', method: 'Attempted' }).toMatchSnapshot('PolkadotXcm assethub')
	checkEvents(events, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'sender events foreignAssets'
	)

	await createBlock(peregrineContext)

	// TODO FIX me
	// It is expected that the FailedToTransactAsset error is been thrown.

	// checkSystemEvents(peregrineContext, { section: 'xcmpQueue', method: 'Fail' }).toMatchSnapshot(
	// 	'receiver events xcm queue pallet'
	// )
}, 20_000)

test('Send eKILT while switch Pair does not exist', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(
			keysCharlie.address,
			[keysAlice.address],
			initialBalanceKILT * BigInt(1000)
		),
	})

	const balanceToTransfer = initialBalanceKILT
	const dest = { V3: getSiblingLocation(PeregrineConfig.paraId) }
	const remoteFeeId = { V3: { Concrete: AssetHubConfig.eKiltLocation } }

	const funds = {
		V3: [
			{
				id: { Concrete: AssetHubConfig.eKiltLocation },
				fun: { Fungible: balanceToTransfer },
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

	const signedTx = assethubContext.api.tx.polkadotXcm
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

	const events = await sendTransaction(signedTx)

	await createBlock(assethubContext)
	// We should still be able to send the msg
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('assetHubs events xcm queue pallet')
	checkEvents(events, { section: 'polkadotXcm', method: 'Attempted' }).toMatchSnapshot('PolkadotXcm assethub')
	checkEvents(events, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'sender events foreignAssets'
	)

	// Will fail on the receiver side
	await createBlock(peregrineContext)
	await checkSystemEvents(peregrineContext, { section: 'xcmpQueue', method: 'Fail' }).toMatchSnapshot(
		'receiver events xcm queue pallet'
	)
	await checkSystemEvents(peregrineContext, 'assetSwitchPool1').toMatchSnapshot('receiver events switch pallet')
}, 20_000)

test('User has no eKILT', async ({ expect }) => {
	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [keysAlice.address], initialBalanceKILT),
	})

	const balanceToTransfer = initialBalanceKILT * BigInt(2)
	const dest = { V3: getSiblingLocation(PeregrineConfig.paraId) }
	const remoteFeeId = { V3: { Concrete: AssetHubConfig.eKiltLocation } }
	const funds = {
		V3: [
			{
				id: { Concrete: AssetHubConfig.eKiltLocation },
				fun: { Fungible: balanceToTransfer },
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

	let section: string = ''
	let errorName: string = ''
	// Execution will fail on the sender side
	await assethubContext.api.tx.polkadotXcm
		.transferAssetsUsingTypeAndThen(
			dest,
			funds,
			'LocalReserve',
			remoteFeeId,
			'LocalReserve',
			xcmMessage,
			'Unlimited'
		)
		.signAndSend(keysAlice, ({ dispatchError }) => {
			if (dispatchError) {
				const decoded = assethubContext.api.registry.findMetaError(dispatchError.asModule)

				section = decoded.section
				errorName = decoded.name
			}
		})

	await createBlock(assethubContext)

	expect(section).toBe('polkadotXcm')
	expect(errorName).toBe('LocalExecutionIncomplete')
}, 20_000)
