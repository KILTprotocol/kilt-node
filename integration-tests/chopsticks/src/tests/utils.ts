import { ExpectStatic } from 'vitest'
import { setTimeout } from 'timers/promises'

import { Config } from '../network/types.js'

export async function createBlock(context: Config) {
	// fixes api runtime disconnect warning
	await setTimeout(50)
	await context.dev.newBlock()
}

export async function setStorage(context: Config, storage: { [key: string]: any }) {
	await context.dev.setStorage(storage)
	await createBlock(context)
}

export async function checkBalanceAndExpectZero(
	getFreeBalanceFunction: (account: string) => Promise<bigint>,
	account: string,
	expect: ExpectStatic
) {
	const balance = await getFreeBalanceFunction(account)
	expect(balance).eq(BigInt(0))
}

export async function checkBalanceAndExpectAmount(
	getFreeBalanceFunction: (account: string) => Promise<bigint>,
	account: string,
	expect: ExpectStatic,
	expectedAmount: bigint
) {
	const balance = await getFreeBalanceFunction(account)
	expect(balance).eq(BigInt(expectedAmount))
}
