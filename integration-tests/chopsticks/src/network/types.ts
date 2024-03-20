import { SetupOption, testingPairs } from '@acala-network/chopsticks-testing'


export type NetworkKind = 'polkadot' | 'kusama'

export type NetworkConfig = {
  name: string
  endpoint: string | string[]
}

export type Context = ReturnType<typeof testingPairs>

export type FullContext = Context &
  NetworkConfig & {
    network: NetworkKind
  }

export type Config<T = object> = {
  polkadot?: NetworkConfig & T
  kusama?: NetworkConfig & T
  config(context: FullContext & T): {
    storages?: Record<string, Record<string, any>>
    options?: Partial<SetupOption>
  }
}
