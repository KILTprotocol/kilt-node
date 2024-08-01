import { test } from 'vitest'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as BasiliskConfig from '../../network/basilisk.js'
import {
	getAssetSwitchParameters,
	initialBalanceKILT,
	initialBalanceROC,
	keysAlice,
	keysBob,
	keysCharlie,
} from '../../utils.js'
import {
	peregrineContext,
	getFreeBalancePeregrine,
	getFreeRocPeregrine,
	basiliskContext,
	rococoContext,
} from '../index.js'
import { checkBalance, createBlock, setStorage, hexAddress, checkBalanceInRange } from '../utils.js'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

test.skip('User transfers all of his dots', async ({ expect }) => {
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

	checkEvents(events, { section: 'fungibles', method: 'Transferred' }).toMatchSnapshot(
		`local Peregrine::fungibles::[Transferred] asset ${JSON.stringify(PeregrineConfig.ROC_LOCATION)}`
	)

	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))

	await checkBalance(getFreeRocPeregrine, keysBob.address, expect, initialBalanceROC)

	await checkBalanceInRange(getFreeBalancePeregrine, keysAlice.address, expect, [
		BigInt('99999800999995545'),
		initialBalanceKILT,
	])
}, 20_000)

test.skip('User gets dusted with ROCs', async ({ expect }) => {
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

	checkEvents(events, { section: 'balances', method: 'Transfer' }).toMatchSnapshot(
		'local Peregrine::balances::[Transfer] native asset'
	)
	// User should get dusted by this operation
	checkEvents(events, { section: 'balances', method: 'DustLost' }).toMatchSnapshot(
		'local balances::fungibles::[DustLost]'
	)
	// he should keep his rocs
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)
}, 20_000)

test('Send DOTs from basilisk 2 Peregrine', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(getAssetSwitchParameters()))

	await setStorage(basiliskContext, {
		...BasiliskConfig.assignNativeTokensToAccounts([keysAlice.address]),
		...BasiliskConfig.assignRocTokensToAccounts([keysAlice.address], initialBalanceROC),
	})

	const balanceToTransfer = initialBalanceROC / BigInt(2)

	const beneficiary = {
		V4: {
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

	await checkEvents(events, 'xTokens').toMatchSnapshot('sender basilisk::xTokens::[TransferredAssets] DOTs')
	await checkEvents(events, 'tokens').toMatchSnapshot('sender basilisk::tokens::[Withdraw] DOTs')
	await checkEvents(events, 'parachainSystem').toMatchSnapshot(
		'sender basilisk::parachainSystem::[UpwardMessageSent]'
	)

	await createBlock(rococoContext)
	await checkSystemEvents(rococoContext, 'messageQueue').toMatchSnapshot('relayer rococo::messageQueue::[Processed]')
	await checkSystemEvents(rococoContext, 'balances').toMatchSnapshot('relayer rococo::balances::[Minted,Burned]')

	await createBlock(peregrineContext)

	// Barrier blocked execution. No event will be emitted.
	await checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot(
		'relayer rococo::messageQueue::[ProcessingFailed]'
	)

	// Alice should still have no rocs.
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))
}, 20_000)
