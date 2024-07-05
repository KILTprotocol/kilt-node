import { test } from 'vitest'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assethub.js'
import * as BasiliskConfig from '../../network/basilisk.js'
import * as RococoConfig from '../../network/rococo.js'
import { initialBalanceKILT, initialBalanceROC, keysAlice, keysBob, keysCharlie } from '../../utils.js'
import {
	peregrineContext,
	getFreeBalancePeregrine,
	getFreeRocPeregrine,
	getFreeEkiltAssetHub,
	assethubContext,
	getFreeRocAssetHub,
	basiliskContext,
	rococoContext,
} from '../index.js'
import { checkBalance, createBlock, setStorage, hexAddress } from '../utils.js'
import { getAccountLocationV3, getChildLocation, getNativeAssetIdLocation } from '../../network/utils.js'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

test.skip('Switch PILTS against EPILTS not same user', async ({ expect }) => {
	// Assign alice some KILT and ROC tokens
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

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, initialBalanceKILT)
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)

	// Alice should have NO eKILT on AH
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, BigInt(0))

	// 50 PILTS
	const balanceToTransfer = BigInt('50000000000000000')

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

	// After creating a new block, the tx should be finalized
	await createBlock(peregrineContext)

	expect(section).toBe('assetSwitchPool1')
	expect(errorName).toBe('Hook')

	// Check sender state
}, 20_000)

test.skip('Switch PILTS against EPILTS user has not enough balance', async ({ expect }) => {
	// Assign alice some KILT and ROC tokens
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

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, initialBalanceKILT)
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)

	// Alice should have NO eKILT on AH
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, BigInt(0))

	// 500 PILTS
	const balanceToTransfer = BigInt('500000000000000000')

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
	expect(errorName).toBe('UserSwitchBalance')

	// Check sender state
}, 20_000)

test.skip('Switch PILTS against EPILTS not enough pool account balance', async ({ expect }) => {
	// Assign alice some KILT and ROC tokens
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT * BigInt(1000)),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	// create swtich pair and give pool account less coins
	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(initialBalanceKILT))

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, initialBalanceKILT * BigInt(1000))
	await checkBalance(getFreeBalancePeregrine, PeregrineConfig.initialPoolAccountId, expect, initialBalanceKILT)
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)

	// Alice should have NO eKILT on AH
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, BigInt(0))

	// 200 PILTS
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
				console.log(section, errorName)
			}
		})

	// After creating a new block, the tx should be finalized
	await createBlock(peregrineContext)

	expect(section).toBe('assetSwitchPool1')
	expect(errorName).toBe('Liquidity')
}, 20_000)

test.skip('Switch PILTS against EPILTS user has no DOTs', async ({ expect }) => {
	// Assign alice some KILT and ROC tokens
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	// create switch pair and give pool account less coins
	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair())

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, initialBalanceKILT)
	await checkBalance(
		getFreeBalancePeregrine,
		PeregrineConfig.initialPoolAccountId,
		expect,
		PeregrineConfig.initialRemoteFeeAssetBalance
	)
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))

	// Alice should have NO eKILT on AH
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, BigInt(0))

	// 50 PILTS
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
				console.log(section, errorName)
			}
		})

	// After creating a new block, the tx should be finalized
	await createBlock(peregrineContext)

	expect(section).toBe('assetSwitchPool1')
	expect(errorName).toBe('UserXcmBalance')
}, 20_000)

test.skip('Switch PILTS against EPILTS no SwitchPair', async ({ expect }) => {
	// Assign alice some KILT and ROC tokens
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

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, initialBalanceKILT)
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)

	// Alice should have NO eKILT on AH
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, BigInt(0))

	// 50 PILTS
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
				console.log(section, errorName)
			}
		})

	// After creating a new block, the tx should be finalized
	await createBlock(peregrineContext)

	expect(section).toBe('assetSwitchPool1')
	expect(errorName).toBe('SwitchPairNotFound')
}, 20_000)

test.skip('Switch PILTS against EPILTS no enough DOTs on AH', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Assign alice some KILT and ROC tokens
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

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, initialBalanceKILT)
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)

	// Alice should have NO eKILT on AH
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, BigInt(0))
	await checkBalance(getFreeRocAssetHub, keysAlice.address, expect, BigInt(0))

	// 50 PILTS
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

	//await assethubContext.pause()
}, 20_00000)

