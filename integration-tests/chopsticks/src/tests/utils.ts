import { expect } from 'vitest'
import { withExpect } from '@acala-network/chopsticks-testing'

const { check, checkEvents, checkHrmp, checkSystemEvents, checkUmp } = withExpect((x: any) => ({
	toMatchSnapshot(msg?: string): void {
		expect(x).toMatchSnapshot(msg)
	},
	toMatch(value: any, _msg?: string): void {
		expect(x).toMatch(value)
	},
	toMatchObject(value: any, _msg?: string): void {
		expect(x).toMatchObject(value)
	},
}))

export { check, checkEvents, checkHrmp, checkSystemEvents, checkUmp }
