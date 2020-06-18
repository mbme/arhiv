import * as React from 'react'
import * as ReactDOM from 'react-dom'

import { injectGlobalStyles } from '@v/web-utils'
import { globalStyles } from './core/global-styles'
import { Library } from './Library'
import { Obj } from '@v/utils'

injectGlobalStyles(`
  ${globalStyles}

  #root {
    height: 100vh;
    max-width: 50rem;
    margin: 0 auto;
    visibility: hidden;
  }
`)

function render() {
  const rootEl = document.getElementById('root')
  if (!rootEl) {
    throw new Error("Can't find #root element")
  }

  ReactDOM.render(
    <React.StrictMode>
      <Library />
    </React.StrictMode>,
    rootEl,
    () => {
      rootEl.style.visibility = 'visible'
    },
  )
}

render()

if ((module as Obj).hot) {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
  (module as Obj).hot.accept('./Library', render)
}
