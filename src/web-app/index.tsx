import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { cssRaw } from 'typestyle'

import {
  Router,
  IRoute,
} from '~/web-router'

import globalStyles from './styles'

cssRaw(globalStyles)

const rootEl = document.getElementById('root')!

function renderView(route: IRoute) {
  return (
    <code>
      {JSON.stringify(route, null, 2)}
    </code>
  )
}

ReactDOM.render(
  <React.StrictMode>
    <Router renderView={renderView} />
  </React.StrictMode>,
  rootEl,
  () => {
    rootEl.style.visibility = 'visible'
  },
)
