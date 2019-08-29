import * as React from 'react'
import { useReactiveValue } from '~/utils/react'
import {
  useRouter,
  ILocation,
} from '~/web-router'
import {
  theme,
  Overlay,
  Icon,
  stylish,
} from '~/web-platform'
import { NotFound } from '../parts'
import { IApp } from './IApp'
import { NavBar } from './NavBar'

const maxWidth = '35rem'
const $container = stylish({
  display: 'grid',
  gridTemplateAreas: '"content"',

  fromMd: {
    gridTemplateColumns: `minmax(180px, 30%) ${maxWidth} auto`,
    gridTemplateAreas: '"sidemenu content whitespace"',
  },
})

const $navbarContainer = stylish({
  gridArea: 'sidemenu',
  position: 'sticky',
  top: '0',

  display: 'none',

  fromMd: {
    display: 'block',
  },
})

const $menuIcon = stylish({
  position: 'fixed',

  top: theme.spacing.fine,
  left: theme.spacing.small,

  fromMd: {
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

  fromSm: {
    padding: `0 ${theme.spacing.medium}`,
  },

  fromMd: {
    padding: `0 ${theme.spacing.large}`,
  },
})

interface IProps {
  apps: IApp[]
  onLogout(): void
}

function getCurrentApp(apps: IApp[], location: ILocation) {
  for (const app of apps) {
    for (const path of Object.keys(app.routes)) {
      if (path === location.path) {
        return app
      }
    }
  }

  return null
}

export function Chrome({ apps, onLogout }: IProps) {
  const router = useRouter()
  const location = useReactiveValue(router.$location)
  const [isNavVisible, setIsNavVisible] = React.useState(false)

  const app = getCurrentApp(apps, location)
  if (!app) {
    return NotFound
  }

  const view = app.routes[location.path](location.params)

  const navbar = (
    <NavBar
      apps={apps}
      currentApp={app}
      onClick={() => setIsNavVisible(false)}
      onLogout={onLogout}
    />
  )

  return (
    <div className={$container.className}>
      {!isNavVisible && (
        <Icon
          type="menu"
          $style={$menuIcon}
          onClick={() => setIsNavVisible(!isNavVisible)}
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
        {view}
      </div>
    </div>
  )
}
