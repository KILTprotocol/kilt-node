import { test } from 'vitest'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assetHub.js'
import * as RococoConfig from '../../network/rococo.js'
import {
	KILT,
	ROC,
	getAssetSwitchParameters,
	initialBalanceKILT,
	initialBalanceROC,
	keysAlice,
	keysCharlie,
} from '../../utils.js'
import { peregrineContext, getFreeEkiltAssetHub, assethubContext, getFreeRocAssetHub, rococoContext } from '../index.js'
import {
	checkSwitchPalletInvariant,
	checkBalance,
	createBlock,
	setStorage,
	hexAddress,
	getXcmMessageV4ToSendEkilt,
} from '../utils.js'
import { getAccountLocationV4, getChildLocation, getSiblingLocationV4 } from '../../network/utils.js'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

test('Switch KILTs against EKILTs no enough DOTs on AH', async ({ expect }) => {
	const { checkEvents } = withExpect(expect)

	const switchParameters = getAssetSwitchParameters()

	// 10 % of relay tokens are used as fees
	const feeAmount = (ROC * BigInt(10)) / BigInt(100)

	const remoteAssetId = { V4: AssetHubConfig.eKiltLocation }
	const remoteXcmFeeId = { V4: { id: AssetHubConfig.nativeTokenLocation, fun: { Fungible: feeAmount } } }
	const remoteReserveLocation = getSiblingLocationV4(AssetHubConfig.paraId)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address]),
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	await setStorage(
		peregrineContext,
		PeregrineConfig.setSwitchPair(switchParameters, remoteAssetId, remoteXcmFeeId, remoteReserveLocation)
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

	// process msg. We don't care about the events. We check only the funds.
	await createBlock(assethubContext)

	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, BigInt(0))
	await checkBalance(getFreeRocAssetHub, keysAlice.address, expect, BigInt(0))

	await checkSwitchPalletInvariant(expect, balanceToTransfer)
}, 20_000)

test('Send eKILT while switch Pair does not exist', async ({ expect }) => {
	const { checkSystemEvents } = withExpect(expect)

	const switchParameters = getAssetSwitchParameters(initialBalanceKILT * BigInt(1000))

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccountsAsStorage(
			[keysAlice.address, PeregrineConfig.sovereignAccountAsSibling],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address),
		...AssetHubConfig.assignForeignAssetToAccounts([[keysAlice.address, switchParameters.circulatingSupply]]),
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

	await sendTransaction(signedTx)
	// send msg
	await createBlock(assethubContext)

	// Will fail on the receiver side
	await createBlock(peregrineContext)
	await checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot(
		'receiver Peregrine::messageQueue::[Processed]'
	)
}, 20_000)

test('Send eKILT from other reserve location', async ({ expect }) => {
	const { checkSystemEvents } = withExpect(expect)

	const switchParameters = getAssetSwitchParameters()
	const feeAmount = (ROC * BigInt(10)) / BigInt(100)
	const remoteAssetId = { V4: AssetHubConfig.eKiltLocation }
	const remoteXcmFeeId = { V4: { id: AssetHubConfig.nativeTokenLocation, fun: { Fungible: feeAmount } } }
	const remoteReserveLocation = getSiblingLocationV4(AssetHubConfig.paraId)

	await setStorage(rococoContext, {
		...RococoConfig.setSudoKey(keysAlice.address),
		...RococoConfig.assignNativeTokensToAccounts([keysAlice.address]),
	})

	await setStorage(assethubContext, {
		...AssetHubConfig.createForeignAsset(keysCharlie.address),
	})

	await setStorage(
		assethubContext,
		AssetHubConfig.assignForeignAssetToAccounts([
			[PeregrineConfig.sovereignAccountAsSibling, switchParameters.sovereignSupply],
		])
	)

	await setStorage(
		peregrineContext,
		PeregrineConfig.setSwitchPair(switchParameters, remoteAssetId, remoteXcmFeeId, remoteReserveLocation)
	)

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

	// send msg
	await sendTransaction(tx)
	await createBlock(rococoContext)

	await createBlock(peregrineContext)
	// We expect the UntrustedReserveLocation error which results in failing the msg. The error will NOT emitted as an event.
	await checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot(
		'receiver Peregrine::messageQueue::[Processed]'
	)

	await checkSwitchPalletInvariant(expect)
}, 20_000)