// Is failing: Todo: Fix XCM config
test.skip('Send DOTs from Relay 2 Peregrine', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Assign alice some KILTs
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair())

	// Assigned Alice some ROCs and HDX on Basilisk
	await setStorage(rococoContext, RococoConfig.assignNativeTokensToAccounts([keysAlice.address]))

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, initialBalanceKILT)
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))

	// 50 ROCs
	const balanceToTransfer = initialBalanceROC / BigInt(2)

	const aliceAddress = hexAddress(keysAlice.address)
	const hydraDxDestination = { V3: getChildLocation(PeregrineConfig.paraId) }
	const beneficiary = getAccountLocationV3(aliceAddress)
	const assetToTransfer = { V3: [getNativeAssetIdLocation(balanceToTransfer)] }

	const signedTx = rococoContext.api.tx.xcmPallet
		.limitedReserveTransferAssets(hydraDxDestination, beneficiary, assetToTransfer, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	await createBlock(rococoContext)

	// MSG should be successfully send
	checkEvents(events, 'xcmPallet').toMatchSnapshot('sender events xcmPallet')

	await createBlock(peregrineContext)

	// messageQueue should not successfully execute the msg
	checkSystemEvents(peregrineContext, {
		section: 'parachainSystem',
		method: 'DownwardMessagesReceived',
	}).toMatchSnapshot('receiver events parachainSystem pallet')

	checkSystemEvents(peregrineContext, {
		section: 'dmpQueue',
		method: 'ExecutedDownward',
	}).toMatchSnapshot('receiver events dmpQueue pallet')

	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))
}, 20_000)

// Is failing: Todo: Fix XCM config
test.skip('Send DOTs from basilisk 2 Peregrine', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Assign alice some KILTs
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair())

	// Assigned Alice some ROCs and HDX on Basilisk
	await setStorage(basiliskContext, {
		...BasiliskConfig.assignNativeTokensToAccounts([keysAlice.address]),
		...BasiliskConfig.assignRocTokensToAccounts([keysAlice.address], initialBalanceROC),
	})

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, initialBalanceKILT)
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))

	// 50 ROCs
	const balanceToTransfer = initialBalanceROC / BigInt(2)

	const beneficiary = {
		V3: {
			parents: 1,
			interior: {
				X2: [
					{ Parachain: PeregrineConfig.paraId },
					{
						AccountId32: {
							id: hexAddress(keysAlice.address),
						},
					},
				],
			},
		},
	}

	const signedTx = basiliskContext.api.tx.xTokens
		.transfer(BasiliskConfig.dotTOkenId, balanceToTransfer, beneficiary, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	await createBlock(basiliskContext)

	// MSG should be successfully send
	checkEvents(events, 'xTokens').toMatchSnapshot('sender events xTokens')
	// tokens have to be withdrawn
	checkEvents(events, 'tokens').toMatchSnapshot('sender events tokens')
	// An upward message should have been sent
	checkEvents(events, 'parachainSystem').toMatchSnapshot('sender events tokens')

	await createBlock(rococoContext)

	// messageQueue should not successfully execute the msg
	checkSystemEvents(rococoContext, 'messageQueue').toMatchSnapshot('relayer events messageQueue')
	// Balance should be moved to peregrine sovereign account
	checkSystemEvents(rococoContext, { section: 'balances', method: 'Minted' }).toMatchSnapshot(
		'relayer events balances minted'
	)
	// Balance should be burned from the basilisk sovereign account
	checkSystemEvents(rococoContext, { section: 'balances', method: 'Burned' }).toMatchSnapshot(
		'relayer events balances Burned'
	)

	await createBlock(peregrineContext)

	// We should still receive the message
	checkSystemEvents(peregrineContext, {
		section: 'parachainSystem',
		method: 'DownwardMessagesReceived',
	}).toMatchSnapshot('receiver events parachainSystem pallet')

	// ... But msg execution should fail
	checkSystemEvents(peregrineContext, {
		section: 'dmpQueue',
		method: 'ExecutedDownward',
	}).toMatchSnapshot('receiver events dmpQueue')

	// // Alice should still have zero balance
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))
}, 20_000)

test.skip('User gets dusted with ROCs', async ({ expect }) => {
	const { checkEvents } = withExpect(expect)

	// Assign alice some KILTs and ROCs
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address]),
	})

	// 99.9998% of the balance
	const balanceToTransfer = (initialBalanceKILT * BigInt(999998)) / BigInt(1000000)

	// Send all coins to Bob
	const signedTx = peregrineContext.api.tx.balances
		.transferAllowDeath(keysBob.address, balanceToTransfer)
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	await createBlock(peregrineContext)

	checkEvents(events, { section: 'balances', method: 'Transfer' }).toMatchSnapshot('balances transfer event')
	// User should get dusted by this operation
	checkEvents(events, { section: 'balances', method: 'DustLost' }).toMatchSnapshot('balances transfer event')

	// ... But alice's ROC funds should still exist
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)
}, 20_000)
