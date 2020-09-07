import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { configureLogger } from '@v/logger'
import {
  injectGlobalStyles,
} from '@v/web-utils'
import {
  globalStyles,
} from '@v/web-platform'
import { App } from './App'

configureLogger({ minLogLevel: 'INFO' })

injectGlobalStyles(`
  ${globalStyles}

  #root {
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

render(App)
