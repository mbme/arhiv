import * as React from 'react'
import { ArhivContext } from '~/web-app/arhiv-context'
import { Column, ProgressLocker } from '~/web-platform'
import { useObservable } from '~/web-utils'
import { matches } from '../document-types'
import { CatalogEntry } from './Entry'
import { useWorkspaceStore } from '../store'

export function Catalog() {
  const arhiv = ArhivContext.use()

  const store = useWorkspaceStore()
  const [documents] = useObservable(
    () => arhiv.documents.getDocuments$({ matches: matches(store.state.filter) }),
    [store.state.filter],
  )

  if (!documents) {
    return (
      <ProgressLocker />
    )
  }

  const items = documents
    .map(document => (
      <CatalogEntry
        key={document.id}
        document={document}
        isOpen={store.isDocumentOpen(document.id)}
        onClick={() => store.openDocument(document.id)}
      />
    ))

  return (
    <Column
      width="500px"
      maxWidth="100%"
      alignX="stretch"
      mx="auto"
    >
      {items}
    </Column>
  )
}
