import * as React from 'react'
import {
  ILocation,
  IParams,
  Link,
} from '~/web-router'
import {
  theme,
  Overlay,
  Icon,
} from '~/web-platform'
import { stylish } from '~/stylish'
import { IsodbWebClient } from '~/isodb-web-client'
import { NotFound } from '../parts'

const maxWidth = '35rem'
const $container = stylish({
  display: 'grid',
  gridTemplateAreas: '"content"',
  [theme.media.fromMd]: {
    gridTemplateColumns: `minmax(180px, 30%) ${maxWidth} auto`,
    gridTemplateAreas: '"sidemenu content whitespace"',
  },
})

const $navbarContainer = stylish({
  gridArea: 'sidemenu',
  position: 'sticky',
  top: '0',

  display: 'none',

  [theme.media.fromMd]: {
    display: 'block',
  },
})

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

const $menuIcon = stylish({
  position: 'fixed',
  top: theme.spacing.fine,
  left: theme.spacing.small,
  [theme.media.fromMd]: {
    display: 'none',
  },
})

const $view = stylish({
  gridArea: 'content',
  justifySelf: 'center',
  padding: `0 ${theme.spacing.small}`,
  width: '100%',
  maxWidth,

  display: 'flex',
  flexDirection: 'column',

  [theme.media.fromSm]: {
    padding: `0 ${theme.spacing.medium}`,
  },

  [theme.media.fromMd]: {
    padding: `0 ${theme.spacing.large}`,
  },
})

export interface IApp {
  name: string,
  rootRoute: string,
  routes: { [route: string]: (params: IParams) => React.ReactNode },
}

interface IProps {
  apps: IApp[]
  location: ILocation
  client: IsodbWebClient
}

interface IState {
  currentApp?: IApp
  isNavVisible: boolean
}

export class View extends React.PureComponent<IProps, IState> {
  state: IState = {
    currentApp: undefined,
    isNavVisible: false,
  }

  static getDerivedStateFromProps({ apps, location }: IProps) {
    for (const app of apps) {
      for (const path of Object.keys(app.routes)) {
        if (path === location.path) {
          return { currentApp: app }
        }
      }
    }

    return null
  }

  logout = () => {
    this.props.client.deauthorize()
  }

  toggleNav = () => {
    this.setState(state => ({ isNavVisible: !state.isNavVisible }))
  }

  hideNav = () => {
    this.setState({ isNavVisible: false })
  }

  renderNavbar() {
    const {
      apps,
    } = this.props

    const {
      currentApp,
    } = this.state

    const links = apps.map(app => (
      <Link
        key={app.name}
        to={{ path: app.rootRoute }}
        className={$navlink.with({ isSelected: app === currentApp }).className}
      >
        {app.name}
      </Link>
    ))

    return (
      <nav className={$navbar.className} onClick={this.hideNav}>
        {links}

        <div
          className={$logoutLink.className}
          onClick={this.logout}
        >
          Logout
        </div>
      </nav>
    )
  }

  render() {
    const {
      location,
    } = this.props

    const {
      currentApp,
      isNavVisible,
    } = this.state

    const view = currentApp ? currentApp.routes[location.path](location.params) : null
    const navbar = this.renderNavbar()

    return (
      <div className={$container.className}>
        {!isNavVisible && (
          <Icon
            type="menu"
            $style={$menuIcon}
            onClick={this.toggleNav}
          />
        )}
        <div className={$navbarContainer.className}>
          {navbar}
        </div>

        {isNavVisible && (
          <Overlay>
            {navbar}
          </Overlay>
        )}

        <div className={$view.className}>
          {view || NotFound}
        </div>
      </div>
    )
  }
}
