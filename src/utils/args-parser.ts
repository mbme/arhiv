import { Result } from './types'

// global options
// if there is at least 1 command, command becomes required
// command options
// positionalOne
// positionalRest

interface ICommand<C extends string> {
  name: C
  description: string
}

interface IOption<O extends string> {
  name: O
  description: string
  assertValid?: (value: string) => void
}

type ArgsOptions<O extends string> = {
  [key in O]: string
}

interface IArgs<C extends string, O extends string> {
  command: C
  options: ArgsOptions<O>
}

class ArgsParserBuilder<C extends string, O extends string> {
  private constructor(
    private _commmands: ICommand<C>[],
    private _options: IOption<O>[],
    private _positionalOptions: IOption<O>[],
  ) { }

  static create() {
    return new ArgsParserBuilder([], [], [])
  }

  command<C1 extends string>(command: ICommand<C1>) {
    return new ArgsParserBuilder<C | C1, O>(
      [...(this._commmands || []), command],
      this._options,
      this._positionalOptions,
    )
  }

  option<O1 extends string>(option: IOption<O1>) {
    return new ArgsParserBuilder<C, O | O1>(
      this._commmands,
      [...this._options, option],
      this._positionalOptions,
    )
  }

  positionalOption<O1 extends string>(option: IOption<O1>) {
    return new ArgsParserBuilder<C, O | O1>(
      this._commmands,
      this._options,
      [...this._positionalOptions, option],
    )
  }

  getHelp(): string {
    return `` // FIXME
  }

  parse(args: string[]): Result<IArgs<C, O>> {

  }
}

export const ArgsParser = ArgsParserBuilder.create()
