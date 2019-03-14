import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { cssRaw } from 'typestyle'

import {
  globalStyles,
  OverlayRenderer,
} from '~/web-components'
import {
  Router,
  IRoute,
} from '~/web-router'

import { IApp, View } from './chrome'
import AppNotes from './app-notes'
import AppLibrary from './app-library'

cssRaw(`
  ${globalStyles}

  #root {
    overflow-y: scroll;
    height: 100vh;
    visibility: hidden;
  }
`)

const apps: IApp[] = [
  AppNotes,
  AppLibrary,
]

function renderView(route: IRoute) {
  return (
    <OverlayRenderer>
      <View route={route} apps={apps} />
    </OverlayRenderer>
  )
}

const rootEl = document.getElementById('root')!

ReactDOM.render(
  <React.StrictMode>
    <Router renderView={renderView} />
  </React.StrictMode>,
  rootEl,
  () => {
    rootEl.style.visibility = 'visible'
  },
)
