interface ISuccess<T> {
  success: true

  result: T,
  nextPos: number
}

interface IFailure {
  success: false

  label: string
  msg: string
  pos: number
}

type Parser<T> = (src: string, pos: number) => ISuccess<T> | IFailure

const success = <T>(result: T, nextPos: number): ISuccess<T> => ({ success: true, result, nextPos })
const failure = (msg: string, pos: number, label: string = 'unknown'): IFailure => ({ success: false, msg, pos, label })

export const stringifyFailure = (f: IFailure) => `Failed to parse ${f.label} at pos ${f.pos}: ${f.msg}`

// COMBINATORS

// sequence of matchers
export const andThen = <T>(...parsers: Array<Parser<T>>): Parser<T[]> => (msg, pos) => {
  const values: T[] = []

  let currentPos = pos
  for (const parser of parsers) {
    const result = parser(msg, currentPos)
    if (!result.success) {
      return result
    }

    currentPos = result.nextPos
    values.push(result.result)
  }

  return success(values, currentPos)
}

// one | two | three
export const orElse = <T>(...parsers: Array<Parser<T>>): Parser<T> => (msg, pos) => {
  for (const parser of parsers) {
    const result = parser(msg, pos)
    if (result.success) {
      return result
    }
  }

  return failure('No matches', pos)
}

// transform result
export const mapP = <T, V>(fn: (p: T) => V, parser: Parser<T>): Parser<V> => (msg, pos) => {
  const result = parser(msg, pos)
  if (!result.success) {
    return result
  }

  return success(fn(result.result), result.nextPos)
}

// a+
export const oneOrMore = <T>(parser: Parser<T>): Parser<T[]> => (msg, pos) => {
  const values: T[] = []

  let currentPos = pos
  let latestResult = parser(msg, currentPos)
  if (!latestResult.success) {
    return latestResult
  }

  do {
    values.push(latestResult.result)

    currentPos = latestResult.nextPos
    latestResult = parser(msg, currentPos)
  } while (latestResult.success)

  return success(values, currentPos)
}

// a*
export const zeroOrMore = <T>(parser: Parser<T>): Parser<T[]> => (msg, pos) => {
  const values: T[] = []

  let currentPos = pos
  let latestResult = parser(msg, currentPos)

  while (latestResult.success) {
    values.push(latestResult.result)

    currentPos = latestResult.nextPos
    latestResult = parser(msg, currentPos)
  }

  return success(values, currentPos)
}

// a?
export const optional = <T>(parser: Parser<T>): Parser<T[]> => (msg, pos) => {
  const result = parser(msg, pos)
  if (result.success) {
    return success([result.result], result.nextPos)
  }

  return success([], pos)
}

export const everythingUntil = <T>(parser: Parser<T>): Parser<string> => (msg, pos) => {
  let currentPos = pos
  let result
  do {
    result = parser(msg, currentPos)
    if (!result.success) {
      currentPos += 1
    }

    if (currentPos > msg.length) {
      return failure('no match: eof', pos, 'everythingUntil')
    }
  } while (!result.success)

  if (currentPos === pos) {
    return failure('no match', pos, 'everythingUntil')
  }

  return success(msg.substring(pos, currentPos), currentPos + 1)
}

// set failure label
export const setLabel = <T>(parser: Parser<T>, label: string): Parser<T> => (msg, pos) => {
  const result = parser(msg, pos)
  if (result.success) {
    return result
  }

  return failure(result.msg, result.pos, label)
}

// MATCHERS

export const eof: Parser<string> = (src, pos) => {
  if (pos === src.length) {
    return success('', pos)
  }

  return failure('Not EOF', pos)
}

export const satisfy = (predicate: (current: string) => [boolean, string]): Parser<string> => (src, pos) => {
  if (pos === src.length) {
    return failure('No more input', pos)
  }

  const result = predicate(src.substring(pos))
  if (!result[0]) {
    return failure(result[1], pos)
  }

  return success(result[1], pos + result[1].length)
}

export const expect = (s: string) => satisfy((current) => {
  if (s.length > current.length) {
    return [false, 'Not enough input']
  }

  if (!current.startsWith(s)) {
    return [false, 'No match']
  }

  return [true, s]
})

export const regex = (re: RegExp) => satisfy((current) => {
  if (re.toString()[1] !== '^') {
    throw new Error(`regex parsers must contain '^' start assertion.`)
  }

  const result = re.exec(current)
  if (!result) {
    return [false, 'No match']
  }

  return [true, result[0]]
})
