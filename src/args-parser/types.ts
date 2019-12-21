export interface IOption<O extends string> {
  name: O
  description: string
  positional?: 'one' | 'array'
}

export interface ICommand<C extends string, CO extends string> {
  name: C
  description: string
  options: Array<IOption<CO>>
}

export type ArgsOptions<O extends string> = {
  [key in O]: string
}

export interface IArgs<C extends string, CO extends string> {
  command: C
  options: ArgsOptions<CO>
}

export class NeedHelpError extends Error { }
