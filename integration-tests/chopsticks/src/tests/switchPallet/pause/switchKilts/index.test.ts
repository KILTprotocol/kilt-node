import { describe, beforeEach, it, afterEach } from 'vitest'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, setStorage } from '../../../../network/utils.js'
import { testCases } from './config.js'
import { Config } from '../../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../../../network/utils.js'
import { hexAddress } from '../../../../helper/utils.js'

describe.skip.each(testCases)(
	'Switch KILTs while paused',
	{ timeout: 30_000 },
	async ({ account, txContext, config }) => {
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

		it(
			desc,
			async ({ expect }) => {
				const { balanceToTransfer, tx } = txContext
				let section: string = ''
				let errorName: string = ''

				// This should fail.
				await tx(senderContext, hexAddress(senderAccount.address), balanceToTransfer.toString()).signAndSend(
					senderAccount,
					({ dispatchError }) => {
						if (dispatchError) {
							const decoded = senderContext.api.registry.findMetaError(dispatchError.asModule)
							section = decoded.section
							errorName = decoded.name
						}
					}
				)

				await createBlock(senderContext)

				expect(section).toBe('assetSwitchPool1')
				expect(errorName).toBe('SwitchPairNotEnabled')
			},
			30_000
		)
	}
)
