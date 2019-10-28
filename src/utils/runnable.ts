import { Procedure } from './types'
import { createLogger } from './logger'

const log = createLogger('runnable')

export function createRunnable(run: (...args: string[]) => Promise<void> | void) {
  const args = process.argv.slice(3)

  Promise.resolve(run(...args)).catch((e) => {
    log.error('runnable: process failed', e)

    process.exit(2)
  })

  process.on('SIGINT', () => log.debug('got SIGINT'))
  process.on('SIGTERM', () => log.debug('got SIGTERM'))
  process.on('exit', (code: number) => log.debug(`got exit code=${code}`))
}

export function onTermination(cb: Procedure) {
  let terminated = false
  const handler = () => {
    if (!terminated) {
      terminated = true
      cb()
    }
  }

  process.on('SIGINT', handler)
  process.on('SIGTERM', handler)
  process.on('exit', handler)
}
