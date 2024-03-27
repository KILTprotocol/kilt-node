import { Keyring } from '@polkadot/keyring'

const keyring = new Keyring({ type: 'ed25519', ss58Format: 38 })

export const keysAlice = keyring.addFromUri('//alice', undefined, 'ed25519')
export const keysBob = keyring.addFromUri('//bob', undefined, 'ed25519')
export const keysCharlie = keyring.addFromUri('//charlie', undefined, 'ed25519')
