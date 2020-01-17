import {
  Parser,
  Success,
  Failure,
  isFailure,
} from './parser'

// matches end of the string
export const eof: Parser<string> = new Parser((msg, pos) => {
  if (pos === msg.length) {
    return new Success('', pos)
  }

  return new Failure('Not EOF', pos, 'eof')
})

// matches beginning of the string
export const bof: Parser<string> = new Parser((_msg, pos) => {
  if (pos === 0) {
    return new Success('', pos)
  }

  return new Failure('Not BOF', pos, 'bof')
})

type Predicate = (msg: string, pos: number) => [boolean, string]
export const satisfy = (predicate: Predicate, label = 'unknown'): Parser<string> => (
  new Parser((msg, pos) => {
    if (pos === msg.length) {
      return new Failure('No more input', pos, `satisfy>${label}`)
    }

    const result = predicate(msg, pos)
    if (!result[0]) {
      return new Failure(result[1], pos, `satisfy>${label}`)
    }

    const nextPos = pos + result[1].length

    return new Success(result[1], nextPos)
  })
)

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
    throw new Error("regex parsers must start with '^' assertion.")
  }

  const result = re.exec(msg.substring(pos))
  if (!result) {
    return [false, 'No match']
  }

  return [true, result[0]]
}, `regex(${re})`)

export const anyChar = satisfy((msg, pos) => {
  if (pos >= msg.length) {
    return [false, 'no match: eof']
  }

  return [true, msg[pos]]
}, 'anyChar')

export const anyCharExcept = (chars: string) => satisfy((msg, pos) => {
  if (chars.includes(msg[pos])) {
    return [false, 'Matched forbidden char']
  }

  return [true, msg[pos]]
}, `anyCharExcept(${chars})`)

export const everythingUntil = <T>(parser: Parser<T>): Parser<string> => new Parser((msg, pos) => {
  let currentPos = pos
  let result
  do {
    result = parser.apply(msg, currentPos)
    if (isFailure(result)) {
      currentPos += 1
    }

    if (currentPos > msg.length) {
      return new Failure('no match: eof', pos, 'everythingUntil')
    }
  } while (isFailure(result))

  if (currentPos === pos) {
    return new Failure('no match', pos, 'everythingUntil')
  }

  const nextPos = currentPos + 1

  return new Success(msg.substring(pos, currentPos), nextPos)
})
