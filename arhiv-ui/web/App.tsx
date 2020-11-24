import * as React from 'react'
import { pathMatcher as pm } from '@v/utils'
import {
  Redirect,
  Routes,
} from '@v/web-utils'
import {
  PlatformProvider,
} from '@v/web-platform'
import { NotFoundBlock } from './parts'
import { Url } from './Url'
import { routes as noteRoutes } from './notes'
import { routes as projectsTasksRoutes } from './projects-tasks'

export function App() {
  return (
    <PlatformProvider>
      <Routes
        onNotFound={() => <NotFoundBlock>View not found</NotFoundBlock>}
      >
        {[
          [pm`/`, () => <Redirect to="/notes" />], // TODO status board
          ...noteRoutes,
          ...projectsTasksRoutes,
        ]}
      </Routes>

      {!window.RPC_URL && process.env.NODE_ENV === 'development' && (
        <Url />
      )}
    </PlatformProvider>
  )
}
