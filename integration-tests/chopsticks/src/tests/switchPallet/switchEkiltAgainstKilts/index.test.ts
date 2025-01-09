import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, setStorage } from '../../../network/utils.js'
import { calculateTxFees, getPaidXcmFees, hexAddress } from '../../../helper/utils.js'
import { testPairsSwitchFunds } from './config.js'
import { Config } from '../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../../network/utils.js'
import { checkSwitchPalletInvariant, getPoolAccount, getRemoteLockedSupply } from '../index.js'

describe.each(testPairsSwitchFunds)(
	'Switch EKILTs',
	{ timeout: 30_00000 },
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
		}, 20_00000)

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

		it(desc, { timeout: 10_00000, retry: 0 }, async ({ expect }) => {
			const { checkEvents, checkSystemEvents } = withExpect(expect)

			const poolAccount = await getPoolAccount(receiverContext)

			// check initial state

			const balanceBeforeTxReceiverChain = await query.receiver(
				receiverContext,
				hexAddress(senderAccount.address)
			)
			const balanceBeforeTxSenderChain = await query.sender(senderContext, hexAddress(senderAccount.address))

			expect(balanceBeforeTxReceiverChain).toBe(BigInt(0))
			expect(balanceBeforeTxSenderChain).toBeGreaterThan(BigInt(0))

			const initialBalancePoolAccount = await query.receiver(receiverContext, poolAccount)
			const initialBalanceSovereignAccount = await query.sender(senderContext, sovereignAccount.sender)
			const initalBalanceUser = await query.sender(senderContext, hexAddress(senderAccount.address))
			const initialRemoteLockedSupply = await getRemoteLockedSupply(receiverContext)

			const { balanceToTransfer, events, tx } = txContext

			const signedTx3 = tx(senderContext, balanceToTransfer.toString()).signAsync(senderAccount)

			const events3 = await sendTransaction(signedTx3)

			// process tx
			await createBlock(senderContext)
			// process xcm message
			await createBlock(receiverContext)

			// check balance movement

			const nativeBalanceForeignChainAfterx = await query.sender(senderContext, sovereignAccount.sender)
			const balanceAfterTxSenderChain = await query.sender(senderContext, hexAddress(senderAccount.address))

			expect(initialBalanceSovereignAccount + balanceToTransfer).toBe(nativeBalanceForeignChainAfterx)
			expect(initalBalanceUser - balanceToTransfer).toBe(balanceAfterTxSenderChain)

			await receiverContext.pause()

			// check events

			events.sender.map(
				async (pallet) =>
					await checkEvents(events3, pallet).toMatchSnapshot(
						`Withdraw native funds on foreign chain ${JSON.stringify(pallet)}`
					)
			)

			// events.native.receive.native.map(
			// 	async (pallet) =>
			// 		await checkSystemEvents(nativeContext, pallet).toMatchSnapshot(
			// 			`Receive native funds on native chain ${JSON.stringify(pallet)}`
			// 		)
			//)

			// finalize the switch. Create a another block to process the query xcm message
			// await createBlock(receiverContext)
			// checkSwitchPalletInvariant(
			// 	expect,
			// 	receiverContext,
			// 	senderContext,
			// 	sovereignAccount.sender,
			// 	query.receiver,
			// 	query.sender
			// )
		})
	}
)
