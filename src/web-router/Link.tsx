import * as React from 'react'
import {
  Route,
  RouterContext,
} from './Router'

interface IProps {
  to: Route
  newTab?: boolean
  className?: string
  children: string
}

export function Link({ to, newTab, className, children }: IProps) {
  const routerContext = React.useContext(RouterContext)

  const url = routerContext.getUrl(to)

  const onClick = (e: React.MouseEvent<HTMLAnchorElement>) => {
    routerContext.push(to)
    e.preventDefault()
  }

  return (
    <a
      href={url}
      onClick={onClick}
      target={newTab ? '_blank' : undefined}
      className={className}
    >
      {children}
    </a>
  )
}
