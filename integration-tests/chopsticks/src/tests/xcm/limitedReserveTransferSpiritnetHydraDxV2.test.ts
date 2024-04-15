import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as SpiritnetConfig from '../../network/spiritnet.js'
import * as HydraDxConfig from '../../network/hydraDx.js'
import { KILT, keysAlice } from '../../utils.js'
import { spiritnetContext, hydradxContext, getFreeBalanceSpiritnet, getFreeBalanceHydraDxKilt } from '../index.js'
import { getAccountDestinationV2, getNativeAssetIdLocation, getSiblingDestination } from '../../network/utils.js'
import { checkBalance, createBlock, hexAddress, setStorage } from '../utils.js'

const KILT_ASSET_V2 = { V2: [getNativeAssetIdLocation(KILT)] }

test('Limited Reserve V2 Transfers from Spiritnet Account Alice -> HydraDx', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Set storage
	await setStorage(spiritnetContext, SpiritnetConfig.assignNativeTokensToAccount([keysAlice.address]))
	await setStorage(hydradxContext, HydraDxConfig.assignNativeTokensToAccount([keysAlice.address]))

	// check initial balance
	await checkBalance(getFreeBalanceSpiritnet, SpiritnetConfig.hydraDxSovereignAccount, expect)
	await checkBalance(getFreeBalanceHydraDxKilt, HydraDxConfig.omnipoolAccount, expect)

	const omniPoolAddress = hexAddress(HydraDxConfig.omnipoolAccount)
	const hydraDxDestination = { V2: getSiblingDestination(HydraDxConfig.paraId) }
	const beneficiary = getAccountDestinationV2(omniPoolAddress)

	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(hydraDxDestination, beneficiary, KILT_ASSET_V2, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Order matters here, we need to create a block on the sender first
	await createBlock(spiritnetContext)
	await createBlock(hydradxContext)

	// Check events sender
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	checkEvents(events, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	// Check events receiver
	checkSystemEvents(hydradxContext, { section: 'currencies', method: 'Deposited' }).toMatchSnapshot(
		'receiver events currencies'
	)
	checkSystemEvents(hydradxContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')

	// check balance
	await checkBalance(getFreeBalanceSpiritnet, SpiritnetConfig.hydraDxSovereignAccount, expect, KILT)
	await checkBalance(getFreeBalanceHydraDxKilt, HydraDxConfig.omnipoolAccount, expect, KILT)
}, 20_000)
