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

import { CatalogView } from './views/CatalogView'
import { CardView } from './views/CardView'
import {
  NewCardEditorView,
  CardEditorView,
} from './views/CardEditorView'
import { MetadataView } from './views/MetadataView'

export function App() {
  const [dataManager] = React.useState(() => new DataManager(MODULES))

  return (
    <PlatformProvider>
      <DataManagerContext.Provider value={dataManager}>
        <Routes
          onNotFound={() => <NotFoundBlock>View not found</NotFoundBlock>}
        >
          {[
            [pm`/`, () => <Redirect to="/catalog/note" />], // TODO status board

            [
              pm`/catalog/${'documentType'}`,
              ({ documentType }) => <CatalogView key={documentType} documentType={documentType}/>,
            ],
            [
              pm`/catalog/${'documentType'}/new`,
              ({ documentType }) => <NewCardEditorView key={documentType} documentType={documentType} />,
            ],
            [
              pm`/documents/${'id'}`,
              ({ id }) => <CardView key={id} id={id} />,
            ],
            [
              pm`/documents/${'id'}/metadata`,
              ({ id }) => <MetadataView key={id} id={id} />,
            ],
            [
              pm`/documents/${'id'}/edit`,
              ({ id }) => <CardEditorView key={id} id={id} />,
            ],
          ]}
        </Routes>

        {!window.RPC_URL && process.env.NODE_ENV === 'development' && (
          <Url />
        )}
      </DataManagerContext.Provider>
    </PlatformProvider>
  )
}
