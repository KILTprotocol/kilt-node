// TODO remove
/* eslint-disable @typescript-eslint/no-unused-vars */

import * as PolkadotChainConfigs from '../../../network/index.js'
import { initialBalanceKILT, keysAlice, keysBob } from '../../../helper/utils.js'
import * as SpiritnetConfig from '../../../network/spiritnet.js'
import * as HydraDxConfig from '../../../network/hydraDx.js'
import { tx, query } from '../../../helper/api.js'

import type { ApiPromise } from '@polkadot/api'
import type { BasicConfig, BasicXcmTestConfiguration, BasisTxContext } from '../types.js'
import type { SubmittableExtrinsic } from '@polkadot/api/types'

interface Config extends BasicConfig {
	precision: bigint
}

interface Query {
	sender: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
	receiver: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

interface TxContext extends BasisTxContext {
	balanceToTransfer: bigint
	tx: ({ api }: { api: ApiPromise }, submitter: string, amount: string | number) => SubmittableExtrinsic<'promise'>
}

/*
 * Configuration for the LimitedReserveTransfer test extends the BasicXcmTestConfiguration
 **/
interface LimitedReserveTestConfiguration extends BasicXcmTestConfiguration {
	config: Config
	query: Query
	txContext: TxContext
}
