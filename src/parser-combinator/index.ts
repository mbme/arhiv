interface ISuccess<T> {
  success: true

  result: T,
  nextPos: number
  eof: boolean
}

interface IFailure {
  success: false

  label: string
  msg: string
  pos: number
}

type Parser<T> = (src: string, pos: number) => ISuccess<T> | IFailure

const success = <T>(result: T, nextPos: number, isEof: boolean): ISuccess<T> => ({
  success: true,
  result,
  nextPos,
  eof: isEof,
})
const failure = (msg: string, pos: number, label: string = 'unknown'): IFailure => ({ success: false, msg, pos, label })

export const stringifyFailure = (f: IFailure) => `Failed to parse ${f.label} at pos ${f.pos}: ${f.msg}`

export const parse = <T>(parser: Parser<T>, s: string) => {
  const result = parser(s, 0)
  if (!result.success) {
    return result
  }

  if (!result.eof) {
    return failure('Failed to parse whole string', 0, 'parseAll')
  }

  return result
}

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

  return success(values, currentPos, currentPos === msg.length)
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

  return success(fn(result.result), result.nextPos, result.nextPos === msg.length)
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

  return success(values, currentPos, currentPos === msg.length)
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

  return success(values, currentPos, currentPos === msg.length)
}

// a?
export const optional = <T>(parser: Parser<T>): Parser<T[]> => (msg, pos) => {
  const result = parser(msg, pos)
  if (result.success) {
    return success([result.result], result.nextPos, result.nextPos === msg.length)
  }

  return success([], pos, pos === msg.length)
}

export const between = (start: Parser<string>, stop: Parser<string>): Parser<string> => (msg, pos) => {
  const startResult = start(msg, pos)
  if (!startResult.success) {
    return startResult
  }

  let currentPos = startResult.nextPos
  let result
  do {
    result = stop(msg, currentPos)
    if (!result.success) {
      currentPos += 1
    }

    if (currentPos > msg.length) {
      return failure('no match: eof', pos, 'between')
    }
  } while (!result.success)

  if (currentPos === pos) {
    return failure('no match', pos, 'between')
  }

  const nextPos = (result as ISuccess<string>).nextPos

  const str = msg.substring(pos + startResult.result.length, currentPos)

  return success(str, nextPos, nextPos === msg.length)
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

  const nextPos = currentPos + 1

  return success(msg.substring(pos, currentPos), nextPos, nextPos === msg.length)
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

export const eof: Parser<string> = (msg, pos) => {
  if (pos === msg.length) {
    return success('', pos, true)
  }

  return failure('Not EOF', pos)
}

export const satisfy = (predicate: (current: string) => [boolean, string]): Parser<string> => (msg, pos) => {
  if (pos === msg.length) {
    return failure('No more input', pos)
  }

  const result = predicate(msg.substring(pos))
  if (!result[0]) {
    return failure(result[1], pos)
  }

  const nextPos = pos + result[1].length

  return success(result[1], nextPos, nextPos === msg.length)
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
