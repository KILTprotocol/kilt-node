import type { setupContext } from '@acala-network/chopsticks-testing'

export type Config = Awaited<ReturnType<typeof setupContext>>
