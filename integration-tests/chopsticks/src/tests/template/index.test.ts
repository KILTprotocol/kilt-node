import { describe, beforeEach, it, afterEach } from 'vitest'
import type { KeyringPair } from '@polkadot/keyring/types'

import { testCases } from './config.js'
import type { Config } from '../../network/types.js'
import { spinUpNetwork, tearDownNetwork } from '../utils.js'

describe.skip.each(testCases)('TEMPLATE', ({ account, config }) => {
	let senderContext: Config
	let senderAccount: KeyringPair
	const { desc } = config

	// Create the network context
	beforeEach(async () => {
		const { relayChainContext } = await spinUpNetwork(config)
		senderContext = relayChainContext
		senderAccount = account
	})

	// Shut down the network
	afterEach(async () => await tearDownNetwork([senderContext]))

	it(desc, async ({ expect }) => {
		expect(senderAccount).toBeTruthy()
	})
})
