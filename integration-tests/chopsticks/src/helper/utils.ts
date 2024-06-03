import { Keyring } from '@polkadot/keyring'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'
import { ExpectStatic } from 'vitest'

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

export function hexAddress(addr: string) {
	return u8aToHex(decodeAddress(addr))
}

export function validateBalanceWithPrecision(
	previousBalance: bigint,
	receivedBalance: bigint,
	removedBalance: bigint,
	expect: ExpectStatic,
	precision: bigint
) {
	if (precision < BigInt(0) || precision > BigInt(100)) {
		throw new Error('Precision must be between 0 and 100')
	}

	const allowedError = BigInt(100) - precision
	const lowerBound = previousBalance - (previousBalance * allowedError) / BigInt(100)
	const upperBound = previousBalance + (previousBalance * allowedError) / BigInt(100)

	const newBalance = previousBalance + receivedBalance - removedBalance

	expect(newBalance).toBeGreaterThanOrEqual(lowerBound)
	expect(newBalance).toBeLessThanOrEqual(upperBound)
}

export const KILT = BigInt(1e15)
export const DOT = BigInt(1e10)
export const HDX = BigInt(1e12)

export const initialBalanceKILT = BigInt(100) * KILT
export const initialBalanceDOT = BigInt(100) * DOT
export const initialBalanceHDX = BigInt(100) * HDX
