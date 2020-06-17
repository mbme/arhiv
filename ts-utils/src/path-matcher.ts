import { Obj } from './types'
import {
  trimPrefix,
  trimSuffix,
} from './string'
import { getLastEl } from './array'

interface IStaticSegment {
  type: 'static'
  value: string
}

interface INamedSegment<N extends string> {
  type: 'named'
  name: N
}

interface IEverythingSegment {
  type: 'everything'
}

type Segment<N extends string> = IStaticSegment | INamedSegment<N> | IEverythingSegment

export class PathMatcher<C extends Obj> {
  constructor(
    private _segments: Segment<keyof C>[] = [],
  ) { }

  match(pathRaw: string): C | undefined {
    const path = trimPrefix(pathRaw, '/').split('/').filter(Boolean)

    if (path.length !== this._segments.length && getLastEl(this._segments)?.type !== 'everything') {
      return undefined
    }

    const result: Obj = {}
    for (let i = 0; i < this._segments.length; i += 1) {
      const segment = this._segments[i]
      const pathSegment = path[i]

      if (segment.type === 'named') {
        result[segment.name] = pathSegment

        continue
      }

      if (segment.type === 'static') {
        if (segment.value !== pathSegment) {
          return undefined
        }

        continue
      }

      if (segment.type === 'everything') {
        result['*'] = path.slice(i).join('/')
        break
      }
    }

    return result as C
  }
}

function staticSegment(value: string): IStaticSegment {
  return {
    type: 'static',
    value,
  }
}

export function pathMatcher<N extends string>(
  rawLiterals: TemplateStringsArray,
  ...namedSegments: N[]
) {
  const literals = [...rawLiterals]
  if (!literals[literals.length - 1]) {
    literals.pop()
  }

  if (namedSegments.length > literals.length) {
    throw new Error('all named segments must be separated by /')
  }

  const segments: Segment<N>[] = []
  for (let i = 0; i < literals.length; i += 1) {
    const literal = literals[i]

    const isTrailingSegment = i === namedSegments.length

    if (!literal.startsWith('/')) {
      throw new Error('path segment must start with /')
    }

    // only trailing literal (which goes after last namedSegment) might not end with /
    if (!literal.endsWith('/') && !isTrailingSegment) {
      throw new Error("path segment must end with / if it's not a trailing segment")
    }

    if (literal !== '/') {
      const literalSegments = trimPrefix(trimSuffix(literal, '/'), '/').split('/')
      segments.push(...literalSegments.map(staticSegment))
    }

    const namedSegment = namedSegments[i]
    if (namedSegment === undefined) {
      continue
    }

    if (namedSegment === '*') {
      // make sure "everything" is a last segment
      const isLastSegment = (i === namedSegments.length - 1) && (i === literals.length - 1)
      if (!isLastSegment) {
        throw new Error('everything segment must be a last segment')
      }

      segments.push({
        type: 'everything',
      })
    } else {
      segments.push({
        type: 'named',
        name: namedSegment,
      })
    }
  }

  return new PathMatcher<{ [key in N]: string }>(segments)
}
