import {
  getLastEl,
  removeMut,
  Dict,
  Obj,
} from '@v/utils'

const nameRegex = /^[a-zA-Z0-9-]+$/

interface IOption<O extends string, V> {
  name: O
  description: string
  positional?: 'one' | 'array'
  mandatory: boolean
  defaultValue?: V
}

function option2string(option: IOption<string, any>): string {
  if (option.positional === 'one') {
    return `<${option.name}>`
  }

  if (option.positional === 'array') {
    return `<${option.name}...>`
  }

  if (option.defaultValue) {
    return `${option.name}=${option.defaultValue}`
  }

  return option.name
}

function getOptionHelp(option: IOption<any, any>) {
  let result = '    ' + option2string(option)

  if (option.description) {
    result += ` - ${option.description}`
  }

  if (option.mandatory) {
    result += ', mandatory'
  }

  return result
}

export class Command<C extends string, CO extends Obj> {
  constructor(
    public readonly name: C,
    public readonly description: string,
    private _options: IOption<keyof CO, any>[],
  ) {
    if (name && !nameRegex.test(name)) {
      throw new Error(`command ${name} doesn't match ${nameRegex}`)
    }

    if (name.startsWith('-')) {
      throw new Error(`command ${name} shouldn't start with -`)
    }
  }

  private _addOption<O extends string, V>(option: IOption<O, V>) {
    if (!nameRegex.test(option.name)) {
      throw new Error(`option ${option.name} doesn't match ${nameRegex}`)
    }

    if (!option.positional && !option.name.startsWith('-')) {
      throw new Error(`option ${option.name} should start with -`)
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

  option<O extends string, T extends string | undefined, M extends boolean>(
    name: O,
    description: string,
    defaultValue?: T,
    mandatory?: M,
  ) {
    return this._addOption<O, T extends string ? string : M extends true ? string : string | undefined>({
      name,
      description,
      mandatory: mandatory || false,
      defaultValue: defaultValue as any,
    })
  }

  positional<O extends string, M extends boolean>(name: O, description: string, mandatory?: M) {
    return this._addOption<O, M extends true ? string : string | undefined>({
      name,
      description,
      positional: 'one',
      mandatory: mandatory || false,
    })
  }

  positionalArray<O extends string>(name: O, description: string, mandatory = false) {
    return this._addOption<O, string[]>({
      name,
      description,
      positional: 'array',
      mandatory,
    })
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

        result[option.name] = arg === option.name ? '' : arg.substring(option.name.length + 1)
      } else { // positional
        const option = optionsToCheck.find(item => item.positional)
        if (!option) {
          throw new Error(`unexpected positional option ${arg}`)
        }

        removeMut(optionsToCheck, option)

        if (option.positional === 'one') {
          result[option.name] = arg
        } else {
          result[option.name] = args.slice(i)
          break
        }
      }
    }

    // save options which has default values
    for (const option of optionsToCheck) {
      if (option.defaultValue !== undefined) {
        result[option.name] = option.defaultValue
      }
    }

    // make sure there are no mandatory options missing
    const mandatoryOptions = optionsToCheck
      .filter(option => option.mandatory)
      .map(option => option.name)

    if (mandatoryOptions.length) {
      throw new Error(`Mandatory options are missing: ${mandatoryOptions.join(', ')}`)
    }

    return result as CO
  }

  getHelp(appName: string) {
    const options = this._options
      .filter(option => !option.positional)
      .map(getOptionHelp)

    const positionalOptions = this._options
      .filter(option => option.positional)
      .map(getOptionHelp)

    const positionalStr = this._options
      .filter(option => option.positional)
      .map(option2string)
      .join(' ')

    const help = []
    if (this.description) {
      help.push(`  # ${this.description}`)
    }

    help.push([
      ' ',
      appName,
      this.name,
      options.length ? '[options]' : '',
      positionalStr,
    ].filter(item => item).join(' '))

    help.push(...positionalOptions)
    help.push(...options)

    return help.join('\n')
  }
}

export function command(name: string, description: string) {
  return new Command(name, description, [])
}
