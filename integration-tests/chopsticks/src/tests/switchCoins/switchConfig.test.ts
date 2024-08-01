import { test } from 'vitest'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assetHub.js'
import * as RococoConfig from '../../network/rococo.js'
import {
	KILT,
	getAssetSwitchParameters,
	initialBalanceKILT,
	initialBalanceROC,
	keysAlice,
	keysBob,
	keysCharlie,
} from '../../utils.js'
import {
	peregrineContext,
	getFreeBalancePeregrine,
	getFreeEkiltAssetHub,
	assethubContext,
	getFreeRocAssetHub,
	rococoContext,
} from '../index.js'
import { checkBalance, createBlock, setStorage, hexAddress } from '../utils.js'
import { getAccountLocationV4, getChildLocation, getSiblingLocationV4 } from '../../network/utils.js'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

test('Switch KILTs against EKILTs not same user', async ({ expect }) => {
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(getAssetSwitchParameters()))

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	const balanceToTransfer = initialBalanceKILT / BigInt(2)
	const beneficiary = getAccountLocationV4(hexAddress(keysBob.address))

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
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(getAssetSwitchParameters()))

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	const balanceToTransfer = initialBalanceKILT * BigInt(2)
	const beneficiary = getAccountLocationV4(hexAddress(keysAlice.address))

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
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(getAssetSwitchParameters(KILT * BigInt(1000))))

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	// try to send 10_000 KILTs. The pool account should have less
	const balanceToTransfer = KILT * BigInt(10000)

	const beneficiary = getAccountLocationV4(hexAddress(keysAlice.address))

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
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(getAssetSwitchParameters()))

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	const balanceToTransfer = initialBalanceKILT / BigInt(2)

	const beneficiary = getAccountLocationV4(hexAddress(keysAlice.address))

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
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	const balanceToTransfer = initialBalanceKILT / BigInt(2)

	const beneficiary = getAccountLocationV4(hexAddress(keysAlice.address))

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
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(getAssetSwitchParameters()))

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts([PeregrineConfig.siblingSovereignAccount], initialBalanceROC),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	const balanceToTransfer = initialBalanceKILT / BigInt(2)

	const beneficiary = getAccountLocationV4(hexAddress(keysAlice.address))

	const signedTx = peregrineContext.api.tx.assetSwitchPool1
		.switch(balanceToTransfer.toString(), beneficiary)
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	await createBlock(peregrineContext)

	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender Peregrine::xcmpQueue::[XcmpMessageSent]')
	checkEvents(events, 'assetSwitchPool1').toMatchSnapshot(
		'sender Peregrine::assetSwitchPool1::[LocalToRemoteSwitchExecuted]'
	)
	checkEvents(events, { section: 'balances', method: 'Transfer' }).toMatchSnapshot(
		'sender Peregrine::balances::[Transfer]'
	)

	// Strange behavior here... After creating one block another block with a transfer tx is created. The new block is messing up with the checks. We reset the head here
	await createBlock(assethubContext)

	// messageQueue should not successfully execute the msg
	await checkSystemEvents(assethubContext, 'messageQueue').toMatchSnapshot(
		'receiver AssetHub::messageQueue::[Processed]'
	)
	// Refunded fees should be trapped
	await checkSystemEvents(assethubContext, 'polkadotXcm').toMatchSnapshot(
		'receiver AssetHub::polkadotXcm::[AssetsTrapped]'
	)

	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, BigInt(0))
	await checkBalance(getFreeRocAssetHub, keysAlice.address, expect, BigInt(0))
}, 20_000)

