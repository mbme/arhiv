import * as React from 'react'
import { pathMatcher as pm } from '@v/utils'
import {
  ILocation,
  Redirect,
  Routes,
} from '@v/web-utils'
import {
  PlatformProvider,
} from '@v/web-platform'
import { NotFoundBlock, Url } from './parts'

import { DataManager, DataManagerContext } from './data-manager'
import { MODULES } from './api'

import { CatalogView, DOCUMENT_TYPE_QUERY_PARAM } from './views/CatalogView'
import { CardView } from './views/CardView'
import { CardEditorView } from './views/CardEditorView'

const defaultLocation: ILocation = {
  path: '/documents',
  params: [{ name: DOCUMENT_TYPE_QUERY_PARAM, value: 'note' }],
}

export function App() {
  const [dataManager] = React.useState(() => new DataManager(MODULES))

  return (
    <PlatformProvider>
      <DataManagerContext.Provider value={dataManager}>
        <Routes
          onNotFound={() => <NotFoundBlock>View not found</NotFoundBlock>}
        >
          {[
            [pm`/`, () => <Redirect to={defaultLocation} />], // TODO status board

            [pm`/documents`, () => <CatalogView />],
            [pm`/documents/new`, () => <CardEditorView />],
            [pm`/documents/${'id'}`, ({ id }) => <CardView id={id} />],
            [pm`/documents/${'id'}/edit`, ({ id }) => <CardEditorView id={id} />],
          ]}
        </Routes>

        {!window.RPC_URL && process.env.NODE_ENV === 'development' && (
          <Url />
        )}
      </DataManagerContext.Provider>
    </PlatformProvider>
  )
}
