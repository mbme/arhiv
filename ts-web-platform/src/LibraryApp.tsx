import * as React from 'react'
import * as ReactDOM from 'react-dom'

import './webpack-hot'
import { injectGlobalStyles } from '@v/web-utils'
import { globalStyles } from './core/global-styles'
import { Library } from './Library'

injectGlobalStyles(`
  ${globalStyles}

  #root {
    height: 100vh;
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
