import { Config } from './types'

export default {
  polkadot: {
    name: 'spiritnet' as const,
    endpoint: 'wss://kilt-rpc.dwellir.com',
  },
  config: ({ alice }) => ({
    storages: {
      System: {
        Account: [[[alice.address], { providers: 1, data: { free: '1000000000000000000000' } }]],
      },

    },
  }),
} satisfies Config

export const spiritnet = {
  paraId: 2086,
}
