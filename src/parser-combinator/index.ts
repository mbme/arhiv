import {
  Parser,
} from './parser'

export * from './matchers'

export type ParserResult<P> = P extends Parser<infer T> ? T : never
