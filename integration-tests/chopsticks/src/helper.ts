import { Keyring } from '@polkadot/keyring'
import { expect } from 'vitest'
import { withExpect } from '@acala-network/chopsticks-testing'

const keyring = new Keyring({ type: 'sr25519', ss58Format: 2 })
keyring.setSS58Format(38)

export const keysAlice = keyring.addFromUri('//alice', undefined, 'sr25519')
export const keysBob = keyring.addFromUri('//bob', undefined, 'sr25519')
export const keysCharlie = keyring.addFromUri('//charlie', undefined, 'sr25519')

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
