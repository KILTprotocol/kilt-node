import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as SpiritnetConfig from '../../../../network/spiritnet.js'
import * as AssetHubConfig from '../../../../network/assetHub.js'
import { KILT, initialBalanceKILT, keysAlice } from '../../../../utils.js'
import { spiritnetContext, assethubContext, getFreeBalanceSpiritnet } from '../../../index.js'
import { getAccountLocationV3, getNativeAssetIdLocation, getSiblingLocation } from '../../../../network/utils.js'
import { checkBalance, createBlock, hexAddress, setStorage } from '../../../utils.js'

test('Limited Reserve Transfers from Spiritnet Account Alice -> AH Account Alice', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Assign alice some KILT tokens
	await setStorage(spiritnetContext, {
		...SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...SpiritnetConfig.XcmPalletSafeVersion3StorageEntry(),
	})

	// Balance of the AH sovereign account before the transfer
	const assetHubSovereignAccountBalance = await getFreeBalanceSpiritnet(
		AssetHubConfig.sovereignAccountOnSiblingChains
	)

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalanceSpiritnet, keysAlice.address, expect, initialBalanceKILT)

	const aliceAddress = hexAddress(keysAlice.address)
	const assetHubDestination = { V3: getSiblingLocation(AssetHubConfig.paraId) }
	const beneficiary = getAccountLocationV3(aliceAddress)
	const kiltAsset = { V3: [getNativeAssetIdLocation(KILT.toString())] }

	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(assetHubDestination, beneficiary, kiltAsset, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Check sender state
	await createBlock(spiritnetContext)

	// Check events sender
	await checkEvents(events, 'xcmpQueue').toMatchSnapshot(
		`sender spiritnet::xcmpQueue::[XcmpMessageSent] asset ${JSON.stringify(kiltAsset)}`
	)
	await checkEvents(events, 'polkadotXcm').toMatchSnapshot(
		`sender spiritnet::polkadotXcm::[Attempted] asset ${JSON.stringify(kiltAsset)}`
	)
	await checkEvents(events, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot(
		`sender spiritnet::balances::[Withdraw] asset ${JSON.stringify(kiltAsset)}`
	)

	//	check balance. The sovereign account should hold one additional KILT.
	await checkBalance(
		getFreeBalanceSpiritnet,
		AssetHubConfig.sovereignAccountOnSiblingChains,
		expect,
		assetHubSovereignAccountBalance + KILT
	)

	await createBlock(assethubContext)

	// MSG processing will fail on AH.
	await checkSystemEvents(assethubContext, 'messageQueue').toMatchSnapshot(
		`receiver assetHub::messageQueue::[Processed] asset ${JSON.stringify(kiltAsset)}`
	)
}, 20_000)
