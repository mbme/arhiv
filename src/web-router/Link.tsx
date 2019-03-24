import * as React from 'react'
import {
  classes,
} from 'typestyle'
import {
  Location,
  RouterContext,
} from './Router'

interface IProps {
  to: Location
  newTab?: boolean
  clean?: boolean
  className?: string
  children: React.ReactNode
}

export function Link({ to, newTab, clean, className, children }: IProps) {
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
      className={classes(className, clean && 'is-clean')}
    >
      {children}
    </a>
  )
}
