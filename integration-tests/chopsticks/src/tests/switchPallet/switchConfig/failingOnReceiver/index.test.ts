import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, setStorage } from '../../../../network/utils.js'
import { calculateTxFees, hexAddress } from '../../../../helper/utils.js'
import { testCases } from './config.js'
import type { Config } from '../../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../../../network/utils.js'
import { checkSwitchPalletInvariant } from '../../index.js'

describe.each(testCases)(
	'Switch KILTs while receiver can not handle them',
	{ timeout: 60_000 },
	async ({ account, query, txContext, config, sovereignAccount }) => {
		let senderContext: Config
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		const { desc, network, storage } = config

		// Create the network context
		beforeEach(async () => {
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

			senderAccount = account
		}, 40_000)

		// Shut down the network
		afterEach(async () => {
			try {
				await shutDownNetwork([receiverContext, senderContext, relayContext])
			} catch (error) {
				if (!(error instanceof TypeError)) {
					console.error(error)
				}
			}
		})

		it(desc, { timeout: 20_000 }, async ({ expect }) => {
			const { checkSystemEvents, checkEvents } = withExpect(expect)

			const { tx, balanceToTransfer, events } = txContext

			// initial checks
			const balanceBeforeTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			const balanceBeforeTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceBeforeTx).toBe(BigInt(0))
			expect(balanceBeforeTxSender).toBeGreaterThan(BigInt(0))
			const rawTx = tx(senderContext, hexAddress(senderAccount.address), balanceToTransfer.toString())

			const events1 = await sendTransaction(rawTx.signAsync(senderAccount))

			// process tx
			await createBlock(senderContext)
			// process msg
			await createBlock(receiverContext)

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
