import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, scheduleTx } from '../../../network/utils.js'
import { hexAddress } from '../../../helper/utils.js'
import { testCases } from './config.js'
import type { Config } from '../../../network/types.js'
import { tx as txApi } from '../../../helper/api.js'
import { checkSwitchPalletInvariant, isSwitchPaused } from '../index.js'
import { spinUpNetwork, tearDownNetwork } from '../../utils.js'

describe.each(testCases)(
	'Reclaim trapped assets',

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

			const { tx, balanceToTransfer, events, reclaimTx, getXcmMessage, senderLocation } = txContext

			// precondition checks
			const balanceBeforeTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			const balanceBeforeTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceBeforeTx).toBe(0n)
			expect(balanceBeforeTxSender).toBeGreaterThan(0n)
			expect(await isSwitchPaused(receiverContext)).toBe(true)

			// action
			const rawTx = tx(senderContext, balanceToTransfer.toString())
			const events1 = await sendTransaction(rawTx.signAsync(senderAccount))

			// process tx
			await createBlock(senderContext)
			// process msg
			await createBlock(receiverContext)

			// check balance movement on sender chain.
			const balanceAfterTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceAfterTxSender).toBe(balanceBeforeTxSender - balanceToTransfer)

			// Tx should fail on receiver
			const balanceAfterTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			expect(balanceAfterTx).toBe(0n)

			// check events
			await Promise.all(
				events.sender.map((pallet) =>
					checkEvents(events1, pallet).toMatchSnapshot(
						`Switch eKILTs sender chain: ${JSON.stringify(pallet)}`
					)
				)
			)

			await checkSystemEvents(receiverContext, 'polkadotXcm').toMatchSnapshot(
				'AssetsTrapped event on receiver chain'
			)

			// enable the switch pair again
			const resumeTx = txApi.switchPallet.resume()(receiverContext)
			scheduleTx(receiverContext, resumeTx.method.toHex())
			// process scheduled tx
			await createBlock(receiverContext)

			// create reclaim Tx
			const xcmMessage = getXcmMessage(balanceToTransfer.toString(), senderAccount.address)
			const rawReclaimTx = reclaimTx(senderContext, xcmMessage)

			// create reclaim message for relay chain
			const transactMessage = [
				{ UnpaidExecution: { weightLimit: 'Unlimited' } },
				{
					Transact: {
						originKind: 'SuperUser',
						requireWeightAtMost: { refTime: '1000000000', proofSize: '65527' },
						call: {
							encoded: rawReclaimTx.method.toHex(),
						},
					},
				},
			]

			// TODO: make relay reclaim tx configurable in the config
			const reclaimTxRelay = relayContext.api.tx.xcmPallet.send({ V3: senderLocation }, { V3: transactMessage })
			await sendTransaction(relayContext.api.tx.sudo.sudo(reclaimTxRelay).signAsync(senderAccount))
			// process tx
			await createBlock(relayContext)

			// post condition checks

			// check if the tx was successful
			await checkSystemEvents(relayContext, 'xcmPallet').toMatchSnapshot('reclaim xcm message on relay chain')

			// process and send message on sender chain.
			await createBlock(senderContext)

			// check if the tx was successful
			await checkSystemEvents(senderContext, 'polkadotXcm').toMatchSnapshot('reclaim xcm message on sender chain')
			// process message  receiver chain
			await createBlock(receiverContext)

			// check events
			await Promise.all(
				events.receiver.map((pallet) =>
					checkSystemEvents(receiverContext, pallet).toMatchSnapshot(
						`reclaim trapped assets receiver chain: ${JSON.stringify(pallet)}`
					)
				)
			)

			await checkSwitchPalletInvariant(
				expect,
				receiverContext,
				senderContext,
				sovereignAccount,
				query.receiver,
				query.sender
			)
		})
	}
)
