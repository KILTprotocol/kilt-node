import { Keyring } from '@polkadot/keyring'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'

const keyring = new Keyring({ type: 'ed25519', ss58Format: 38 })

export const keysAlice = keyring.addFromUri('//alice', undefined, 'ed25519')
export const keysBob = keyring.addFromUri('//bob', undefined, 'ed25519')
export const keysCharlie = keyring.addFromUri('//charlie', undefined, 'ed25519')

console.log('alice', u8aToHex(decodeAddress('4qPZ8fv6BjGoGKzfx5LtBFnEUp2b5Q5C1ErrjBNGmoFTLNHG')))
console.log('alice 2', u8aToHex(decodeAddress(keysAlice.address)))
