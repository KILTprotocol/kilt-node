import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as SpiritnetConfig from '../../../network/spiritnet.js'
import * as AssetHubConfig from '../../../network/assetHub.js'
import { KILT, initialBalanceKILT, keysAlice } from '../../../utils.js'
import { spiritnetContext, assetHubContext, getFreeBalanceSpiritnet } from '../../index.js'
import { getAccountLocationV3, getNativeAssetIdLocation, getSiblingLocation } from '../../../network/utils.js'
import { checkBalance, createBlock, hexAddress, setStorage } from '../../utils.js'

test('Limited Reserve Transfers from Spiritnet Account Alice -> AH Account Alice', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Assign alice some KILT tokens
	await setStorage(spiritnetContext, {
		...SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...SpiritnetConfig.setSafeXcmVersion3(),
	})

	// Balance of the AH sovereign account before the transfer
	const assetHubSovereignAccountBalance = await getFreeBalanceSpiritnet(AssetHubConfig.siblingSovereignAccount)

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalanceSpiritnet, keysAlice.address, expect, initialBalanceKILT)

	const aliceAddress = hexAddress(keysAlice.address)
	const assetHubDestination = { V3: getSiblingLocation(AssetHubConfig.paraId) }
	const beneficiary = getAccountLocationV3(aliceAddress)
	const kiltAsset = { V3: [getNativeAssetIdLocation(KILT)] }

	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(assetHubDestination, beneficiary, kiltAsset, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Check sender state
	await createBlock(spiritnetContext)

	// Check events sender
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	checkEvents(events, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	//	check balance. The sovereign account should hold one additional KILT.
	await checkBalance(
		getFreeBalanceSpiritnet,
		AssetHubConfig.siblingSovereignAccount,
		expect,
		assetHubSovereignAccountBalance + KILT
	)

	await createBlock(assetHubContext)

	// MSG processing will fail on AH.
	await checkSystemEvents(assetHubContext, 'messageQueue').toMatchSnapshot('AH message queue')
}, 20_000)
