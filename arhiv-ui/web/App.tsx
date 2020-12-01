import * as React from 'react'
import { pathMatcher as pm } from '@v/utils'
import {
  Redirect,
  Routes,
} from '@v/web-utils'
import {
  PlatformProvider,
} from '@v/web-platform'
import { NotFoundBlock, Url } from './parts'

import { DataManager, DataManagerContext } from './data-manager'
import { MODULES } from './api'

import { DocumentCatalogView } from './views/DocumentCatalogView'
import { DocumentCardView } from './views/DocumentCardView'
import { DocumentCardEditorView } from './views/DocumentCardEditorView'

export function App() {
  const [dataManager] = React.useState(() => new DataManager(MODULES))

  return (
    <PlatformProvider>
      <DataManagerContext.Provider value={dataManager}>
        <Routes
          onNotFound={() => <NotFoundBlock>View not found</NotFoundBlock>}
        >
          {[
            [pm`/`, () => <Redirect to="/documents" />], // TODO status board

            [pm`/documents`, () => <DocumentCatalogView />],
            [pm`/documents/new`, () => <DocumentCardEditorView />],
            [pm`/documents/${'id'}`, ({ id }) => <DocumentCardView id={id} />],
            [pm`/documents/${'id'}/edit`, ({ id }) => <DocumentCardEditorView id={id} />],
          ]}
        </Routes>

        {!window.RPC_URL && process.env.NODE_ENV === 'development' && (
          <Url />
        )}
      </DataManagerContext.Provider>
    </PlatformProvider>
  )
}
