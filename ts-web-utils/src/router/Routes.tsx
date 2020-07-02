import * as React from 'react'
import { PathMatcher, Obj } from '@v/utils'
import { RouterContext } from './context'
import { useObservable } from '../hooks'
import { IQueryParam } from './types'

export type Route<T extends Obj> = [PathMatcher<T>, (props: T, queryParams: IQueryParam[]) => React.ReactNode]

interface IProps {
  children: Route<any>[]
  onNotFound?: () => React.ReactNode
}

const renderNull = () => null

export function Routes({ children, onNotFound = renderNull }: IProps) {
  const router = RouterContext.use()
  const [location] = useObservable(() => router.location$.value$)

  if (!location) {
    return null
  }

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
