import * as React from 'react'
import * as ReactDOM from 'react-dom'
import {
  loggerConfig,
  createLogger,
} from '~/logger'
import {
  debugLayoutSnippet,
} from '~/utils'
import {
  ArhivReplica,
  ArhivContext,
} from '~/arhiv/replica'
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

const isDev = true

loggerConfig.minLogLevel = isDev ? 'DEBUG' : 'WARN'

const log = createLogger('web-app')

injectGlobalStyles(`
  ${globalStyles}

  #root {
    overflow-y: scroll;
    height: 100vh;
    visibility: hidden;
  }

  ${isDev ? debugLayoutSnippet : ''}
`)

const rootEl = document.getElementById('root')
if (!rootEl) {
  throw new Error("Can't find #root element")
}

ArhivReplica.create().then(
  (arhiv) => {
    const apps: IApp[] = [
      NotesApp,
      LibraryApp,
    ]

    ReactDOM.render(
      <React.StrictMode>
        <ArhivContext.Provider value={arhiv}>
          <RouterContext.Provider value={new WebRouter()}>
            <OverlayRenderer>
              <Chrome
                apps={apps}
                onLogout={() => arhiv.deauthorize()}
              />
              <AuthManager />
            </OverlayRenderer>
          </RouterContext.Provider>
        </ArhivContext.Provider>
      </React.StrictMode>,
      rootEl,
      () => {
        rootEl.style.visibility = 'visible'
      },
    )
  },
  (err) => {
    log.error('Failed to initialize arhiv', err)
  },
)
