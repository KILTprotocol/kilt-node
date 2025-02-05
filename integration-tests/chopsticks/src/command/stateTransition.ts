import { execa } from 'execa'

export async function stateTransition(endpoint: string, blockNumber?: number) {
	const options = ['chopsticks', 'run-block', `--endpoint=${endpoint}`, '--html', '--open']

	if (blockNumber) {
		options.push(`--block=${blockNumber}`)
	}

	try {
		const { stdout } = await execa('yarn', options)
		console.log(stdout)
	} catch (error) {
		console.error('Error:', error)
	}
}
