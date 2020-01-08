import * as React from 'react'
import { useArhiv } from '~/arhiv/replica'
import {
  useObservable,
  Button,
  FilterInput,
  Link,
  Box,
  Spacer,
  ProgressLocker,
} from '~/web-platform'
import { formatDate } from '~/utils'
import { useWorkspaceManager } from './useWorkspaceManager'
import { Toolbar } from '../parts'

export function DeckOfCards() {
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
      <Box
        key={document.id}
        onClick={() => ws.openId(document.id)}
        mb="medium"
        cursor="pointer"
      >
        <Box as="small" mr="small">
          {formatDate(document.updatedAt)}
        </Box>

        <Box>
          {document.id}
        </Box>

        [{document.type}] {document.getTitle()}
      </Box>
    ))

  return (
    <Box
      width="360px"
      overflowY="scroll"
      p="small"
    >
      <Toolbar>
        <FilterInput
          placeholder="Filter notes"
          filter={ws.filter}
          onChange={newFilter => ws.updateFilter(newFilter)}
        />

        <Spacer />

        <Link to={{ path: '/note-editor' }}>
          <Button primary>
            Add
          </Button>
        </Link>
      </Toolbar>

      <small>
        {items.length} items
      </small>

      {items}
    </Box>
  )
}
