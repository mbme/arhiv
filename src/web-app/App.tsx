import * as React from 'react'
import {
  useRouter,
} from '~/web-router'
import {
  useObservable,
} from '~/web-platform'
import { Library } from '~/web-platform/Library'
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
  const [location] = useObservable(() => router.location$.value$)

  if (!location) {
    return null
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
