import 'virtual:windi.css'

import { API } from '@v/arhiv-api'

import App from './App.svelte'

API.get_schema().then((schema) => {
  const context = new Map()
  context.set('schema', schema)

  new App({
    target: document.getElementById('app')!,
    context,
  })
})
