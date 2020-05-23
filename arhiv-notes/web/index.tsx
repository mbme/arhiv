// eslint-disable-next-line @typescript-eslint/triple-slash-reference
/// <reference path="../../app-shell/src/rpc.d.ts" />
import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { configureLogger } from '@v/logger'
import {
  injectGlobalStyles,
  HotkeysResolverProvider,
  Route,
  RouterProvider,
} from '@v/web-utils'
import {
  globalStyles,
  StylishProvider,
  OverlayRenderer,
} from '@v/web-platform'
import { App } from './App'
import { pathMatcher as pm } from '@v/utils'

configureLogger({ minLogLevel: 'INFO' })

injectGlobalStyles(`
  ${globalStyles}

  #root {
    visibility: hidden;
  }
`)

function run() {
  const rootEl = document.getElementById('root')
  if (!rootEl) {
    throw new Error("Can't find #root element")
  }

  ReactDOM.render(
    <React.StrictMode>
      <RouterProvider hashBased>
        <StylishProvider>
          <HotkeysResolverProvider>
            <OverlayRenderer>
              <Route pm={pm`/`}>
                {() => <App />}
              </Route>
            </OverlayRenderer>
          </HotkeysResolverProvider>
        </StylishProvider>
      </RouterProvider>
    </React.StrictMode>,
    rootEl,
    () => {
      rootEl.style.visibility = 'visible'
    },
  )
}

run()
