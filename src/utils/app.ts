import { createLogger } from '~/logger'
import {
  ArgsParserBuilder,
  Command,
  NeedHelpError,
} from './args-parser'
import {
  Dict,
  Procedure,
} from './types'
import { Callbacks } from './callbacks'

export { command } from './args-parser'

const log = createLogger('app')

type Runnable = (options: any, onExit: (cb: Procedure) => void) => void

export class App {
  private _argsParser: ArgsParserBuilder<any, any>

  private _commands: Dict<Runnable> = {}

  private _callbacks = new Callbacks()

  private constructor(
    private _appName: string,
    help: boolean,
  ) {
    this._argsParser = ArgsParserBuilder.create(help)
  }

  static create(appName: string, help = true) {
    return new App(appName, help)
  }

  addCommand<CO1 extends object>(command: Command<string, CO1>, cb: (o: CO1) => void) {
    this._argsParser = this._argsParser.addCommand(command)
    this._commands[command.name] = cb

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

    const run = this._commands[args[0]]

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
