import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, setStorage } from '../../../network/utils.js'
import { hexAddress, validateBalanceWithPrecision } from '../../../helper/utils.js'
import { testPairsWithdrawAssets } from './config.js'
import { Config } from '../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../../network/utils.js'

describe.each(testPairsWithdrawAssets)(
	'Withdraw Asset',

	async ({ accounts, query, sovereignAccount, txContext, config }) => {
		let senderContext: Config
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		let receiverAccount: KeyringPair
		const { desc, precision } = config

		// Create the network context
		beforeEach(async () => {
			const { network, storage } = config
			const { parachains, relay } = network

			const { parachainContexts, relayChainContext } = await setupNetwork(relay, parachains)
			const [senderChainContext, receiverChainContext] = parachainContexts

			relayContext = relayChainContext
			senderContext = senderChainContext
			receiverContext = receiverChainContext

			const { receiverStorage, senderStorage, relayStorage } = storage
			await setStorage(senderContext, senderStorage)
			await setStorage(receiverContext, receiverStorage)
			await setStorage(relayContext, relayStorage)

			const { senderAccount: a, receiverAccount: b } = accounts
			senderAccount = a
			receiverAccount = b
		})

		// Shut down the network
		afterEach(async () => {
			try {
				await shutDownNetwork([senderContext, receiverContext, relayContext])
			} catch (error) {
				if (!(error instanceof TypeError)) {
					console.error(error)
				}
			}
		})

		it(desc, async ({ expect }) => {
			const { checkEvents, checkSystemEvents } = withExpect(expect)

			const { pallets, tx, balanceToTransfer } = txContext

			// Balance of the sovereign account before the transfer
			const senderSovereignAccountBalanceBeforeTransfer = await query.receiver(
				receiverContext,
				sovereignAccount.receiver
			)

			const balanceSenderBeforeTransfer = await query.sender(senderContext, senderAccount.address)
			const initialBalanceReceiver = await query.receiver(receiverContext, receiverAccount.address)

			// Check initial balance receiver should be zero
			expect(initialBalanceReceiver).toBe(BigInt(0))

			// Fire tx
			const signedTx = tx(
				senderContext,
				hexAddress(receiverAccount.address),
				balanceToTransfer.toString()
			).signAsync(senderAccount)

			const events = await sendTransaction(signedTx)

			// check sender state
			await createBlock(senderContext)

			Promise.all(
				pallets.sender.map((pallet) =>
					checkEvents(events, pallet)
						.redact({ number: 1 })
						.toMatchSnapshot(`sender events ${JSON.stringify(pallet)}`)
				)
			)

			const balanceSenderAfterTransfer = await query.sender(senderContext, senderAccount.address)
			const removedBalance = balanceToTransfer * BigInt(-1)

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
				senderSovereignAccountBalanceBeforeTransfer - BigInt(balanceToTransfer)
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
