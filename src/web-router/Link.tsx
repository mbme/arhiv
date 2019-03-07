import * as React from 'react'
import {
  Route,
  RouterContext,
} from './Router'

interface IProps {
  to: Route
  children: string
  newTab?: boolean
}

export function Link({ to, newTab, ...otherProps }: IProps) {
  const routerContext = React.useContext(RouterContext)

  const url = React.useMemo(() => routerContext.getUrl(to), [to])

  return (
    <a
      href={url}
      target={newTab ? '_blank' : undefined}
      {...otherProps}
    />
  )
}
