import * as React from 'react'
import {
  useObservable,
  FilterInput,
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
    <Column>
      <Box
        py="fine"
      >
        <FilterInput
          placeholder="Filter notes"
          filter={ws.filter}
          onChange={newFilter => ws.updateFilter(newFilter)}
          alwaysExpanded
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

      <Box
        display="flex"
        flexDirection="column"
        width="500px"
        maxWidth="100%"
      >
        {items}
      </Box>
    </Column>
  )
}
