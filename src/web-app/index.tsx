import * as React from 'react'
import * as ReactDOM from 'react-dom'

import { setLogLevel } from '~/logger'
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
  Router,
  ILocation,
} from '~/web-router'

import {
  IApp,
  View,
  AuthManager,
} from './chrome'
import AppNotes from './app-notes'
import AppLibrary from './app-library'

setLogLevel('INFO')

injectGlobalStyles(`
  ${globalStyles}

  #root {
    overflow-y: scroll;
    height: 100vh;
    visibility: hidden;
  }
`)

const arhiv = new Arhiv()

const apps: IApp[] = [
  AppNotes,
  AppLibrary,
]

function renderView(location: ILocation) {
  return (
    <OverlayRenderer>
      <View
        location={location}
        apps={apps}
        arhiv={arhiv}
      />
      <AuthManager arhiv={arhiv} />
    </OverlayRenderer>
  )
}

const rootEl = document.getElementById('root')!

ReactDOM.render(
  <React.StrictMode>
    <ArhivContext.Provider value={arhiv}>
      <Router renderView={renderView} />
    </ArhivContext.Provider>
  </React.StrictMode>,
  rootEl,
  () => {
    rootEl.style.visibility = 'visible'
  },
)
