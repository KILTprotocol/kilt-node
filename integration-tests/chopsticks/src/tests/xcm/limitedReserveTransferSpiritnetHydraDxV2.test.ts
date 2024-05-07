import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as SpiritnetConfig from '../../network/spiritnet.js'
import * as HydraDxConfig from '../../network/hydraDx.js'
import { KILT, keysAlice } from '../../utils.js'
import { spiritnetContext, hydradxContext, getFreeBalanceSpiritnet, getFreeBalanceHydraDxKilt } from '../index.js'
import { getAccountLocationV2, getNativeAssetIdLocation, getSiblingLocation } from '../../network/utils.js'
import { checkBalance, createBlock, hexAddress, setStorage } from '../utils.js'

const KILT_ASSET_V2 = { V2: [getNativeAssetIdLocation(KILT)] }

test('Limited Reserve V2 Transfers from Spiritnet Account Alice -> HydraDx', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Set storage
	await setStorage(spiritnetContext, SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address]))
	await setStorage(hydradxContext, HydraDxConfig.assignNativeTokensToAccounts([keysAlice.address]))

	// check initial balance
	await checkBalance(getFreeBalanceSpiritnet, SpiritnetConfig.hydraDxSovereignAccount, expect)
	await checkBalance(getFreeBalanceHydraDxKilt, HydraDxConfig.omnipoolAccount, expect)

	const omniPoolAddress = hexAddress(HydraDxConfig.omnipoolAccount)
	const hydraDxDestination = { V2: getSiblingLocation(HydraDxConfig.paraId) }
	const beneficiary = getAccountLocationV2(omniPoolAddress)

	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(hydraDxDestination, beneficiary, KILT_ASSET_V2, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Check sender state
	await createBlock(spiritnetContext)

	// Check events sender
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	checkEvents(events, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	// check balance
	await checkBalance(getFreeBalanceSpiritnet, SpiritnetConfig.hydraDxSovereignAccount, expect, KILT)

	// Check receiver state
	await createBlock(hydradxContext)

	// Check events receiver
	checkSystemEvents(hydradxContext, { section: 'currencies', method: 'Deposited' }).toMatchSnapshot(
		'receiver events currencies'
	)
	checkSystemEvents(hydradxContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')

	// check balance
	await checkBalance(getFreeBalanceHydraDxKilt, HydraDxConfig.omnipoolAccount, expect, KILT)
}, 20_000)
