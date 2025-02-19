import { ApiPromise, SubmittableResult } from '@polkadot/api'
import { Keyring } from '@polkadot/keyring'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'

import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'
import type { Codec } from '@polkadot/types/types'
import type { ExpectStatic } from 'vitest'
import { EventRecord } from '@polkadot/types/interfaces'

/**
 * Keyring instance to generate key pairs with 38 ss58Format encoding.
 */
const keyring = new Keyring({ type: 'ed25519', ss58Format: 38 })

/**
 * Key pairs for Alice, Bob, and Charlie used for testing.
 */
export const keysAlice = keyring.addFromUri('//alice', undefined)
export const keysBob = keyring.addFromUri('//bob', undefined)
export const keysCharlie = keyring.addFromUri('//charlie', undefined)

/**
 * Converts a given value to a Number
 */
export function toNumber(value: string | undefined): number | undefined {
	if (value === undefined) {
		return undefined
	}
	return Number(value)
}

/**
 * Converts a given address to its hexadecimal representation.
 *
 * @param addr - The address to be converted. It should be a string representation of an address.
 */
export function hexAddress(addr: string) {
	return u8aToHex(decodeAddress(addr))
}

/**
 * Validates if the received balance is within the expected range considering a certain precision.
 *
 * @param previousBalance - The balance before the operation.
 * @param receivedBalance - The balance after the operation.
 * @param deltaBalance - The expected change in balance.
 * @param expect - The assertion function from the testing library.
 * @param precision - The precision of the balance validation. It must be a value between 0 and 100.
 *
 * The function calculates the expected balance by adding the deltaBalance to the previousBalance.
 * It then calculates the lower and upper bounds of the expected balance considering the precision.
 * The received balance is then checked if it falls within the range of the lower and upper bounds.
 *
 * If the precision is not between 0 and 100, the function throws an error.
 *
 * @throws {Error} If the precision is not between 0 and 100.
 */
export function validateBalanceWithPrecision(
	previousBalance: bigint,
	receivedBalance: bigint,
	deltaBalance: bigint,
	expect: ExpectStatic,
	precision: bigint
) {
	if (precision < 0n || precision > 100n) {
		throw new Error('Precision must be between 0 and 100')
	}

	const allowedError = 100n - precision
	const expectedBalance = previousBalance + deltaBalance

	const lowerBound = expectedBalance - (expectedBalance * allowedError) / 100n
	const upperBound = expectedBalance + (expectedBalance * allowedError) / 100n

	expect(receivedBalance).toBeGreaterThanOrEqual(lowerBound)
	expect(receivedBalance).toBeLessThanOrEqual(upperBound)
}

/**
 * Fetches the paid fees for the executed XCM message. Is only working on the sender chain.
 */
export async function getPaidXcmFees(api: ApiPromise, eventRecords: Codec[]): Promise<bigint> {
	const polkadotFees = eventRecords.find((eventRecord) =>
		api.events.polkadotXcm.FeesPaid.is((eventRecord as EventRecord).event)
	)
	if (!polkadotFees) {
		throw new Error('FeesPaid event not found')
	}

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	return (polkadotFees as any).event.data.fees[0].fun.asFungible.toBigInt()
}

/**
 * Calculates the transaction fees for a given transaction.
 */
export async function calculateTxFees(
	tx: SubmittableExtrinsic<'promise', SubmittableResult>,
	account: KeyringPair
): Promise<bigint> {
	const paymentInfo = await tx.paymentInfo(account)
	return paymentInfo.partialFee.toBigInt()
}

// Constants for the balances
export const KILT = BigInt(1e15)
export const DOT = BigInt(1e10)
export const HDX = BigInt(1e12)

export const initialBalanceKILT = 100n * KILT
export const initialBalanceDOT = 100n * DOT
export const initialBalanceHDX = 100n * HDX
