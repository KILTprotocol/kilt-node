import { ExpectStatic } from 'vitest'
import { setTimeout } from 'timers/promises'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'

import { Config } from '../network/types.js'

/// Creates a new block for the given context
export async function createBlock(context: Config) {
	// fixes api runtime disconnect warning
	await setTimeout(50)
	await context.dev.newBlock()
}

/// sets the storage for the given context.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function setStorage(context: Config, storage: { [key: string]: any }) {
	await context.dev.setStorage(storage)
	await createBlock(context)
}

/// checks the balance of an account and expects it to be the given amount
export async function checkBalance(
	getFreeBalanceFunction: (account: string) => Promise<bigint>,
	account: string,
	expect: ExpectStatic,
	expectedAmount = BigInt(0)
) {
	const balance = await getFreeBalanceFunction(account)
	expect(balance).eq(BigInt(expectedAmount))
}

/// checks the balance of an account and expects it to be in the given range
export async function checkBalanceInRange(
	getFreeBalanceFunction: (account: string) => Promise<bigint>,
	account: string,
	expect: ExpectStatic,
	expectedRange: [bigint, bigint]
) {
	const balance = await getFreeBalanceFunction(account)
	expect(balance >= expectedRange[0])
	expect(balance <= expectedRange[1])
}

export function hexAddress(addr: string) {
	return u8aToHex(decodeAddress(addr))
}
