import { Dict } from './types'

interface IStaticSegment {
  dynamic: false
  value: string
}

interface INamedSegment {
  dynamic: true
  name: string
}

type Segment = IStaticSegment | INamedSegment

export class PathMatcher<C extends object> {
  private constructor(
    private _segments: Segment[] = [],
  ) { }

  static create(): PathMatcher<{}> {
    return new PathMatcher<{}>([])
  }

  param<T extends string>(name: T): PathMatcher<C & { [key in T]: string }> {
    const segment: INamedSegment = {
      dynamic: true,
      name,
    }

    return new PathMatcher<C & { [key in T]: string }>([...this._segments, segment])
  }

  string(value: string) {
    const segment: IStaticSegment = {
      dynamic: false,
      value,
    }

    return new PathMatcher<C>([...this._segments, segment])
  }

  match(path: string[]): C | undefined {
    if (path.length !== this._segments.length) {
      return undefined
    }

    const result: Dict = {}

    for (let i = 0; i < this._segments.length; i += 1) {
      const segment = this._segments[i]
      const pathSegment = path[i]

      if (segment.dynamic) {
        result[segment.name] = path[i]
      } else if (segment.value !== pathSegment) {
        return undefined
      }
    }

    return result as C
  }
}
