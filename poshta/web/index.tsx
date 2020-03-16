import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { configureLogger } from '@v/logger'
import { Box, globalStyles, injectGlobalStyles } from '@v/web-platform'

configureLogger({ minLogLevel: 'INFO' })

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

ReactDOM.render(
  <React.StrictMode>
    <Box>
      HELLO WORLD
    </Box>
  </React.StrictMode>,
  rootEl,
  () => {
    rootEl.style.visibility = 'visible'
  },
)
