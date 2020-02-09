import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { configureLogger } from '~/logger'
import { ArhivReplica } from '~/arhiv/replica'
import {
  injectGlobalStyles,
  globalStyles,
  OverlayRenderer,
} from '~/web-platform'
import {
  RouterContext,
  WebRouter,
} from '~/web-router'
import { App } from './App'
import { ArhivContext } from './arhiv-context'

const isDev = true

configureLogger({
  minLogLevel: isDev ? 'INFO' : 'WARN',
})

injectGlobalStyles(`
  ${globalStyles}

  #root {
    height: 100vh;
    visibility: hidden;
  }
`)

const rootEl = document.getElementById('root')
if (!rootEl) {
  throw new Error("Can't find #root element")
}

ArhivReplica.create().then((arhiv) => {
  ReactDOM.render(
    <React.StrictMode>
      <ArhivContext.Provider value={arhiv}>
        <RouterContext.Provider value={new WebRouter()}>
          <OverlayRenderer>
            <App />
          </OverlayRenderer>
        </RouterContext.Provider>
      </ArhivContext.Provider>
    </React.StrictMode>,
    rootEl,
    () => {
      rootEl.style.visibility = 'visible'
    },
  )
}).catch((err) => {
  // eslint-disable-next-line no-console
  console.error('Failed to initialize arhiv', err)
})
