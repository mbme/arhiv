import * as React from 'react'
import {
  Route,
  RouterContext,
} from './Router'

interface IProps {
  to: Route
}

export function Redirect({ to }: IProps) {
  const routerContext = React.useContext(RouterContext)
  React.useEffect(() => {
    routerContext.push(to)
  }, [])

  return null
}
