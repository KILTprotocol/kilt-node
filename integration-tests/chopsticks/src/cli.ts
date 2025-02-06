import { Command } from 'commander'

import { createTestNetwork, scheduleTxCommand, stateTransition } from './command/index.js'

const program = new Command()

program.name('chopsticks utils').description('CLI to mange some chopsticks instances').version('0.0.1')

program
	.command('spinUp')
	.description('Spin up the network')
	.action(async () => {
		await createTestNetwork()
	})

program
	.command('scheduleTx')
	.description('Executes a transaction on the network')
	.argument('<endpoint>', 'The endpoint of the network')
	.argument('<rawTx>', 'The raw transaction to execute')
	.option(
		'--origin',
		'The origin of the transaction Either "Root" or "Signed". Default is "Root"',
		(value) => {
			if (value !== 'Root' && value !== 'Signed') {
				throw new Error('Invalid origin. Must be either "Root" or "Signed"')
			}
			return value
		},
		'Root'
	)
	.option('--port', 'The RPC port', '8888')
	.action(async (endpoint, rawTx, options) => {
		const { origin, port } = options

		await scheduleTxCommand(endpoint, rawTx, origin, +port)
	})

program
	.command('stateTransition')
	.description('Shows the state transition of the network by the latest block')
	.argument('<endpoint>', 'The endpoint of the network')
	.option('--block', 'The block number to do the state transition', 'undefined')
	.action(async (endpoint, options) => {
		const { block } = options
		const blockNumber = block === 'undefined' ? undefined : +block
		await stateTransition(endpoint, blockNumber)
	})

program.parse()
