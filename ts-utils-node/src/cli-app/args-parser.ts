import { Command } from './command'
import { Obj } from '@v/utils'

export class NeedHelpError extends Error { }

export class ArgsParserBuilder<CT extends Obj, C extends keyof CT> {
  private constructor(
    private _help: boolean,
    private _commmands: Array<Command<C, any>>,
  ) { }

  static create(help = true) {
    return new ArgsParserBuilder(help, [])
  }

  addCommand<C1 extends string, CO1 extends Obj>(command: Command<C1, CO1>) {
    return new ArgsParserBuilder<CT & { [key in C1]: CO1 }, C | C1>(
      this._help,
      [...this._commmands, command],
    )
  }

  getHelp(appName: string): string {
    const help = this._commmands
      .map(command => command.getHelp(appName))

    help.unshift(`${appName} usage:`)

    return help.join('\n\n')
  }

  private _findCommand(commandName: string): Command<C, any> | undefined {
    const command = this._commmands.find(item => item.name === commandName)
    if (command) {
      return command
    }

    const emptyCommand = this._commmands.find(item => item.name === '')
    if (emptyCommand) {
      return emptyCommand
    }

    return undefined
  }

  parse(args: string[]): [C, CT[C]] {
    if (!this._commmands.length) {
      throw new Error('no command has been configured')
    }

    if (this._help && args.includes('--help')) {
      throw new NeedHelpError()
    }

    const commandName = args[0] || ''
    const command = this._findCommand(commandName)
    if (!command) {
      throw new Error(`got unexpected command "${commandName}" and no empty command has been specified`)
    }

    const options = command.parseOptions(args.slice(command.name ? 1 : 0)) as CT[C]

    return [command.name, options]
  }
}
