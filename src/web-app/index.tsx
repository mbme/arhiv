import * as React from 'react'
import * as ReactDOM from 'react-dom'
import App from './app'
import './static/styles.css'

const rootEl = document.getElementById('root')!

ReactDOM.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
  rootEl,
  () => {
    rootEl.style.visibility = 'visible'
  }
)
