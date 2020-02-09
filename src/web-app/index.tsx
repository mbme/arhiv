import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { ArhivReplica } from '~/arhiv/replica'
import { configureLogger } from '~/logger'
import { Box, globalStyles, injectGlobalStyles, OverlayRenderer } from '~/web-platform'
import { Library } from '~/web-platform/Library'
import { RouterContext, WebRouter } from '~/web-router'
import { useObservable } from '~/web-utils'
import { ArhivContext } from './arhiv-context'
import { NoteModule } from './module-note'
import { NotFound } from './parts'
import { useModule } from './workspace/modules'
import { WorkspaceViewContainer } from './workspace/WorkspaceViewContainer'

useModule(NoteModule)

configureLogger({ minLogLevel: 'INFO' })

injectGlobalStyles(`
  ${globalStyles}

  #root {
    height: 100vh;
    visibility: hidden;
  }
`)

const rootEl = document.getElementById('root')
if (!rootEl) {
  throw new Error("Can't find #root element")
}

function App() {
  const router = RouterContext.use()

  const [location] = useObservable(() => router.location$.value$)

  if (!location) {
    return null
  }

  switch (location.path) {
    case '/': {
      return (
        <WorkspaceViewContainer />
      )
    }

    case '/library': {
      return (
        <Box
          maxWidth="50rem"
          mx="auto"
          p="medium"
        >
          <Library />
        </Box>
      )
    }

    default: {
      return NotFound
    }
  }
}

ArhivReplica.create().then((arhiv) => {
  ReactDOM.render(
    <React.StrictMode>
      <ArhivContext.Provider value={arhiv}>
        <RouterContext.Provider value={new WebRouter()}>
          <OverlayRenderer>
            <App />
          </OverlayRenderer>
        </RouterContext.Provider>
      </ArhivContext.Provider>
    </React.StrictMode>,
    rootEl,
    () => {
      rootEl.style.visibility = 'visible'
    },
  )
}).catch((err) => {
  // eslint-disable-next-line no-console
  console.error('Failed to initialize arhiv', err)
})
