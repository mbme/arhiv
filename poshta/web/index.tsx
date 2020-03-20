// eslint-disable-next-line @typescript-eslint/triple-slash-reference
/// <reference path="../../app-shell/src/rpc.d.ts" />
import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { configureLogger, createLogger } from '@v/logger'
import { Box, globalStyles, injectGlobalStyles } from '@v/web-platform'
import { Gmail } from './gmail'

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

console.log(localStorage.getItem('test'))
localStorage.setItem('test', 'ok')
async function run() {
  const callResult: any = await window.RPC.call('get_token')
  const token = callResult.value

  log.info('Got auth token', token)

  const gmail = new Gmail(token)

  console.error(await gmail.getProfile())
  console.error(await gmail.listMessages(undefined, 10).loadNextPage())
}

run().catch(console.error)
