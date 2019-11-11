import * as React from 'react'
import * as ReactDOM from 'react-dom'

import {
  setLogLevel,
  debugLayoutSnippet,
} from '~/utils'
import {
  Arhiv,
  ArhivContext,
} from '~/arhiv'
import {
  injectGlobalStyles,
  globalStyles,
  OverlayRenderer,
} from '~/web-platform'
import {
  RouterContext,
  WebRouter,
} from '~/web-router'

import {
  IApp,
  Chrome,
  AuthManager,
} from './chrome'
import { NotesApp } from './app-notes'
import { LibraryApp } from './app-library'

const isDev = true

setLogLevel(isDev ? 'DEBUG' : 'WARN')

injectGlobalStyles(`
  ${globalStyles}

  #root {
    overflow-y: scroll;
    height: 100vh;
    visibility: hidden;
  }

  ${isDev ? debugLayoutSnippet : ''}
`)

const router = new WebRouter()

const apps: IApp[] = [
  NotesApp,
  LibraryApp,
]

async function runApp(rootEl: HTMLElement) {
  const arhiv = await Arhiv.create()

  ReactDOM.render(
    <React.StrictMode>
      <ArhivContext.Provider value={arhiv}>
        <RouterContext.Provider value={router}>
          <OverlayRenderer>
            <Chrome
              apps={apps}
              onLogout={() => arhiv.deauthorize()}
            />
            <AuthManager arhiv={arhiv} />
          </OverlayRenderer>
        </RouterContext.Provider>
      </ArhivContext.Provider>
    </React.StrictMode>,
    rootEl,
    () => {
      rootEl.style.visibility = 'visible'
    },
  )
}

runApp(document.getElementById('root')!)
