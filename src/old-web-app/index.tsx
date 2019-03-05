import * as React from 'react'
import * as ReactDOM from 'react-dom'

import './static/styles.css'
import { StoreProvider } from './store'
import App from './App'

const rootEl = document.getElementById('root')!

ReactDOM.render(
  <React.StrictMode>
    <StoreProvider>
      <App />
    </StoreProvider>
  </React.StrictMode>,
  rootEl,
  () => {
    rootEl.style.visibility = 'visible'
  }
)
