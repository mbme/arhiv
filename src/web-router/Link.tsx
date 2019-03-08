import * as React from 'react'
import {
  Route,
  RouterContext,
} from './Router'

interface IProps {
  to: Route
  newTab?: boolean
  children: string
}

export function Link({ to, newTab, children }: IProps) {
  const routerContext = React.useContext(RouterContext)

  const url = React.useMemo(() => routerContext.getUrl(to), [to])

  return (
    <a
      href={url}
      target={newTab ? '_blank' : undefined}
    >
      {children}
    </a>
  )
}
