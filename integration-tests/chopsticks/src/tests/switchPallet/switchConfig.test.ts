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
	checkSwitchPalletInvariant,
} from '../index.js'
import { checkBalance, createBlock, setStorage, hexAddress, getXcmMessageV4ToSendEkilt } from '../utils.js'
import { getAccountLocationV4, getChildLocation, getSiblingLocationV4 } from '../../network/utils.js'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

test('Switch KILTs against EKILTs no enough DOTs on AH', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	const switchParameters = getAssetSwitchParameters()

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address]),
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(switchParameters))

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts([PeregrineConfig.siblingSovereignAccount], initialBalanceROC),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [
			[PeregrineConfig.siblingSovereignAccount, switchParameters.sovereignSupply],
		]),
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

	// We can only check the soft invariant. On the source chain, the sovereign supply is decreased,
	// while it stays constant on the destination chain.
	await checkSwitchPalletInvariant(expect, true)
}, 20_000)

test('Pool accounts funds goes to zero', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)
	const switchParameters = getAssetSwitchParameters(KILT * BigInt(1000))

	// assign the pool account only 100 KILTs. The pool account gets 10% of the provided total supply.
	await setStorage(peregrineContext, {
		...PeregrineConfig.setSwitchPair(switchParameters),
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	// create foreign asset on assethub and assign Alice more eKILTs then existing
	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [
			// we kinda break the invariant here. This should not bot possible.
			[keysAlice.address, switchParameters.circulatingSupply + BigInt(2) * KILT],
			[PeregrineConfig.siblingSovereignAccount, switchParameters.sovereignSupply],
		]),
	})

	// Check initial state. The pool account should have 100 KILTs + ED.
	await checkBalance(
		getFreeBalancePeregrine,
		PeregrineConfig.initialPoolAccountId,
		expect,
		KILT * BigInt(100) + PeregrineConfig.existentialDeposit
	)
	await checkBalance(
		getFreeEkiltAssetHub,
		keysAlice.address,
		expect,
		switchParameters.circulatingSupply + BigInt(2) * KILT
	)

	// try to dry out the pool account by sending the whole circulating supply + 1 KILT.
	// This should be never possible, as the pool account should always have enough funds.
	const balanceToTransfer = switchParameters.circulatingSupply + KILT

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

	const signedTx = assethubContext.api.tx.polkadotXcm
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

	const events = await sendTransaction(signedTx)

	await createBlock(assethubContext)

	await checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender AssetHub::xcmpQueue::[XcmpMessageSent]')
	await checkEvents(events, { section: 'polkadotXcm', method: 'Attempted' }).toMatchSnapshot(
		'sender AssetHub::polkadotXcm::[Attempted]'
	)
	await checkEvents(events, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'sender AssetHub::foreignAssets::[Transferred]'
	)

	await createBlock(peregrineContext)

	await checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot(
		'receiver Peregrine::messageQueue::[Processed]'
	)
	// AssetTransactor will fail to send the funds. Therefore, the funds will be trapped.
	await checkSystemEvents(peregrineContext, 'polkadotXcm').toMatchSnapshot(
		'receiver Peregrine::polkadotXcm::[AssetsTrapped]'
	)

	await checkSwitchPalletInvariant(expect, true)
}, 20_000)

test('Send eKILT while switch Pair does not exist', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	const switchParameters = getAssetSwitchParameters(initialBalanceKILT * BigInt(1000))

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [
			[keysAlice.address, switchParameters.circulatingSupply],
		]),
	})

	const dest = getSiblingLocationV4(PeregrineConfig.paraId)
	const remoteFeeId = { V4: AssetHubConfig.eKiltLocation }

	const funds = {
		V4: [
			{
				id: AssetHubConfig.eKiltLocation,
				fun: { Fungible: KILT },
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
			getXcmMessageV4ToSendEkilt(keysAlice.address),
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

test('Send eKILT from other reserve location', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	const switchParameters = getAssetSwitchParameters()

	await setStorage(rococoContext, {
		...RococoConfig.setSudoKey(keysAlice.address),
		...RococoConfig.assignNativeTokensToAccounts([keysAlice.address]),
	})

	await setStorage(
		assethubContext,
		AssetHubConfig.createForeignAsset(keysCharlie.address, [
			[PeregrineConfig.siblingSovereignAccount, switchParameters.sovereignSupply],
		])
	)

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(switchParameters))

	const dest = { V3: getChildLocation(PeregrineConfig.paraId) }

	const xcmMessage = {
		V3: [
			{
				ReserveAssetDeposited: [
					{
						id: { Concrete: AssetHubConfig.eKiltLocation },
						fun: { Fungible: initialBalanceKILT },
					},
				],
			},
			'ClearOrigin',
			{
				BuyExecution: {
					fees: {
						id: { Concrete: AssetHubConfig.eKiltLocation },
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

	const innerTx = rococoContext.api.tx.xcmPallet.send(dest, xcmMessage)

	const tx = rococoContext.api.tx.sudo.sudo(innerTx).signAsync(keysAlice)

	const events = await sendTransaction(tx)

	await createBlock(rococoContext)

	// MSG should have been send
	await checkEvents(events, 'xcmPallet').toMatchSnapshot('sender Rococo::xcmPallet::[XcmMessageSent]')

	await createBlock(peregrineContext)

	// We expect the UntrustedReserveLocation error which results in failing the msg. The error will NOT emitted as an event.
	await checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot(
		'receiver Peregrine::messageQueue::[Processed]'
	)

	await checkSwitchPalletInvariant(expect)
}, 20_000)