test('Pool accounts funds goes to zero', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// assign the pool account only 100 KILTs. The pool account gets 10% of the provided total supply.
	await setStorage(peregrineContext, {
		...PeregrineConfig.setSwitchPair(getAssetSwitchParameters(KILT * BigInt(1000))),
		...PeregrineConfig.setSafeXcmVersion4(),
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

	// Check initial state. The pool account should have 100 KILTs + ED.
	checkBalance(
		getFreeBalancePeregrine,
		PeregrineConfig.initialPoolAccountId,
		expect,
		KILT * BigInt(100) + PeregrineConfig.existentialDeposit
	)
	checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, initialBalanceKILT * BigInt(1000))

	// try to dry out the pool account
	const balanceToTransfer = KILT * BigInt(100) + PeregrineConfig.existentialDeposit

	const dest = { V4: getSiblingLocationV4(PeregrineConfig.paraId) }

	const remoteFeeId = { V4: AssetHubConfig.eKiltLocation }

	const funds = {
		V4: [
			{
				id: AssetHubConfig.eKiltLocation,
				fun: { Fungible: balanceToTransfer },
			},
		],
	}

	const xcmMessage = {
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
										id: hexAddress(keysAlice.address),
									},
								},
							],
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

	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender AssetHub::xcmpQueue::[XcmpMessageSent]')
	checkEvents(events, { section: 'polkadotXcm', method: 'Attempted' }).toMatchSnapshot(
		'sender AssetHub::polkadotXcm::[Attempted]'
	)
	checkEvents(events, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'sender AssetHub::foreignAssets::[Transferred]'
	)

	await createBlock(peregrineContext)

	await checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot(
		'receiver Peregrine::messageQueue::[Processed]'
	)
	await checkSystemEvents(peregrineContext, 'polkadotXcm').toMatchSnapshot(
		'receiver Peregrine::polkadotXcm::[AssetsTrapped]'
	)
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
	const dest = { V4: getSiblingLocationV4(PeregrineConfig.paraId) }
	const remoteFeeId = { V4: AssetHubConfig.eKiltLocation }

	const funds = {
		V4: [
			{
				id: AssetHubConfig.eKiltLocation,
				fun: { Fungible: balanceToTransfer },
			},
		],
	}

	const xcmMessage = {
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
										id: hexAddress(keysAlice.address),
									},
								},
							],
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
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender AssetHub::xcmpQueue::[XcmpMessageSent]')
	checkEvents(events, { section: 'polkadotXcm', method: 'Attempted' }).toMatchSnapshot(
		'sender AssetHub::polkadotXcm::[Attempted]'
	)
	checkEvents(events, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'sender AssetHub::foreignAssets::[Transferred]'
	)

	// Will fail on the receiver side
	await createBlock(peregrineContext)
	await checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot(
		'receiver Peregrine::messageQueue::[Processed]'
	)
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
	const dest = { V4: getSiblingLocationV4(PeregrineConfig.paraId) }
	const remoteFeeId = { V4: AssetHubConfig.eKiltLocation }
	const funds = {
		V4: [
			{
				id: AssetHubConfig.eKiltLocation,
				fun: { Fungible: balanceToTransfer },
			},
		],
	}

	const xcmMessage = {
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
										id: hexAddress(keysAlice.address),
									},
								},
							],
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

test('Send eKILT from other reserve location', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	await setStorage(rococoContext, {
		...RococoConfig.setSudoKey(keysAlice.address),
		...RococoConfig.assignNativeTokensToAccounts([keysAlice.address]),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(getAssetSwitchParameters()))

	const dest = { V4: getChildLocation(PeregrineConfig.paraId) }

	const xcmMessage = {
		V4: [
			{
				ReserveAssetDeposited: [
					{
						id: AssetHubConfig.eKiltLocation,
						fun: { Fungible: initialBalanceKILT },
					},
				],
			},
			'ClearOrigin',
			{
				BuyExecution: {
					fees: {
						id: AssetHubConfig.eKiltLocation,
						fun: { Fungible: initialBalanceKILT },
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
							X1: [
								{
									AccountId32: {
										id: hexAddress(keysAlice.address),
									},
								},
							],
						},
					},
				},
			},
		],
	}

	const innerTx = rococoContext.api.tx.xcmPallet.send(dest, xcmMessage)

	const tx = rococoContext.api.tx.sudo.sudo(innerTx).signAsync(keysAlice)

	const events = await sendTransaction(tx)

	await createBlock(rococoContext)

	// MSG should have been send
	await checkEvents(events, 'xcmPallet').toMatchSnapshot('sender Rococo::xcmPallet::[XcmMessageSent]')

	await createBlock(peregrineContext)

	// We expect the UntrustedReserveLocation error
	await checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot(
		'receiver Peregrine::messageQueue::[Processed]'
	)
}, 20_000)
