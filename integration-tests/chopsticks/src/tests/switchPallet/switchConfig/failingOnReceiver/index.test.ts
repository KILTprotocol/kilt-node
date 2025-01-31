import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock } from '../../../../network/utils.js'
import { calculateTxFees, hexAddress } from '../../../../helper/utils.js'
import { testCases } from './config.js'
import type { Config } from '../../../../network/types.js'
import { checkSwitchPalletInvariant } from '../../index.js'
import { spinUpNetwork, tearDownNetwork } from '../../../utils.js'

describe.each(testCases)(
	'Switch KILTs while receiver can not handle them',

	async ({ account, query, txContext, config, sovereignAccount }) => {
		let senderContext: Config
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		const { desc } = config

		// Create the network context
		beforeEach(async () => {
			const { parachainContexts, relayChainContext } = await spinUpNetwork(config)
			relayContext = relayChainContext
			senderContext = parachainContexts[0]
			receiverContext = parachainContexts[1]
			senderAccount = account
		})

		// Shut down the network
		afterEach(async () => {
			await tearDownNetwork([receiverContext, senderContext, relayContext])
		})

		it(desc, async ({ expect }) => {
			const { checkSystemEvents, checkEvents } = withExpect(expect)
			const { tx, balanceToTransfer, events } = txContext

			//pre condition checks
			const balanceBeforeTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			const balanceBeforeTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceBeforeTx).toBe(BigInt(0))
			expect(balanceBeforeTxSender).toBeGreaterThan(BigInt(0))

			// action
			const rawTx = tx(senderContext, hexAddress(senderAccount.address), balanceToTransfer.toString())
			const events1 = await sendTransaction(rawTx.signAsync(senderAccount))

			// process tx
			await createBlock(senderContext)
			// process msg
			await createBlock(receiverContext)

			// post condition checks
			// check balance movement on sender chain.
			const txFees = await calculateTxFees(rawTx, senderAccount)
			const balanceAfterTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceAfterTxSender).toBe(balanceBeforeTxSender - balanceToTransfer - txFees)

			// Tx should fail on receiver
			const balanceAfterTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))

			expect(balanceAfterTx).toBe(BigInt(0))

			// check events
			await Promise.all(
				events.sender.map((pallet) =>
					checkEvents(events1, pallet).toMatchSnapshot(`Switch on native chain: ${JSON.stringify(pallet)}`)
				)
			)

			await Promise.all(
				events.receiver.map((pallet) =>
					checkSystemEvents(receiverContext, pallet).toMatchSnapshot(
						`Switch on receiver chain: ${JSON.stringify(pallet)}`
					)
				)
			)

			// finalize switch
			await createBlock(senderContext)
			await checkSystemEvents(senderContext, 'assetSwitchPool1').toMatchSnapshot('assetSwitchPool1 Finalization')

			const balanceAfterFinalization = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceAfterFinalization).toBe(balanceBeforeTxSender - txFees)

			await checkSwitchPalletInvariant(
				expect,
				senderContext,
				receiverContext,
				sovereignAccount,
				query.sender,
				query.receiver
			)
		})
	}
)
