import { createLogger } from '~/logger'
import {
  Procedure,
  AsyncProcedure,
} from './types'
import { Callbacks } from './callbacks'

const log = createLogger('runnable')

type Runnable = (args: string[], onExit: (cb: Procedure | AsyncProcedure) => void) => Promise<void> | void

export function createRunnable(run: Runnable) {
  const args = process.argv.slice(2)

  const callbacks = new Callbacks()

  const onExit = (cb: Procedure | AsyncProcedure) => callbacks.add(cb)

  Promise.resolve(run(args, onExit)).catch((e) => {
    log.error('runnable: process failed', e)

    process.exit(2)
  })

  process.on('SIGINT', () => {
    log.debug('got SIGINT')
    callbacks.runAll(true)
  })

  process.on('SIGTERM', () => {
    log.debug('got SIGTERM')
    callbacks.runAll(true)
  })

  process.on('exit', (code: number) => {
    log.debug(`got exit code=${code}`)
    callbacks.runAll(true)
  })
}
