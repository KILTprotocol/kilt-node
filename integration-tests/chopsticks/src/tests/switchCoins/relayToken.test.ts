import { test } from 'vitest'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as BasiliskConfig from '../../network/basilisk.js'
import { initialBalanceKILT, initialBalanceROC, keysAlice, keysBob, keysCharlie } from '../../utils.js'
import {
	peregrineContext,
	getFreeBalancePeregrine,
	getFreeRocPeregrine,
	basiliskContext,
	rococoContext,
} from '../index.js'
import { checkBalance, createBlock, setStorage, hexAddress, checkBalanceInRange } from '../utils.js'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

test('User transfers all of his dots', async ({ expect }) => {
	const { checkEvents } = withExpect(expect)

	// Assign alice some KILTs and ROCs
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address, keysBob.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address]),
	})

	const signedTx = peregrineContext.api.tx.fungibles
		.transfer(PeregrineConfig.ROC_LOCATION, keysBob.address, initialBalanceROC)
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	await createBlock(peregrineContext)

	checkEvents(events, { section: 'fungibles', method: 'Transferred' }).toMatchSnapshot('balances transfer event')

	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))

	await checkBalance(getFreeRocPeregrine, keysBob.address, expect, initialBalanceROC)

	await checkBalanceInRange(getFreeBalancePeregrine, keysAlice.address, expect, [
		BigInt('99999800999995545'),
		initialBalanceKILT,
	])
}, 20_000)

test('User gets dusted with ROCs', async ({ expect }) => {
	const { checkEvents } = withExpect(expect)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address]),
	})

	const balanceToTransfer = (initialBalanceKILT * BigInt(999998)) / BigInt(1000000)

	const signedTx = peregrineContext.api.tx.balances
		.transferAllowDeath(keysBob.address, balanceToTransfer)
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	await createBlock(peregrineContext)

	checkEvents(events, { section: 'balances', method: 'Transfer' }).toMatchSnapshot('balances transfer event')
	// User should get dusted by this operation
	checkEvents(events, { section: 'balances', method: 'DustLost' }).toMatchSnapshot('balances transfer event')

	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)
}, 20_000)

// Is failing: Todo: Fix XCM config

// Is failing: Todo: Fix XCM config
test('Send DOTs from basilisk 2 Peregrine', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair())

	await setStorage(basiliskContext, {
		...BasiliskConfig.assignNativeTokensToAccounts([keysAlice.address]),
		...BasiliskConfig.assignRocTokensToAccounts([keysAlice.address], initialBalanceROC),
	})

	const balanceToTransfer = initialBalanceROC / BigInt(2)

	const beneficiary = {
		V3: {
			parents: 1,
			interior: {
				X2: [
					{ Parachain: PeregrineConfig.paraId },
					{
						AccountId32: {
							id: hexAddress(keysAlice.address),
						},
					},
				],
			},
		},
	}

	const signedTx = basiliskContext.api.tx.xTokens
		.transfer(BasiliskConfig.dotTokenId, balanceToTransfer, beneficiary, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	await createBlock(basiliskContext)

	checkEvents(events, 'xTokens').toMatchSnapshot('sender events xTokens')

	checkEvents(events, 'tokens').toMatchSnapshot('sender events tokens')

	checkEvents(events, 'parachainSystem').toMatchSnapshot('sender events tokens')

	await createBlock(rococoContext)

	checkSystemEvents(rococoContext, 'messageQueue').toMatchSnapshot('relayer events messageQueue')
	checkSystemEvents(rococoContext, { section: 'balances', method: 'Minted' }).toMatchSnapshot(
		'relayer events balances minted'
	)

	checkSystemEvents(rococoContext, { section: 'balances', method: 'Burned' }).toMatchSnapshot(
		'relayer events balances Burned'
	)

	await createBlock(peregrineContext)

	checkSystemEvents(peregrineContext, {
		section: 'parachainSystem',
		method: 'DownwardMessagesReceived',
	}).toMatchSnapshot('receiver events parachainSystem pallet')

	checkSystemEvents(peregrineContext, {
		section: 'dmpQueue',
		method: 'ExecutedDownward',
	}).toMatchSnapshot('receiver events dmpQueue')

	//await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))
}, 20_000)
