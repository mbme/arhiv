import * as React from 'react'
import {
  useObservable,
  FilterInput,
  Box,
  ProgressLocker,
} from '~/web-platform'
import { useArhiv } from '~/arhiv/useArhiv'
import { useWorkspaceManager } from '../useWorkspaceManager'
import { CatalogEntry } from './Entry'

export function Catalog() {
  const ws = useWorkspaceManager()
  const arhiv = useArhiv()

  const [documents] = useObservable(() => arhiv.documents.getDocuments$())

  if (!documents) {
    return (
      <ProgressLocker />
    )
  }

  const items = documents
    .filter(document => document.matches(ws.filter))
    .map(document => (
      <CatalogEntry
        key={document.id}
        document={document}
      />
    ))

  return (
    <Box
      as="aside"
      width="360px"
      height="100%"
      overflowY="scroll"
      bgColor="bgDarker"
    >
      <Box
        position="sticky"
        top="0"
        bgColor="bg"
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

      {items}
    </Box>
  )
}
