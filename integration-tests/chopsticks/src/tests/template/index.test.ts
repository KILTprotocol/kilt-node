import { describe, beforeEach, it, afterEach } from 'vitest'
import type { KeyringPair } from '@polkadot/keyring/types'
import { setupContext } from '@acala-network/chopsticks-testing'

import { setStorage } from '../../network/utils.js'
import { testCases } from './config.js'
import type { Config } from '../../network/types.js'
import { shutDownNetwork } from '../../network/utils.js'

describe.skip.each(testCases)('TEMPLATE', async ({ account, config }) => {
	let senderContext: Config
	let senderAccount: KeyringPair
	const { desc, network, storage } = config

	// Create the network context
	beforeEach(async () => {
		const { parachains } = network
		senderContext = await setupContext(parachains[0])

		const { senderStorage } = storage
		await setStorage(senderContext, senderStorage)
		senderAccount = account
	})

	// Shut down the network
	afterEach(async () => {
		try {
			await shutDownNetwork([senderContext])
		} catch (error) {
			if (!(error instanceof TypeError)) {
				console.error(error)
			}
		}
	})

	it(desc, async ({ expect }) => {
		expect(senderAccount).toBeTruthy()
	})
})
