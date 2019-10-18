import * as React from 'react'
import * as ReactDOM from 'react-dom'

import { setLogLevel } from '~/utils'
import {
  Arhiv,
  ArhivContext,
} from '~/arhiv'
import {
  injectGlobalStyles,
  globalStyles,
  OverlayRenderer,
} from '~/web-platform'
import {
  RouterContext,
  WebRouter,
} from '~/web-router'

import {
  IApp,
  Chrome,
  AuthManager,
} from './chrome'
import { NotesApp } from './app-notes'
import { LibraryApp } from './app-library'

setLogLevel('DEBUG')

injectGlobalStyles(`
  ${globalStyles}

  #root {
    overflow-y: scroll;
    height: 100vh;
    visibility: hidden;
  }

  /* helps to debug layout https://dev.to/gajus/my-favorite-css-hack-32g3 */
  html.debug * {
    background: rgba(255, 0, 0, .1);
    box-shadow: 0 0 0 1px red;
  }
`)

const arhiv = new Arhiv()
const router = new WebRouter()

const apps: IApp[] = [
  NotesApp,
  LibraryApp,
]

const rootEl = document.getElementById('root')!

ReactDOM.render(
  <React.StrictMode>
    <ArhivContext.Provider value={arhiv}>
      <RouterContext.Provider value={router}>
        <OverlayRenderer>
          <Chrome
            apps={apps}
            onLogout={() => arhiv.deauthorize()}
          />
          <AuthManager arhiv={arhiv} />
        </OverlayRenderer>
      </RouterContext.Provider>
    </ArhivContext.Provider>
  </React.StrictMode>,
  rootEl,
  () => {
    rootEl.style.visibility = 'visible'
  },
)
