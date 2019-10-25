// tslint:disable:no-console
import { Procedure } from './types'

export function createRunnable(run: (...args: string[]) => Promise<void> | void) {
  const args = process.argv.slice(3)

  Promise.resolve(run(...args)).catch((e) => {
    console.error('runnable: process failed', e)

    process.exit(2)
  })

  process.on('SIGINT', () => console.error('runnable: got SIGINT'))
  process.on('SIGTERM', () => console.error('runnable: got SIGTERM'))
  process.on('exit', (code: number) => console.error(`runnable: got exit code=${code}`))
}

export function onTermination(cb: Procedure) {
  process.on('SIGINT', cb)
  process.on('SIGTERM', cb)
  process.on('exit', cb)
}
