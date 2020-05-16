import * as React from 'react'
import { PathMatcher } from '@v/utils'
import { RouterContext } from './context'
import { useObservable } from '../hooks'
import { IQueryParam } from './types'

interface IProps<T extends object> {
  pm: PathMatcher<T>,
  children(props: T, queryParams: IQueryParam[]): React.ReactNode
}

export function Route<T extends object>({ pm, children }: IProps<T>) {
  const router = RouterContext.use()
  const [location] = useObservable(() => router.location$.value$)

  if (!location) {
    return null
  }

  const match = pm.match(location.path)

  if (!match) {
    return null
  }

  return (
    <>
      {children(match, location.params)}
    </>
  )
}
