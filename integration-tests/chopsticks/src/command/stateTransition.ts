import { execa } from 'execa'

export async function stateTransition(endpoint: string, blockNumber?: number) {
	const options = ['chopsticks', 'run-block', `--endpoint=${endpoint}`, '--html', '--open']

	if (blockNumber) {
		options.push(`--block=${blockNumber}`)
	}

	await execa('yarn', options)
		.then(({ stdout }) => {
			console.log(stdout)
		})
		.catch((error) => {
			console.error('Error:', error)
		})
}
