import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock } from '../../../network/utils.js'
import { hexAddress, validateBalanceWithPrecision } from '../../../helper/utils.js'
import { testPairsWithdrawAssets } from './config.js'
import { Config } from '../../../network/types.js'
import { spinUpNetwork, tearDownNetwork } from '../../utils.js'

describe.each(testPairsWithdrawAssets)(
	'Withdraw Asset',

	({ accounts, query, sovereignAccount, txContext, config }) => {
		let senderContext: Config
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		let receiverAccount: KeyringPair
		const { desc, precision } = config

		// Create the network context
		beforeEach(async () => {
			const { parachainContexts, relayChainContext } = await spinUpNetwork(config)

			relayContext = relayChainContext
			senderContext = parachainContexts[0]
			receiverContext = parachainContexts[1]

			const { senderAccount: a, receiverAccount: b } = accounts
			senderAccount = a
			receiverAccount = b
		})

		// Shut down the network
		afterEach(async () => await tearDownNetwork([receiverContext, senderContext, relayContext]))

		it(desc, async ({ expect }) => {
			const { checkEvents, checkSystemEvents } = withExpect(expect)
			const { pallets, tx, balanceToTransfer } = txContext

			// precondition checks
			// Balance of the sovereign account before the transfer
			const senderSovereignAccountBalanceBeforeTransfer = await query.receiver(
				receiverContext,
				sovereignAccount.receiver
			)

			const balanceSenderBeforeTransfer = await query.sender(senderContext, senderAccount.address)
			const initialBalanceReceiver = await query.receiver(receiverContext, receiverAccount.address)
			// Check initial balance receiver should be zero
			expect(initialBalanceReceiver).toBe(0n)

			// action
			const signedTx = tx(
				senderContext,
				hexAddress(receiverAccount.address),
				balanceToTransfer.toString()
			).signAsync(senderAccount)

			const events = await sendTransaction(signedTx)

			// check sender state
			await createBlock(senderContext)

			// post condition checks

			Promise.all(
				pallets.sender.map((pallet) =>
					checkEvents(events, pallet)
						.redact({ number: 1 })
						.toMatchSnapshot(`sender events ${JSON.stringify(pallet)}`)
				)
			)

			const balanceSenderAfterTransfer = await query.sender(senderContext, senderAccount.address)
			const removedBalance = balanceToTransfer * -1n

			validateBalanceWithPrecision(
				balanceSenderBeforeTransfer,
				balanceSenderAfterTransfer,
				removedBalance,
				expect,
				precision
			)

			// check receiver state
			await createBlock(receiverContext)

			const senderSovereignAccountBalanceAfterTransfer = await query.receiver(
				receiverContext,
				sovereignAccount.receiver
			)

			expect(senderSovereignAccountBalanceAfterTransfer).toBe(
				senderSovereignAccountBalanceBeforeTransfer - balanceToTransfer
			)

			await Promise.all(
				pallets.receiver.map((pallet) =>
					checkSystemEvents(receiverContext, pallet)
						.redact({ number: 1 })
						.toMatchSnapshot(`receiver events ${JSON.stringify(pallet)}`)
				)
			)

			const balanceReceiverAfterTransfer = await query.receiver(receiverContext, receiverAccount.address)

			validateBalanceWithPrecision(
				initialBalanceReceiver,
				balanceReceiverAfterTransfer,
				balanceToTransfer,
				expect,
				precision
			)
		})
	}
)
