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
	// Set balance to 0 for HydraDX sovereign account on Spiritnet
	await setStorage(spiritnetContext, SpiritnetConfig.assignNativeTokensToAccounts([SpiritnetConfig.hydraDxSovereignAccount], BigInt(0)))
	// Set KILT balance to 0 for Omnipool account on HydraDx
	await setStorage(hydradxContext, HydraDxConfig.assignKiltTokensToAccounts([HydraDxConfig.omnipoolAccount], BigInt(0)))

	// check initial balance
	await checkBalance(getFreeBalanceSpiritnet, SpiritnetConfig.hydraDxSovereignAccount, expect, BigInt(0))
	await checkBalance(getFreeBalanceHydraDxKilt, HydraDxConfig.omnipoolAccount, expect, BigInt(0))

	const omnipoolAddress = hexAddress(HydraDxConfig.omnipoolAccount)
	const hydraDxDestination = { V2: getSiblingLocation(HydraDxConfig.paraId) }
	const beneficiary = getAccountLocationV2(omnipoolAddress)

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
