import { execa } from 'execa'

export async function stateTransition(endpoint: string) {
	try {
		const { stdout } = await execa('yarn', [
			'chopsticks',
			'run-block',
			`--endpoint=${endpoint}`,
			'--html',
			'--open',
		])
		console.log(stdout)
	} catch (error) {
		console.error('Error:', error)
	}
}
