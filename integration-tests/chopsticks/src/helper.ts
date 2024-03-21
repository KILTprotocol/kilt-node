import { Keyring } from '@polkadot/keyring'

// Random mnemonic

const keyring = new Keyring({ type: 'sr25519', ss58Format: 2 })
keyring.setSS58Format(38)

export const keysAlice = keyring.addFromUri('//alice', undefined, 'sr25519')
export const keysBob = keyring.addFromUri('//bob', undefined, 'sr25519')
export const keysCharlie = keyring.addFromUri('//charlie', undefined, 'sr25519')
