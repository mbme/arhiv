import 'tachyons'
import './static/styles.css'
import preact from 'preact'

const rootEl = document.getElementById('root')!

preact.render(
  <h1>HELLO WORLD!</h1>,
  rootEl
)

rootEl.style.visibility = 'visible'
