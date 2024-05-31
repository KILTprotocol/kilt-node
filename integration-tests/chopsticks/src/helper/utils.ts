import { Keyring } from '@polkadot/keyring'

const keyring = new Keyring({ type: 'ed25519', ss58Format: 38 })

export const keysAlice = keyring.addFromUri('//alice', undefined, 'ed25519')
export const keysBob = keyring.addFromUri('//bob', undefined, 'ed25519')
export const keysCharlie = keyring.addFromUri('//charlie', undefined, 'ed25519')

export function toNumber(value: string | undefined): number | undefined {
	if (value === undefined) {
		return undefined
	}

	return Number(value)
}

export const KILT = BigInt(1e15)
export const DOT = BigInt(1e10)
export const HDX = BigInt(1e12)

export const initialBalanceKILT = BigInt(100) * KILT
export const initialBalanceDOT = BigInt(100) * DOT
export const initialBalanceHDX = BigInt(100) * HDX
