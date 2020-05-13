// eslint-disable-next-line @typescript-eslint/triple-slash-reference
/// <reference path="../../app-shell/src/rpc.d.ts" />
import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { configureLogger, createLogger } from '@v/logger'
import { injectGlobalStyles, HotkeysResolverProvider } from '@v/web-utils'
import {
  globalStyles,
  StylishProvider,
  OverlayRenderer,
} from '@v/web-platform'
import { App } from './App'

configureLogger({ minLogLevel: 'INFO' })

const log = createLogger('arhiv-notes')

injectGlobalStyles(`
  ${globalStyles}

  #root {
    visibility: hidden;
  }
`)

async function run() {
  const rootEl = document.getElementById('root')
  if (!rootEl) {
    throw new Error("Can't find #root element")
  }

  ReactDOM.render(
    <React.StrictMode>
      <StylishProvider>
        <HotkeysResolverProvider>
          <OverlayRenderer>
            <App />
          </OverlayRenderer>
        </HotkeysResolverProvider>
      </StylishProvider>
    </React.StrictMode>,
    rootEl,
    () => {
      rootEl.style.visibility = 'visible'
    },
  )
}

run().catch(console.error)
