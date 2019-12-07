import { Dict } from './types'

interface IStaticSegment {
  type: 'static'
  value: string
}

interface INamedSegment {
  type: 'named'
  name: string
}

interface IEverythingSegment {
  type: 'everything'
}

type Segment = IStaticSegment | INamedSegment | IEverythingSegment

function splitPath(path: string): string[] {
  if (path.startsWith('/')) {
    return splitPath(path.substring(1))
  }

  return path.split('/')
}

export class PathMatcher<C extends object> {
  private constructor(
    private _segments: Segment[] = [],
  ) { }

  static create() {
    return new PathMatcher<{}>([])
  }

  param<T extends string>(name: T) {
    const segment: INamedSegment = {
      type: 'named',
      name,
    }

    return new PathMatcher<C & { [key in T]: string }>([...this._segments, segment])
  }

  string(value: string) {
    const segment: IStaticSegment = {
      type: 'static',
      value,
    }

    return new PathMatcher<C>([...this._segments, segment])
  }

  everything() {
    const segment: IEverythingSegment = {
      type: 'everything',
    }

    return new PathMatcher<C & { everything: string[] }>([...this._segments, segment])
  }

  match(pathRaw: string): C | undefined {
    const path = splitPath(pathRaw)

    if (path.length < this._segments.length) {
      return undefined
    }

    const result: Dict<any> = {}

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
        result.everything = path.slice(i)
        break
      }
    }

    return result as C
  }
}
