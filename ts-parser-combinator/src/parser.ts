export class Success<T> {
  constructor(
    public value: T,
    public nextPos: number,
  ) { }
}

export class Failure {
  constructor(
    public msg: string,
    public pos: number,
    public label: string,
  ) { }

  toString() {
    return `Failed to parse ${this.label} at pos ${this.pos}: ${this.msg}`
  }
}

// FIXME use general result
export type Result<T> = Success<T> | Failure

export function isSuccess<T>(result: Result<T>): result is Success<T> {
  return result instanceof Success
}

export function isFailure<T>(result: Result<T>): result is Failure {
  return result instanceof Failure
}

export class Parser<T> {
  constructor(public apply: (src: string, pos: number) => Result<T>) { }

  parseAll(src: string) {
    const result = this.apply(src, 0)
    if (isFailure(result)) {
      return result
    }

    if (result.nextPos < src.length) {
      return new Failure('Failed to parse whole string', 0, 'parseAll')
    }

    return result
  }

  // COMBINATORS

  // sequence of matchers
  andThen<K>(p2: Parser<K>): Parser<[T, K]> {
    return new Parser((msg, pos) => {
      const result1 = this.apply(msg, pos)
      if (isFailure(result1)) {
        return result1
      }

      const result2 = p2.apply(msg, result1.nextPos)
      if (isFailure(result2)) {
        return result2
      }

      return new Success([result1.value, result2.value], result2.nextPos)
    })
  }

  // match sequence and drop first result
  dropAndThen<K>(p2: Parser<K>): Parser<K> {
    return new Parser((msg, pos) => {
      const result1 = this.apply(msg, pos)
      if (isFailure(result1)) {
        return result1
      }

      const result2 = p2.apply(msg, result1.nextPos)
      if (isFailure(result2)) {
        return result2
      }

      return new Success(result2.value, result2.nextPos)
    })
  }

  // match sequence and drop last result
  andThenDrop<K>(p2: Parser<K>): Parser<T> {
    return new Parser((msg, pos) => {
      const result1 = this.apply(msg, pos)
      if (isFailure(result1)) {
        return result1
      }

      const result2 = p2.apply(msg, result1.nextPos)
      if (isFailure(result2)) {
        return result2
      }

      return new Success(result1.value, result2.nextPos)
    })
  }

  // one | two
  orElse<K>(p2: Parser<K>): Parser<T | K> {
    return new Parser<T | K>((msg, pos) => {
      {
        const result = this.apply(msg, pos)
        if (isSuccess(result)) {
          return result
        }
      }

      {
        const result = p2.apply(msg, pos)
        if (isSuccess(result)) {
          return result
        }
      }

      return new Failure('No matches', pos, 'orElse')
    })
  }

  // transform result, optionally set failure label
  map<K>(fn: (value: T) => K, label?: string): Parser<K> {
    return new Parser((msg, pos) => {
      const result = this.apply(msg, pos)
      if (isSuccess(result)) {
        return new Success(fn(result.value), result.nextPos)
      }

      if (label) {
        return new Failure(result.msg, result.pos, `${label}>${result.label}`)
      }

      return result
    })
  }

  // set failure label
  withLabel(label: string): Parser<T> {
    return this.map(value => value, label)
  }

  // a+
  oneOrMore(): Parser<T[]> {
    return new Parser((msg, pos) => {
      const values: T[] = []

      let currentPos = pos
      let latestResult = this.apply(msg, currentPos)
      if (isFailure(latestResult)) {
        return latestResult
      }

      do {
        values.push(latestResult.value)

        currentPos = latestResult.nextPos
        latestResult = this.apply(msg, currentPos)
      } while (isSuccess(latestResult))

      return new Success(values, currentPos)
    })
  }

  // a*
  zeroOrMore(): Parser<T[]> {
    return new Parser((msg, pos) => {
      const values: T[] = []

      let currentPos = pos
      let latestResult = this.apply(msg, currentPos)

      while (isSuccess(latestResult)) {
        values.push(latestResult.value)

        currentPos = latestResult.nextPos
        latestResult = this.apply(msg, currentPos)
      }

      return new Success(values, currentPos)
    })
  }

  // a?
  optional(): Parser<T | undefined> {
    return new Parser<T | undefined>((msg, pos) => {
      const result = this.apply(msg, pos)
      if (isSuccess(result)) {
        return new Success(result.value, result.nextPos)
      }

      return new Success(undefined, pos)
    })
  }

  between(left: Parser<string>, right: Parser<string>): Parser<T> {
    return left.dropAndThen(this).andThenDrop(right)
  }

  inside(p: Parser<string>) {
    return this.between(p, p)
  }
}
