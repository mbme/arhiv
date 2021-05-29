import * as React from 'react'
import { pathMatcher as pm } from '@v/utils'
import {
  PlatformProvider,
  Routes,
  usePromise,
} from '@v/web-platform'
import {
  CatalogOptionsOverrides,
  Frame,
  NotFoundBlock,
} from './parts'

import { DataManager, DataManagerContext } from './data-manager'
import { API } from '@v/arhiv-api'

import { CatalogView } from './views/CatalogView'
import { CardView } from './views/CardView'
import {
  NewCardEditorView,
  CardEditorView,
} from './views/CardEditorView'
import { MetadataView } from './views/MetadataView'
import { DashboardView } from './views/DashboardView'
import { StatusView } from './views/StatusView'
import { DocumentRedirectView } from './views/DocumentRedirectView'

const PROJECT_CATALOG_OPTIONS: CatalogOptionsOverrides = {
  pageSize: undefined,
  showEntryModificationDate: false,
}

const PROJECT_CARD_CATALOG_OPTIONS: CatalogOptionsOverrides = {
  pageSize: undefined,
  groupByField: 'status',
  openGroups: ['Inbox', 'InProgress', 'Todo'],
  order: [
    {
      EnumField: {
        selector: '$.status',
        asc: true,
        enumOrder: ['Inbox', 'InProgress', 'Paused', 'Todo', 'Done', 'Later', 'Cancelled' ],
      },
    },
    {
      UpdatedAt: {
        asc: false,
      },
    },
  ],
  showEntryModificationDate: false,
  showEntryDataFields: ['status'],
}

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

              // CATALOG OVERRIDES
              [
                pm`/catalog/attachment`,
                () => <CatalogView key="attachment" documentType="attachment" skipAddDocumentAction />,
              ],

              [
                pm`/catalog/project`,
                () => <CatalogView key="project" documentType="project" catalogOptions={PROJECT_CATALOG_OPTIONS} />,
              ],

              [
                pm`/catalog/${'documentType'}`,
                ({ documentType }) => <CatalogView key={documentType} documentType={documentType} />,
              ],

              // CARD OVERRIDES
              [
                pm`/catalog/project/${'id'}`,
                ({ id }) => <CardView key={id} id={id} catalogOptions={PROJECT_CARD_CATALOG_OPTIONS} />,
              ],

              [
                pm`/catalog/${'documentType'}/${'id'}`,
                ({ id }) => <CardView key={id} id={id} />,
              ],

              // -----------------
              [
                pm`/documents/${'documentType'}/new`,
                ({ documentType }) => <NewCardEditorView key={documentType} documentType={documentType} />,
              ],
              [
                pm`/documents/${'id'}`,
                ({ id }) => <DocumentRedirectView key={id} id={id} />,
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
