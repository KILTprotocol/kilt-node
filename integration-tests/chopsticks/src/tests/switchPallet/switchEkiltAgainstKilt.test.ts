import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assetHub.js'
import { KILT, getAssetSwitchParameters, initialBalanceROC, keysAlice, keysCharlie } from '../../utils.js'
import {
	peregrineContext,
	getFreeBalancePeregrine,
	getFreeRocPeregrine,
	getFreeEkiltAssetHub,
	assethubContext,
	getRemoteLockedSupply,
	checkSwitchPalletInvariant,
} from '../index.js'
import { checkBalance, createBlock, setStorage, getXcmMessageV4ToSendEkilt, checkBalanceInRange } from '../utils.js'
import { getSiblingLocationV4 } from '../../network/utils.js'

test('Switch ePILTs against PILTS on Peregrine', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	const switchParameters = getAssetSwitchParameters()
	// alice has the whole circulating supply.
	const fundsAlice = switchParameters.circulatingSupply

	await setStorage(peregrineContext, {
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
		...PeregrineConfig.setSwitchPair(switchParameters),
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [
			[PeregrineConfig.siblingSovereignAccount, switchParameters.sovereignSupply],
			[keysAlice.address, fundsAlice],
		]),
	})

	// check initial balance of Alice on Spiritnet. Alice should have 0 KILT
	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, BigInt(0))
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, switchParameters.circulatingSupply)

	const initialBalancePoolAccount = await getFreeBalancePeregrine(PeregrineConfig.initialPoolAccountId)
	const initialBalanceSovereignAccount = await getFreeEkiltAssetHub(PeregrineConfig.siblingSovereignAccount)
	const initialRemoteLockedSupply = await getRemoteLockedSupply()

	// 50 PILTS
	const balanceToTransfer = BigInt('50000000000000000')

	const dest = getSiblingLocationV4(PeregrineConfig.paraId)

	const remoteFeeId = { V4: AssetHubConfig.eKiltLocation }

	const funds = {
		V4: [
			{
				id: AssetHubConfig.eKiltLocation,
				fun: { Fungible: balanceToTransfer.toString() },
			},
		],
	}

	const signedTx = assethubContext.api.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
		dest,
		funds,
		'LocalReserve',
		remoteFeeId,
		'LocalReserve',
		getXcmMessageV4ToSendEkilt(keysAlice.address),
		'Unlimited'
	)

	const events = await sendTransaction(signedTx.signAsync(keysAlice))

	await createBlock(assethubContext)

	checkEvents(events, 'xcmpQueue').toMatchSnapshot(
		`sender AssetHubs::xcmpQueue::[XcmpMessageSent] ${JSON.stringify(funds)}`
	)
	checkEvents(events, { section: 'polkadotXcm', method: 'Attempted' }).toMatchSnapshot(
		`sender AssetHub::polkadotXcm::[Attempted] ${JSON.stringify(funds)}`
	)
	checkEvents(events, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		`sender AssetHub::foreignAssets::[Transferred] ${JSON.stringify(funds)}`
	)

	// check balance. Alice should have 50 ePILTs less
	const freeBalanceAlice = await getFreeEkiltAssetHub(keysAlice.address)
	expect(freeBalanceAlice).toBe(fundsAlice - balanceToTransfer)

	// the sovereign account should have 50 more PILTs
	const balanceSovereignAccountAfterTx = await getFreeEkiltAssetHub(PeregrineConfig.siblingSovereignAccount)
	expect(balanceSovereignAccountAfterTx).eq(initialBalanceSovereignAccount + balanceToTransfer)

	await createBlock(peregrineContext)

	checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot('receiver Peregrine::messageQueue::[Processed]')
	checkSystemEvents(peregrineContext, 'assetSwitchPool1').toMatchSnapshot(
		'receiver Peregrine::assetSwitchPool1::[RemoteToLocalSwitchExecuted]'
	)
	checkSystemEvents(peregrineContext, { section: 'balances', method: 'Transfer' }).toMatchSnapshot(
		'receiver Peregrine::balances::[Transfer]'
	)

	// Alice should have some coins now. Calculating the exact amount is not easy. Since fees are taken by xcm.
	checkBalanceInRange(getFreeBalancePeregrine, keysAlice.address, expect, [
		balanceToTransfer - KILT,
		balanceToTransfer,
	])

	// Pool account should have less locked PILTs
	const freeBalancePoolAccount = await getFreeBalancePeregrine(PeregrineConfig.initialPoolAccountId)
	expect(initialBalancePoolAccount - balanceToTransfer).toBe(freeBalancePoolAccount)

	// remote locked supply should have increased by the amount of the transferred PILTs
	const remoteLockedSupply = await getRemoteLockedSupply()
	expect(remoteLockedSupply).toBe(initialRemoteLockedSupply + balanceToTransfer)

	await checkSwitchPalletInvariant(expect)
}, 20_000)
