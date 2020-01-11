import * as React from 'react'
import {
  useRouter,
} from '~/web-router'
import {
  useObservable,
  Box,
} from '~/web-platform'
import { Library } from '~/web-platform/Library'
import { useArhiv } from './useArhiv'
import {
  NotFound,
  Chrome,
  AuthOverlay,
} from './parts'
import { WorkspaceView } from './workspace/WorkspaceView'

export function App() {
  const router = useRouter()
  const arhiv = useArhiv()

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
        <Chrome selected="library">
          <Box
            display="inline-block"
            maxWidth="50rem"
            p="medium"
          >
            <Library />
          </Box>
        </Chrome>
      )
    }

    default: {
      return NotFound
    }
  }
}
