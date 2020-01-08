import * as React from 'react'
import { useArhiv } from '~/arhiv/replica'
import {
  useRouter,
} from '~/web-router'
import {
  useObservable,
} from '~/web-platform'
import { Library } from '~/web-platform/Library'
import { AuthOverlay } from './chrome/AuthOverlay'
import { NotFound } from './parts'
import { WorkspaceView } from './workspace/WorkspaceView'

function parseIds(ids: string): string[] {
  if (!ids.length) {
    return []
  }

  return ids.split('-')
}

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
      const {
        ids,
        filter,
      } = location.params

      return (
        <WorkspaceView
          ids={parseIds(ids || '')}
          filter={filter || ''}
        />
      )
    }

    case '/library': {
      return (
        <Library />
      )
    }

    default: {
      return NotFound
    }
  }
}
