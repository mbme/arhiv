import * as React from 'react'
import { Link } from '~/web-router'
import {
  theme,
  stylish,
} from '~/web-platform'
import { IApp } from './IApp'

const $navbar = stylish({
  position: 'sticky',
  top: '0',

  height: '100vh',
  width: '100%',
  padding: `${theme.spacing.small} ${theme.spacing.large}`,

  backgroundColor: theme.color.secondary,
  color: theme.color.light,
  fontSize: theme.fontSize.medium,

  display: 'flex',
  flexDirection: 'column',
  alignItems: 'flex-end',
})

const $navlink = stylish(
  {
    display: 'inline-block',
    margin: `${theme.spacing.medium} 0`,
  },
  props => props.isSelected && {
    color: theme.color.primary,
  },
)

const $logoutLink = stylish({
  position: 'absolute',
  bottom: theme.spacing.small,
  cursor: 'pointer',
})

interface IProps {
  apps: IApp[]
  currentApp?: IApp
  onClick(): void
  onLogout(): void
}

export function NavBar({ apps, currentApp, onClick, onLogout }: IProps) {
  const links = apps.map(app => (
    <Link
      key={app.name}
      to={{ path: app.route }}
      className={$navlink.with({ isSelected: app === currentApp }).className}
    >
      {app.name}
    </Link>
  ))

  return (
    <nav className={$navbar.className} onClick={onClick}>
      {links}

      <div
        className={$logoutLink.className}
        onClick={onLogout}
      >
        Logout
      </div>
    </nav>
  )
}
