import * as React from 'react'
import * as ReactDOM from 'react-dom'

import { Procedure } from '@v/utils'
import { configureLogger } from '@v/logger'
import { injectGlobalStyles, globalStyles } from './core'
import { Library } from './Library'

configureLogger({ minLogLevel: 'DEBUG' })

injectGlobalStyles(`
  ${globalStyles}

  #root {
    min-height: 100vh;
    max-width: 50rem;
    margin: 0 auto;
    visibility: hidden;
  }
`)

function render(Component: React.ComponentType) {
  const rootEl = document.getElementById('root')
  if (!rootEl) {
    throw new Error("Can't find #root element")
  }

  ReactDOM.render(
    <React.StrictMode>
      <Component />
    </React.StrictMode>,
    rootEl,
    () => {
      rootEl.style.visibility = 'visible'
    },
  )
}

render(Library)

if (module.hot) {
  module.hot.accept('./Library', () => render(Library))
}

declare let module: {
  hot?: {
    accept(module: string, cb: Procedure): void
  }
}
