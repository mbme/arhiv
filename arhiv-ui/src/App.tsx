import * as React from 'react'
import { pathMatcher as pm } from '@v/utils'
import {
  Routes, usePromise,
} from '@v/web-utils'
import {
  PlatformProvider,
} from '@v/web-platform'
import { Frame, NotFoundBlock } from './parts'

import { DataManager, DataManagerContext } from './data-manager'

import { CatalogView } from './views/CatalogView'
import { CardView } from './views/CardView'
import {
  NewCardEditorView,
  CardEditorView,
} from './views/CardEditorView'
import { MetadataView } from './views/MetadataView'
import { DashboardView } from './views/DashboardView'
import { StatusView } from './views/StatusView'
import { API } from './api'

export function App() {
  const [dataManager] = usePromise(async () => {
    const SCHEMA = await API.get_schema()

    return new DataManager(SCHEMA)
  }, [])

  if (!dataManager) {
    return null
  }

  return (
    <PlatformProvider>
      <DataManagerContext.Provider value={dataManager}>
        <Frame>
          <Routes
            onNotFound={() => <NotFoundBlock>View not found</NotFoundBlock>}
          >
            {[
              [
                pm`/`,
                () => <DashboardView />
              ], // TODO status board

              [
                pm`/status`,
                () => <StatusView />,
              ],

              [
                pm`/catalog/attachment`,
                () => <CatalogView key="attachment" documentType="attachment" skipAddDocumentAction />,
              ],
              [
                pm`/catalog/${'documentType'}`,
                ({ documentType }) => <CatalogView key={documentType} documentType={documentType} />,
              ],
              [
                pm`/documents/${'documentType'}/new`,
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
        </Frame>
      </DataManagerContext.Provider>
    </PlatformProvider>
  )
}
