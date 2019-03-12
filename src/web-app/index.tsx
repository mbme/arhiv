import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { cssRaw } from 'typestyle'

import {
  Router,
  IRoute,
  Redirect,
  Link,
} from '~/web-router'
import {
  globalStyles,
  Library,
  OverlayRenderer,
  ProgressLocker,
} from '~/web-components'

cssRaw(`
  ${globalStyles}

  #root {
    overflow-y: scroll;
    height: 100vh;
    visibility: hidden;
  }
`)

const rootEl = document.getElementById('root')!

function renderView(route: IRoute) {
  if (route.path === '/') {
    return (
      <Redirect to={{ path: '/notes' }} />
    )
  }

  if (route.path === '/library') {
    return (
      <Library />
    )
  }

  return (
    <code>
      <Link to={{ path: '/test' }}>
        Test!
      </Link>
      {JSON.stringify(route, null, 2)}
      <ProgressLocker />
    </code>
  )
}

ReactDOM.render(
  <React.StrictMode>
    <OverlayRenderer>
      <Router renderView={renderView} />
    </OverlayRenderer>
  </React.StrictMode>,
  rootEl,
  () => {
    rootEl.style.visibility = 'visible'
  },
)
