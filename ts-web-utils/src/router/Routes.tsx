import * as React from 'react'
import { PathMatcher, Obj } from '@v/utils'
import { IQueryParam } from './types'
import { useLocation } from './hooks'

export type Route<T extends Obj> = [PathMatcher<T>, (props: T, queryParams: IQueryParam[]) => React.ReactNode]

interface IProps {
  children: Route<any>[]
  onNotFound?: () => React.ReactNode
}

const renderNull = () => null

export function Routes({ children, onNotFound = renderNull }: IProps) {
  const location = useLocation()

  for (const [matcher, render] of children) {
    const match = matcher.match(location.path)

    if (match) {
      return (
        <>
          {render(match, location.params)}
        </>
      )
    }
  }

  return (
    <>
      {onNotFound()}
    </>
  )
}
