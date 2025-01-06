import * as PolkadotChainConfigs from '../../../network/index.js'
import { initialBalanceKILT, keysAlice, keysBob } from '../../../helper/utils.js'
import * as SpiritnetConfig from '../../../network/spiritnet.js'
import * as HydraDxConfig from '../../../network/hydraDx.js'
import * as AssetHubContext from '../../../network/assethub.js'
import { tx, query } from '../../../helper/api.js'

import type { ApiPromise } from '@polkadot/api'
import type { BasicConfig, BasicXcmTestConfiguration, BasisTxContext } from '../../types.js'
import type { SubmittableExtrinsic } from '@polkadot/api/types'


interface Query {
    sender: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
    receiver: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

interface TxContext extends BasisTxContext {
    balanceToTransfer: bigint
    tx: ({ api }: { api: ApiPromise }, submitter: string, amount: string | number) => SubmittableExtrinsic<'promise'>
}

/*
 * Configuration for Swtichting coins. 
 **/
interface SwitchTestConfiguration extends BasicXcmTestConfiguration {
    config: BasicConfig
    query: Query
    txContext: TxContext
}

// Test pairs for limited reserve transfers
export const testPairsLimitedReserveTransfers: SwitchTestConfiguration[] = [
    {
        config: {
            desc: 'Kilt -> AssetHub -> Kilt',
        },

        network: {
            sender: PolkadotChainConfigs.all.spiritnet.getConfig({}),
            receiver: PolkadotChainConfigs.all.assetHub.getConfig({}),
            relay: PolkadotChainConfigs.all.polkadot.getConfig({}),
        },
        accounts: {
            senderAccount: keysAlice,
            receiverAccount: keysAlice,
        },
        query: {
            sender: query.balances,
            receiver: query.tokens(HydraDxConfig.kiltTokenId),
        },
        txContext: {
            tx: tx.xcmPallet.limitedReserveTransferAssetsV2(
                SpiritnetConfig.KILT,
                tx.xcmPallet.parachainV2(1, HydraDxConfig.paraId)
            ),
            pallets: {
                sender: ['xcmpQueue', 'polkadotXcm'],
                receiver: ['xcmpQueue'],
            },
            balanceToTransfer: BigInt(1e15),
        },
        storage: {
            senderStorage: SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
            receiverStorage: {
                // Assign some coins to create the account. 
                ...AssetHubContext.assignDotTokensToAccountsAsStorage([keysAlice.address]),
                // Create the eKilts. 
                ...AssetHubContext.createForeignAsset(keysBob.address)
            },
            relayStorage: {},
        },
        sovereignAccount: {
            sender: HydraDxConfig.siblingSovereignAccount,
            receiver: SpiritnetConfig.siblingSovereignAccount,
        },
    },
] as const
