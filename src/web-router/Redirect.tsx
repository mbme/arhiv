import * as React from 'react'
import {
  Location,
  RouterContext,
} from './Router'

interface IProps {
  to: Location
}

export function Redirect({ to }: IProps) {
  const routerContext = React.useContext(RouterContext)
  React.useEffect(() => {
    routerContext.push(to)
  }, [])

  return null
}
