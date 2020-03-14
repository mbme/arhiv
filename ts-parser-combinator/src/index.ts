import {
  Parser,
  isSuccess,
  isFailure,
  Result,
} from './parser'

export {
  isSuccess,
  isFailure,
  Result,
}

export * from './matchers'

export type ParserResult<P> = P extends Parser<infer T> ? T : never
