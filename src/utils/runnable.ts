import { createLogger } from '~/logger'
import {
  Procedure,
} from './types'
import { Callbacks } from './callbacks'
import {
  ArgsParserBuilder,
  NeedHelpError,
} from './args-parser'

const log = createLogger('runnable')

class AppBuilder {
  addCommand(command: Command, cb: () => void) {

  }

  create() {

  }
}

export function createRunnable<CT extends object, C extends keyof CT>(
  appName: string,
  argsParser: ArgsParserBuilder<CT, C>,
  run: (command: C, options: CT[C], onExit: (cb: Procedure) => void) => void,
) {
  const argsArr = process.argv.slice(2)

  let args: [C, CT[C]]
  try {
    args = argsParser.parse(argsArr)
  } catch (e) {
    if (e instanceof NeedHelpError) {
      // tslint:disable-next-line:no-console
      console.log(argsParser.getHelp(appName))
      process.exit(0)
    } else {
      log.error(`Failed to parse args "${argsArr}":`, e)
      process.exit(3)
    }
  }

  const callbacks = new Callbacks()

  const onExit = (cb: Procedure) => callbacks.add(cb)

  Promise.resolve(run(args[0], args[1], onExit)).catch((e) => {
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
