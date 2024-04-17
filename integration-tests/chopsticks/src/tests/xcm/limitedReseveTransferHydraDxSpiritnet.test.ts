import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as HydraDxConfig from '../../network/hydraDx.js'
import * as SpiritnetConfig from '../../network/spiritnet.js'
import { KILT, initialBalanceKILT, keysAlice, keysBob } from '../../utils.js'
import { getFreeBalanceHydraDxKilt, getFreeBalanceSpiritnet, hydradxContext, spiritnetContext } from '../index.js'
import { checkBalance, createBlock, hexAddress, setStorage } from '../utils.js'

const destinationAlice = {
	V3: {
		parents: 1,
		interior: {
			X2: [
				{ Parachain: SpiritnetConfig.paraId },
				{
					AccountId32: {
						id: hexAddress(keysAlice.address),
					},
				},
			],
		},
	},
}

test('Limited Reserve Transfers from HydraDx Account Bob -> Spiritnet', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	const hydraDxConfig = {
		...HydraDxConfig.assignKiltTokensToAccounts([keysBob.address, HydraDxConfig.omnipoolAccount]),
		...HydraDxConfig.assignNativeTokensToAccounts([keysBob.address, HydraDxConfig.omnipoolAccount]),
	}

	// Update storage
	await setStorage(
		spiritnetContext,
		SpiritnetConfig.assignNativeTokensToAccounts([SpiritnetConfig.hydraDxSovereignAccount])
	)
	await setStorage(hydradxContext, hydraDxConfig)

	await createBlock(spiritnetContext)
	await createBlock(hydradxContext)

	// check initial balance of alice
	await checkBalance(getFreeBalanceSpiritnet, keysAlice.address, expect)

	const signedTx = hydradxContext.api.tx.xTokens
		.transfer(HydraDxConfig.kiltTokenId, KILT, destinationAlice, 'Unlimited')
		.signAsync(keysBob)

	const events = await sendTransaction(signedTx)

	// Order matters here, we need to create a block on the sender first
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
	await checkBalance(
		getFreeBalanceSpiritnet,
		SpiritnetConfig.hydraDxSovereignAccount,
		expect,
		initialBalanceKILT - KILT
	)
	await checkBalance(getFreeBalanceHydraDxKilt, keysBob.address, expect, initialBalanceKILT - KILT)
	// Alice receives a bit less since the tx fees has to be paid.
	await checkBalance(getFreeBalanceSpiritnet, keysAlice.address, expect, BigInt('99999999999971175'))
}, 20_000)
