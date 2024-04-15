import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'

import * as HydraDxConfig from '../../network/hydraDx.js'
import * as SpiritnetConfig from '../../network/spiritnet.js'
import { KILT, initialBalanceKILT, keysAlice, keysBob } from '../../utils.js'
import { getFreeBalanceHydraDxKilt, getFreeBalanceSpiritnet, hydradxContext, spiritnetContext } from '../index.js'
import { checkBalanceAndExpectAmount, checkBalanceAndExpectZero, createBlock, setStorage } from '../utils.js'

test('Limited Reserve Transfers from HydraDx Account Bob -> Spiritnet', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	const hydraDxSovereignAccount = u8aToHex(decodeAddress(SpiritnetConfig.hydraDxSovereignAccount))

	// Create some new blocks to have consistent snapshots
	await setStorage(spiritnetContext, SpiritnetConfig.assignNativeTokensToAccount(hydraDxSovereignAccount))
	await setStorage(spiritnetContext, SpiritnetConfig.setSafeXcmVersion(3))
	await setStorage(hydradxContext, HydraDxConfig.registerKilt())
	await setStorage(hydradxContext, HydraDxConfig.assignNativeTokensToAccount(keysBob.address))

	// check initial balance of alice
	await checkBalanceAndExpectZero(getFreeBalanceSpiritnet, keysAlice.address, expect)

	const destination = {
		V3: {
			parents: 1,
			interior: {
				X2: [
					{ Parachain: SpiritnetConfig.paraId },
					{
						AccountId32: {
							id: u8aToHex(decodeAddress(keysAlice.address)),
						},
					},
				],
			},
		},
	}

	const signedTx = hydradxContext.api.tx.xTokens
		.transfer(HydraDxConfig.kiltTokenId, KILT, destination, 'Unlimited')
		.signAsync(keysBob)

	const events = await sendTransaction(signedTx)

	// Produce a new Block
	await createBlock(hydradxContext)
	await createBlock(spiritnetContext)

	// Check Events HydraDx
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, { section: 'currencies', method: 'Withdrawn' }).toMatchSnapshot('sender events currencies')
	checkEvents(events, 'xTokens').toMatchSnapshot('sender events currencies')

	// check Events Spiritnet
	checkSystemEvents(spiritnetContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')
	checkSystemEvents(spiritnetContext, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot(
		'receiver events Balances'
	)
	checkSystemEvents(spiritnetContext, { section: 'balances', method: 'Endowed' }).toMatchSnapshot(
		'receiver events Balances'
	)

	// Check Balance
	await checkBalanceAndExpectAmount(
		getFreeBalanceSpiritnet,
		SpiritnetConfig.hydraDxSovereignAccount,
		expect,
		initialBalanceKILT - KILT
	)
	await checkBalanceAndExpectAmount(getFreeBalanceHydraDxKilt, keysBob.address, expect, initialBalanceKILT - KILT)
	// Alice receives a bit less since the tx fees has to be paid.
	await checkBalanceAndExpectAmount(getFreeBalanceSpiritnet, keysAlice.address, expect, BigInt('99999999999971175'))
}, 20_000)
