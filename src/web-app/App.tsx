import * as React from 'react'
import {
  useRouter,
} from '~/web-router'
import {
  Box,
} from '~/web-platform'
import { useObservable } from '~/web-utils'
import { Library } from '~/web-platform/Library'
import {
  NotFound,
  AuthOverlay,
} from './parts'
import { WorkspaceView } from './workspace/WorkspaceView'
import { ArhivContext } from './arhiv-context'

export function App() {
  const router = useRouter()
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
        <WorkspaceView />
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
