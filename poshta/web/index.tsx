// eslint-disable-next-line @typescript-eslint/triple-slash-reference
/// <reference path="../../app-shell/src/rpc.d.ts" />
import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { configureLogger, createLogger } from '@v/logger'
import { Box, globalStyles, injectGlobalStyles } from '@v/web-platform'

configureLogger({ minLogLevel: 'INFO' })

const log = createLogger('poshta')

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

const BASE_URL = 'https://www.googleapis.com/gmail/v1/users/me'

async function run() {
  const callResult: any = await window.RPC.call('get_token')
  const token = callResult.value

  log.info('Got auth token', token)

  const response = await fetch(BASE_URL + '/profile', {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  })

  const result = await response.json()

  console.error(result)
}

run()
