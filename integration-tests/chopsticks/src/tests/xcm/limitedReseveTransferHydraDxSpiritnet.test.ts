import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as HydraDxConfig from '../../network/hydraDx.js'
import * as SpiritnetConfig from '../../network/spiritnet.js'
import { KILT, initialBalanceHDX, initialBalanceKILT, keysAlice } from '../../utils.js'
import { getFreeBalanceHydraDxKilt, getFreeBalanceSpiritnet, hydradxContext, spiritnetContext } from '../index.js'
import { checkBalance, createBlock, hexAddress, setStorage, checkBalanceInRange } from '../utils.js'

const aliceLocation = {
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

test('Limited Reserve Transfers from HydraDx Account Alice -> Spiritnet Account Alice', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// assign initial balance to Alice. Alice also needs to have some HDX, otherwise the account gets dusted
	const config = {
		...HydraDxConfig.assignKiltTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...HydraDxConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceHDX),
	}

	// Set storage
	await setStorage(hydradxContext, config)

	const hydraDxSovereignAccountBalanceBeforeTransfer = await getFreeBalanceSpiritnet(
		SpiritnetConfig.hydraDxSovereignAccount
	)

	// check initial balance of alice
	await checkBalance(getFreeBalanceHydraDxKilt, keysAlice.address, expect, initialBalanceKILT)

	const signedTx = hydradxContext.api.tx.xTokens
		.transfer(HydraDxConfig.kiltTokenId, KILT, aliceLocation, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Check sender state
	await createBlock(hydradxContext)

	// Check events sender
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, { section: 'currencies', method: 'Withdrawn' }).toMatchSnapshot('sender events currencies')
	checkEvents(events, 'xTokens').toMatchSnapshot('sender events currencies')

	// Check balance
	await checkBalance(getFreeBalanceHydraDxKilt, keysAlice.address, expect, initialBalanceKILT - KILT)

	// Check receiver state
	await createBlock(spiritnetContext)

	// check events receiver
	checkSystemEvents(spiritnetContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')
	checkSystemEvents(spiritnetContext, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot(
		'receiver events Balances'
	)
	checkSystemEvents(spiritnetContext, { section: 'balances', method: 'Endowed' }).toMatchSnapshot(
		'receiver events Balances'
	)

	// Check balance receiver
	await checkBalance(
		getFreeBalanceSpiritnet,
		SpiritnetConfig.hydraDxSovereignAccount,
		expect,
		hydraDxSovereignAccountBalanceBeforeTransfer - KILT
	)

	// Alice receives a bit less since the tx fees has to be paid.
	await checkBalanceInRange(getFreeBalanceSpiritnet, keysAlice.address, expect, [
		BigInt('999999999971174'),
		BigInt('999999999976345'),
	])
}, 20_000)
