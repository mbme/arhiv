/* eslint-disable no-console */
import {
  Dict,
  Procedure,
  AsyncProcedure,
  Callbacks,
  Obj,
} from '@v/utils'
import {
  ArgsParserBuilder,
  NeedHelpError,
} from './args-parser'
import { Command } from './command'

type OnExit = (cb: Procedure | AsyncProcedure) => void
type Runnable<O extends Obj> = (options: O, onExit: OnExit) => void | Promise<void>

export class CliApp {
  private _argsParser: ArgsParserBuilder<any, any>
  private _runnables: Dict<Runnable<any>> = {}
  private _callbacks = new Callbacks()

  private constructor(
    private _appName: string,
    private _help: boolean,
  ) {
    this._argsParser = ArgsParserBuilder.create(_help)
  }

  static create(appName: string, help = true) {
    return new CliApp(appName, help)
  }

  addCommand<O extends Obj>(command: Command<string, O>, cb: Runnable<O>) {
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
        console.log(this._argsParser.getHelp(this._appName))
        console.log('')
        process.exit(0)
      } else {
        console.error(`Failed to parse args "${argsArr}": ${e}`)
        if (this._help) {
          console.error('Too see usage use --help')
        }
        console.error('')
        process.exit(3)
      }
    }

    const onExit = (cb: Procedure) => this._callbacks.add(cb)

    const run = this._runnables[args[0]]

    Promise.resolve(run(args[1], onExit)).catch((e) => {
      console.error('process failed', e)

      process.exit(2)
    })

    process.on('SIGINT', () => {
      console.debug('got SIGINT')
      this._callbacks.runAll(true)
    })

    process.on('SIGTERM', () => {
      console.debug('got SIGTERM')
      this._callbacks.runAll(true)
    })

    process.on('exit', (code: number) => {
      console.debug(`got exit code=${code}`)
      this._callbacks.runAll(true)
    })
  }
}
