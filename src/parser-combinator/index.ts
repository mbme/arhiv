// tslint:disable:max-line-length

interface IParseResult {
  kind: string
}

interface ISuccess<T extends IParseResult> {
  success: true
  result: T,
  nextPos: number
}
const success = <T extends IParseResult>(result: T, nextPos: number): ISuccess<T> => ({ success: true, result, nextPos })

interface IFailure {
  success: false
  msg: string
  pos: number
}
const failure = (msg: string, pos: number): IFailure => ({ success: false, msg, pos })

interface IParser<T extends IParseResult> {
  (src: string, pos: number): ISuccess<T> | IFailure

  parserName?: string
}

// COMBINATORS
interface ICombinedResult<T extends IParseResult> extends IParseResult {
  kind: 'combined'
  values: T[]
}

export const andThen = <T extends IParseResult>(...parsers: Array<IParser<T>>): IParser<ICombinedResult<T>> => (msg, pos) => {
  const combined: ICombinedResult<T> = {
    kind: 'combined',
    values: [],
  }

  let currentPos = pos
  for (const parser of parsers) {
    const result = parser(msg, currentPos)
    if (!result.success) {
      return result
    }

    currentPos = result.nextPos
    combined.values.push(result.result)
  }

  return success(combined, currentPos)
}

export const orElse = <T extends IParseResult>(...parsers: Array<IParser<T>>): IParser<T> => (msg, pos) => {
  for (const parser of parsers) {
    const result = parser(msg, pos)
    if (result.success) {
      return result
    }
  }

  return failure('No matches', pos)
}

export const mapP = <T extends IParseResult, V extends IParseResult>(fn: (p: T) => V, parser: IParser<T>): IParser<V> => (msg, pos) => {
  const result = parser(msg, pos)
  if (!result.success) {
    return result
  }

  return success(fn(result.result), result.nextPos)
}

// PARSERS

interface IParsedString extends IParseResult {
  kind: 'string'
  value: string
}
export const expectStr = (s: string): IParser<IParsedString> => (src, pos) => {
  if (pos === src.length) {
    return failure('No more input', pos)
  }

  if (pos + s.length > src.length) {
    return failure('Not enough input', pos)
  }

  const match = src.substring(pos, pos + s.length)
  if (s === match) {
    return success({ kind: 'string', value: s } as IParsedString, pos + s.length)
  }

  return failure(`No match: expected "${s}" but got "${match}"`, pos)
}
