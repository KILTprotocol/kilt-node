import { createBlock, scheduleTx } from '../network/utils.js'
import { setupContext } from '@acala-network/chopsticks-testing'

type Origin = 'Root' | 'Signed'

export async function scheduleTxCommand(endpoint: string, rawTx: string, origin: Origin, port: number) {
	const context = await setupContext({ endpoint, port })

	await scheduleTx(context, rawTx, undefined, origin)
	await createBlock(context)

	console.log('Transaction scheduled')
	await context.pause()
}
