import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { configureLogger } from '@v/logger'
import { injectGlobalStyles, globalStyles } from '@v/web-platform'
import { App } from './App'

configureLogger({ minLogLevel: 'DEBUG' })

injectGlobalStyles(`
  ${globalStyles}

  body {
    overflow-y: scroll;
  }

  #root {
    visibility: hidden;
    background-color: lightgray;
    height: auto;
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
