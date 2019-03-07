import * as React from 'react'
import {
  Route,
  RouterContext,
} from './Router'

type Props = JSX.IntrinsicElements['a'] & {
  to: Route
  newTab?: boolean
}

export function Link({ to, newTab, ...otherProps }: Props) {
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
