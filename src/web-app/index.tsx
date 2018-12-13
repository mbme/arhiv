import 'tachyons'
import './static/styles.css'
import { render } from 'inferno'
import { h as infernoH } from 'inferno-hyperscript'
import App from './app'

// needed for tsx to work
(window as any)._h = infernoH

const rootEl = document.getElementById('root')!

render(<App />, rootEl, () => {
  rootEl.style.visibility = 'visible'
})
