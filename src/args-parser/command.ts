import {
  Dict,
  removeMut,
  getLastEl,
} from '~/utils'
import { IOption } from './types'

const optionNameRegex = /$[a-z0-9-]+^/g

export class Command<C extends string, CO extends object> {
  constructor(
    public readonly name: C,
    public readonly description: string,
    private _options: Array<IOption<keyof CO>>,
  ) {
    if (name.startsWith('-')) {
      throw new Error('command must not start with -')
    }
  }

  private _addOption<O extends string, V>(option: IOption<O>) {
    if (!optionNameRegex.test(option.name)) {
      throw new Error(`option ${option.name} doesn't match ${optionNameRegex}`)
    }

    if (option.positional && option.name.startsWith('-')) {
      throw new Error(`positional option ${option.name} shouldn't start with -`)
    }

    if (getLastEl(this._options)?.positional === 'array') {
      throw new Error(`can't add option ${option.name} after positional array option`)
    }

    return new Command<C, CO & { [key in O]: V }>(
      this.name,
      this.description,
      [...this._options, option],
    )
  }

  option<O extends string>(name: O, description: string) {
    return this._addOption<O, string>({ name, description })
  }

  positional<O extends string>(name: O, description: string) {
    return this._addOption<O, string>({ name, description, positional: 'one' })
  }

  positionalArray<O extends string>(name: O, description: string) {
    return this._addOption<O, string[]>({ name, description, positional: 'array' })
  }

  parseOptions(args: string[]): CO {
    const result: Dict<any> = {}

    const optionsToCheck = [...this._options]

    for (let i = 0; i < args.length; i += 1) {
      const arg = args[i]

      if (arg.startsWith('-')) {
        const option = optionsToCheck.find(item => arg === item.name || arg.startsWith(`${item.name}=`))
        if (!option) {
          throw new Error(`unexpected option ${arg}`)
        }

        removeMut(optionsToCheck, option)

        result[arg] = arg === option.name ? '' : arg.substring(option.name.length + 1)
      } else { // positional
        const option = optionsToCheck.find(item => item.positional)
        if (!option) {
          throw new Error(`unexpected positional option ${arg}`)
        }

        removeMut(optionsToCheck, option)

        result[option.name] = option.positional === 'one' ? arg : args.slice(i)
      }
    }

    return result
  }
}
