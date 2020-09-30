// eslint-disable-next-line @typescript-eslint/triple-slash-reference
/// <reference path="../../app-shell/src/rpc.d.ts" />
import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { configureLogger, createLogger } from '@v/logger'
import { injectGlobalStyles } from '@v/web-utils'
import {
  globalStyles,
  PlatformProvider,
} from '@v/web-platform'
import { GmailAPI } from './gmail'
import { PoshtaStore } from './poshta-store'
import { App } from './App'

configureLogger({ minLogLevel: 'INFO' })

const log = createLogger('poshta')

injectGlobalStyles(`
  ${globalStyles}

  #root {
    visibility: hidden;
  }
`)

async function run() {
  /* eslint-disable */
  const callResult: any = await window.RPC.call('get_token')
  const token = callResult.value

  log.info('Got auth token', token)

  const gmail = new GmailAPI(token)

  console.error(await gmail.getProfile())

  const store = new PoshtaStore(gmail)
  store.loadData()

  const rootEl = document.getElementById('root')
  if (!rootEl) {
    throw new Error("Can't find #root element")
  }

  ReactDOM.render(
    <React.StrictMode>
      <PlatformProvider>
        <App store={store} />
      </PlatformProvider>
    </React.StrictMode>,
    rootEl,
    () => {
      rootEl.style.visibility = 'visible'
    },
  )
}

run().catch(console.error)
