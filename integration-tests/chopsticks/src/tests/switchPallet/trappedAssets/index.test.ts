import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, scheduleTx, setStorage } from '../../../network/utils.js'
import { hexAddress, keysAlice } from '../../../helper/utils.js'
import { testCases } from './config.js'
import type { Config } from '../../../network/types.js'
import { tx as txApi } from '../../../helper/api.js'
import { setupNetwork, shutDownNetwork } from '../../../network/utils.js'
import { skipTest } from '../../utils.js'

describe.skipIf(skipTest()).each(testCases)(
	'Reclaim trapped assets',
	{ timeout: 30_000 },
	async ({ account, query, txContext, config }) => {
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
		}, 20_000)

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

		it(desc, { timeout: 10_000, retry: 3 }, async ({ expect }) => {
			const { checkSystemEvents, checkEvents } = withExpect(expect)

			const { tx, balanceToTransfer, events, reclaimTx, getXcmMessage, senderLocation } = txContext

			// initial checks
			const balanceBeforeTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			const balanceBeforeTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceBeforeTx).toBe(BigInt(0))
			expect(balanceBeforeTxSender).toBeGreaterThan(BigInt(0))
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
			expect(balanceAfterTx).toBe(BigInt(0))

			// check events
			events.sender.map(
				async (pallet) =>
					await checkEvents(events1, pallet).toMatchSnapshot(
						`Switch eKILTs sender chain: ${JSON.stringify(pallet)}`
					)
			)

			await checkSystemEvents(receiverContext, 'polkadotXcm').toMatchSnapshot(
				'AssetsTrapped event on receiver chain'
			)

			// enable the switch pair again
			const resumeTx = txApi.switchPallet.resume()(receiverContext)
			scheduleTx(receiverContext, resumeTx)
			// process scheduled tx
			await createBlock(receiverContext)

			// create reclaim Tx
			const xcmMessage = getXcmMessage(balanceToTransfer.toString(), keysAlice.address)
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
			await sendTransaction(relayContext.api.tx.sudo.sudo(reclaimTxRelay).signAsync(keysAlice))
			// process tx
			await createBlock(relayContext)

			// check if the tx was successful
			await checkSystemEvents(relayContext, 'xcmPallet').toMatchSnapshot('reclaim xcm message on relay chain')

			// process and send message on sender chain.
			await createBlock(senderContext)

			// check if the tx was successful
			await checkSystemEvents(senderContext, 'polkadotXcm').toMatchSnapshot('reclaim xcm message on sender chain')
			// process message  receiver chain
			await createBlock(receiverContext)

			// check events
			events.receiver.map(
				async (pallet) =>
					await checkSystemEvents(receiverContext, pallet).toMatchSnapshot(
						`reclaim trapped assets receiver chain: ${JSON.stringify(pallet)}`
					)
			)
		})
	}
)
