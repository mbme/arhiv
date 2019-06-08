import * as React from 'react'
import {
  style,
} from 'typestyle'
import {
  ILocation,
  IParams,
  Link,
} from '~/web-router'
import {
  theme,
  Overlay,
  Icon,
  fromMd,
  fromSm,
} from '~/web-components'
import { IsodbWebClient } from '~/isodb-web-client'
import { NotFound } from '../parts'

const maxWidth = '35rem'
const containerStyle = style(
  {
    display: 'grid',
    gridTemplateAreas: '"content"',
  },
  fromMd({
    gridTemplateColumns: `minmax(180px, 30%) ${maxWidth} auto`,
    gridTemplateAreas: '"sidemenu content whitespace"',
  }),
)

const navbarContainerStyle = style(
  {
    gridArea: 'sidemenu',
    position: 'sticky',
    top: '0',

    display: 'none',
  },
  fromMd({
    display: 'block',
  }),
)

const navbarStyle = style({
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

const navlinkStyle = (isSelected?: boolean) => style(
  {
    display: 'inline-block',
    margin: `${theme.spacing.medium} 0`,
  },
  isSelected && {
    color: theme.color.primary,
  },
)

const logoutLinkStyle = style({
  position: 'absolute',
  bottom: theme.spacing.small,
  cursor: 'pointer',
})

const menuIconStyle = style(
  {
    position: 'fixed',
    top: theme.spacing.fine,
    left: theme.spacing.small,
  },
  fromMd({
    display: 'none',
  }),
)

const viewStyle = style(
  {
    gridArea: 'content',
    justifySelf: 'center',
    padding: `0 ${theme.spacing.small}`,
    width: '100%',
    maxWidth,

    display: 'flex',
    flexDirection: 'column',
  },
  fromSm({
    padding: `0 ${theme.spacing.medium}`,
  }),
  fromMd({
    padding: `0 ${theme.spacing.large}`,
  }),
)

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
        className={navlinkStyle(app === currentApp)}
      >
        {app.name}
      </Link>
    ))

    return (
      <nav className={navbarStyle} onClick={this.hideNav}>
        {links}

        <div
          className={logoutLinkStyle}
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
      <div className={containerStyle}>
        {!isNavVisible && (
          <Icon
            type="menu"
            className={menuIconStyle}
            onClick={this.toggleNav}
          />
        )}
        <div className={navbarContainerStyle}>
          {navbar}
        </div>

        {isNavVisible && (
          <Overlay>
            {navbar}
          </Overlay>
        )}

        <div className={viewStyle}>
          {view || NotFound}
        </div>
      </div>
    )
  }
}
