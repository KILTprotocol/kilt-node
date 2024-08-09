import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assetHub.js'
import { KILT, ROC, getAssetSwitchParameters, initialBalanceROC, keysAlice, keysCharlie } from '../../utils.js'
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
	const { checkSystemEvents } = withExpect(expect)

	const switchParameters = getAssetSwitchParameters()
	// alice has the whole circulating supply.
	const fundsAlice = switchParameters.circulatingSupply
	const feeAmount = (ROC * BigInt(10)) / BigInt(100)

	const remoteAssetId = { V4: AssetHubConfig.eKiltLocation }
	const remoteXcmFeeId = { V4: { id: AssetHubConfig.nativeTokenLocation, fun: { Fungible: feeAmount } } }
	const remoteReserveLocation = getSiblingLocationV4(AssetHubConfig.paraId)

	await setStorage(peregrineContext, {
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
		...PeregrineConfig.setSwitchPair(switchParameters, remoteAssetId, remoteXcmFeeId, remoteReserveLocation),
		...PeregrineConfig.setSafeXcmVersion4(),
	})

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
			[keysAlice.address, fundsAlice],
		])
	)

	// check initial balance of Alice on Spiritnet. Alice should have 0 KILT
	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, BigInt(0))
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, switchParameters.circulatingSupply)

	const initialBalancePoolAccount = await getFreeBalancePeregrine(PeregrineConfig.initialPoolAccountId)
	const initialBalanceSovereignAccount = await getFreeEkiltAssetHub(PeregrineConfig.sovereignAccountAsSibling)
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

	// send msg
	await sendTransaction(signedTx.signAsync(keysAlice))
	await createBlock(assethubContext)

	// check balance. Alice should have 50 ePILTs less
	const freeBalanceAlice = await getFreeEkiltAssetHub(keysAlice.address)
	expect(freeBalanceAlice).toBe(fundsAlice - balanceToTransfer)

	// the sovereign account should have 50 more PILTs
	const balanceSovereignAccountAfterTx = await getFreeEkiltAssetHub(PeregrineConfig.sovereignAccountAsSibling)
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
