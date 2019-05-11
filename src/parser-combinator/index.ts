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

const success = <T>(result: T, nextPos: number): ISuccess<T> => ({ success: true, result, nextPos })
const failure = (msg: string, pos: number, label: string = 'unknown'): IFailure => ({ success: false, msg, pos, label })

export interface INode<T> {
  type: string
  value: T
}
export const inode = <T>(type: string, value: T): INode<T> => ({ type, value })

class Parser<T> {
  constructor(public apply: (src: string, pos: number) => ISuccess<T> | IFailure) { }

  parseAll(src: string) {
    const result = this.apply(src, 0)
    if (!result.success) {
      return result
    }

    if (result.nextPos < src.length) {
      return failure('Failed to parse whole string', 0, 'parseAll')
    }

    return result
  }

  // COMBINATORS

  // sequence of matchers
  andThen<K>(p2: Parser<K>): Parser<[T, K]> {
    return new Parser((msg, pos) => {
      const result1 = this.apply(msg, pos)
      if (!result1.success) {
        return result1
      }

      const result2 = p2.apply(msg, result1.nextPos)
      if (!result2.success) {
        return result2
      }

      return success([result1.result, result2.result], result2.nextPos)
    })
  }

  // one | two
  orElse<K>(p2: Parser<K>): Parser<T | K> {
    return new Parser<T | K>((msg, pos) => {
      {
        const result = this.apply(msg, pos)
        if (result.success) {
          return result
        }
      }

      {
        const result = p2.apply(msg, pos)
        if (result.success) {
          return result
        }
      }

      return failure('No matches', pos)
    })
  }

  // transform result
  map<K>(fn: (value: T) => K): Parser<K> {
    return new Parser((msg, pos) => {
      const result = this.apply(msg, pos)
      if (!result.success) {
        return result
      }

      return success(fn(result.result), result.nextPos)
    })
  }

  asNode(type: string): Parser<INode<T>> {
    return this.map(value => ({ type, value })).withLabel(type)
  }

  // set failure label
  withLabel(label: string): Parser<T> {
    return new Parser((msg, pos) => {
      const result = this.apply(msg, pos)
      if (result.success) {
        return result
      }

      return failure(result.msg, result.pos, `${label}>${result.label}`)
    })
  }

  // a+
  oneOrMore(): Parser<T[]> {
    return new Parser((msg, pos) => {
      const values: T[] = []

      let currentPos = pos
      let latestResult = this.apply(msg, currentPos)
      if (!latestResult.success) {
        return latestResult
      }

      do {
        values.push(latestResult.result)

        currentPos = latestResult.nextPos
        latestResult = this.apply(msg, currentPos)
      } while (latestResult.success)

      return success(values, currentPos)
    })
  }

  // a*
  zeroOrMore(): Parser<T[]> {
    return new Parser((msg, pos) => {
      const values: T[] = []

      let currentPos = pos
      let latestResult = this.apply(msg, currentPos)

      while (latestResult.success) {
        values.push(latestResult.result)

        currentPos = latestResult.nextPos
        latestResult = this.apply(msg, currentPos)
      }

      return success(values, currentPos)
    })
  }

  // a?
  optional(): Parser<T | undefined> {
    return new Parser<T | undefined>((msg, pos) => {
      const result = this.apply(msg, pos)
      if (result.success) {
        return success(result.result, result.nextPos)
      }

      return success(undefined, pos)
    })
  }

  between(left: Parser<string>, right: Parser<string>): Parser<T> {
    return left.andThen(this).andThen(right).map(value => value[0][1])
  }
}

export const stringifyFailure = (f: IFailure) => `Failed to parse ${f.label} at pos ${f.pos}: ${f.msg}`

export const everythingUntil = <T>(parser: Parser<T>): Parser<string> => new Parser((msg, pos) => {
  let currentPos = pos
  let result
  do {
    result = parser.apply(msg, currentPos)
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

  return success(msg.substring(pos, currentPos), nextPos)
})

// MATCHERS

// matches end of the string
export const eof: Parser<string> = new Parser((msg, pos) => {
  if (pos === msg.length) {
    return success('', pos)
  }

  return failure('Not EOF', pos)
})

// matches beginning of the string
export const bof: Parser<string> = new Parser((_msg, pos) => {
  if (pos === 0) {
    return success('', pos)
  }

  return failure('Not BOF', pos)
})

type Predicate = (msg: string, pos: number) => [boolean, string]
export const satisfy = (predicate: Predicate, label = 'unknown'): Parser<string> =>
  new Parser((msg, pos) => {
    if (pos === msg.length) {
      return failure('No more input', pos, `satisfy>${label}`)
    }

    const result = predicate(msg, pos)
    if (!result[0]) {
      return failure(result[1], pos)
    }

    const nextPos = pos + result[1].length

    return success(result[1], nextPos)
  })

export const expect = (s: string) => satisfy((msg, pos) => {
  const match = msg.substring(pos, pos + s.length)
  if (s.length > match.length) {
    return [false, 'Not enough input']
  }

  if (s !== match) {
    return [false, 'No match']
  }

  return [true, s]
}, `expect(${s})`)

export const regex = (re: RegExp) => satisfy((msg, pos) => {
  if (re.toString()[1] !== '^') {
    throw new Error(`regex parsers must contain '^' start assertion.`)
  }

  const result = re.exec(msg.substring(pos))
  if (!result) {
    return [false, 'No match']
  }

  return [true, result[0]]
}, `regex(${re})`)

export const anyCharExcept = (chars: string) => satisfy((msg, pos) => {
  if (chars.includes(msg[pos])) {
    return [false, 'Matched forbidden char']
  }

  return [true, msg[pos]]
}, `anyCharExcept(${chars})`)
