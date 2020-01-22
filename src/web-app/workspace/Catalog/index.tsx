import * as React from 'react'
import {
  useObservable,
  Input,
  Box,
  ProgressLocker,
  Column,
} from '~/web-platform'
import { useArhiv } from '~/arhiv/useArhiv'
import { useWorkspaceManager } from '../useWorkspaceManager'
import { CatalogEntry } from './Entry'

export function Catalog() {
  const ws = useWorkspaceManager()
  const arhiv = useArhiv()

  const [documents] = useObservable(
    () => arhiv.documents.getDocuments$({ filter: ws.filter }),
    [ws.filter],
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
      />
    ))

  return (
    <Column
      width="500px"
      maxWidth="100%"
      alignX="stretch"
      mx="auto"
    >
      <Box
        py="fine"
        width="100%"
      >
        <Input
          name="filter"
          light
          defaultValue={ws.filter}
          placeholder="Filter documents"
          onChange={newFilter => ws.updateFilter(newFilter)}
          onClear={() => ws.updateFilter('')}
          autoFocus
        />
      </Box>

      <Box
        as="small"
        pl="medium"
        mb="medium"
        display="block"
      >
        {items.length} items
      </Box>

      {items}
    </Column>
  )
}
