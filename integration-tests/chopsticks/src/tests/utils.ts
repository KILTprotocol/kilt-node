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

export function hexAddress(addr: string) {
	return u8aToHex(decodeAddress(addr))
}
