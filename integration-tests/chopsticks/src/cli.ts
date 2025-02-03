import { Command } from 'commander'

import { createTestNetwork, scheduleTxCommand } from './command/index.js'

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
	.option('--origin <String>', 'The origin of the transaction', 'Root')
	.option('--port <Number>', 'The RPC port', '8888')
	.action(async (endpoint, rawTx, options) => {
		const { origin, port } = options
		console.log(endpoint, rawTx, origin, port)
		await scheduleTxCommand(endpoint, rawTx, origin, +port)
	})

program.parse()
