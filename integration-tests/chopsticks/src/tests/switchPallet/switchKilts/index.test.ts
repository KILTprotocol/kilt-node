import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, setStorage } from '../../../network/utils.js'
import { calculateTxFees, hexAddress } from '../../../helper/utils.js'
import { testCases } from './config.js'
import { Config } from '../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../../network/utils.js'
import { checkSwitchPalletInvariant, getPoolAccount, getRemoteLockedSupply } from '../index.js'

describe.each(testCases)(
	'Switch KILTs',
	{ timeout: 30_000 },
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

		it(desc, { timeout: 10_000 }, async ({ expect }) => {
			const { checkEvents, checkSystemEvents } = withExpect(expect)

			const poolAccount = await getPoolAccount(senderContext)

			// check initial state
			const initialRemoteLockedSupply = await getRemoteLockedSupply(senderContext)
			const initialBalanceReceiverChain = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			const initialBalanceSenderChain = await query.sender(senderContext, hexAddress(senderAccount.address))
			const initialBalancePoolAccount = await query.sender(senderContext, poolAccount)
			const initialBalanceUserSenderChain = await query.sender(senderContext, hexAddress(senderAccount.address))

			expect(initialBalanceReceiverChain).toBe(BigInt(0))
			expect(initialBalanceSenderChain).toBeGreaterThan(BigInt(0))

			const { balanceToTransfer, events, tx } = txContext

			const rawTx = tx(senderContext, hexAddress(senderAccount.address), balanceToTransfer.toString())

			const eventsResult = await sendTransaction(rawTx.signAsync(senderAccount))

			// process tx
			await createBlock(senderContext)
			// process xcm message
			await createBlock(receiverContext)

			// check balance movement
			const txFees = await calculateTxFees(rawTx, senderAccount)
			const remoteLockedSupply = await getRemoteLockedSupply(senderContext)
			const balanceSenderChain = await query.sender(senderContext, hexAddress(senderAccount.address))
			const balancePoolAccount = await query.sender(senderContext, poolAccount)
			const balanceUserReceiverChain = await query.receiver(receiverContext, hexAddress(senderAccount.address))

			expect(initialBalanceUserSenderChain - balanceToTransfer - txFees).toBe(balanceSenderChain)
			expect(initialBalancePoolAccount).toBe(balancePoolAccount - balanceToTransfer)
			expect(balanceUserReceiverChain).toBeGreaterThan(BigInt(0))
			expect(remoteLockedSupply).toBe(initialRemoteLockedSupply - balanceToTransfer)

			// check events
			events.sender.map(
				async (pallet) =>
					await checkEvents(eventsResult, pallet).toMatchSnapshot(
						`Switch KILTs sender chain: ${JSON.stringify(pallet)}`
					)
			)

			events.receiver.map(
				async (pallet) =>
					await checkSystemEvents(receiverContext, pallet).toMatchSnapshot(
						`Switch KILTs receiver chain: ${JSON.stringify(pallet)}`
					)
			)

			// finalize switch
			await createBlock(senderContext)
			await checkSystemEvents(senderContext, 'assetSwitchPool1').toMatchSnapshot('AssetSwitchPool1 Finalization')

			await checkSwitchPalletInvariant(
				expect,
				senderContext,
				receiverContext,
				sovereignAccount.receiver,
				query.sender,
				query.receiver
			)
		})
	}
)
