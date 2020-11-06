import * as React from 'react'
import {
  Routes,
} from '@v/web-utils'
import {
  PlatformProvider,
} from '@v/web-platform'
import { NotFoundBlock } from './parts'
import { Url } from './Url'
import { routes as noteRoutes } from './notes'

export function App() {
  return (
    <PlatformProvider>
      <Routes
        onNotFound={() => <NotFoundBlock>View not found</NotFoundBlock>}
      >
        {[
          ...noteRoutes,
        ]}
      </Routes>

      {!window.RPC_URL && process.env.NODE_ENV === 'development' && (
        <Url />
      )}
    </PlatformProvider>
  )
}
