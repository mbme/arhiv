import * as React from 'react'
import { Box } from '~/web-platform'
import { Library } from '~/web-platform/Library'
import { RouterContext } from '~/web-router'
import { useObservable } from '~/web-utils'
import { ArhivContext } from './arhiv-context'
import { AuthOverlay, NotFound } from './parts'
import { WorkspaceViewContainer } from './workspace/WorkspaceViewContainer'

export function App() {
  const router = RouterContext.use()
  const arhiv = ArhivContext.use()

  const [location] = useObservable(() => router.location$.value$)
  const [authorized] = useObservable(() => arhiv.isAuthorized$.value$)

  if (!location) {
    return null
  }

  if (authorized === false) {
    return (
      <AuthOverlay submit={password => arhiv.authorize(password)} />
    )
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
