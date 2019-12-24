import { createLogger } from '~/logger'
import {
  Dict,
  Procedure,
} from '../types'
import { Callbacks } from '../callbacks'
import {
  ArgsParserBuilder,
  NeedHelpError,
} from './args-parser'
import { Command } from './command'

const log = createLogger('cli-app')

type Runnable<O extends object> = (options: O, onExit: (cb: Procedure) => void) => void

export class CliApp {
  private _argsParser: ArgsParserBuilder<any, any>
  private _runnables: Dict<Runnable<any>> = {}
  private _callbacks = new Callbacks()

  private constructor(
    private _appName: string,
    help: boolean,
  ) {
    this._argsParser = ArgsParserBuilder.create(help)
  }

  static create(appName: string, help = true) {
    return new CliApp(appName, help)
  }

  addCommand<O extends object>(command: Command<string, O>, cb: Runnable<O>) {
    this._argsParser = this._argsParser.addCommand(command)
    this._runnables[command.name] = cb

    return this
  }

  run() {
    const argsArr = process.argv.slice(2)

    let args: [string, any]
    try {
      args = this._argsParser.parse(argsArr)
    } catch (e) {
      if (e instanceof NeedHelpError) {
        // tslint:disable-next-line:no-console
        console.log(this._argsParser.getHelp(this._appName))
        process.exit(0)
      } else {
        log.error(`Failed to parse args "${argsArr}":`, e)
        process.exit(3)
      }
    }

    const onExit = (cb: Procedure) => this._callbacks.add(cb)

    const run = this._runnables[args[0]]

    Promise.resolve(run(args[1], onExit)).catch((e) => {
      log.error('runnable: process failed', e)

      process.exit(2)
    })

    process.on('SIGINT', () => {
      log.debug('got SIGINT')
      this._callbacks.runAll(true)
    })

    process.on('SIGTERM', () => {
      log.debug('got SIGTERM')
      this._callbacks.runAll(true)
    })

    process.on('exit', (code: number) => {
      log.debug(`got exit code=${code}`)
      this._callbacks.runAll(true)
    })
  }
}
