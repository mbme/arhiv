import * as React from 'react'
import {
  fuzzySearch,
} from '~/utils'
import { useRouter } from '~/web-router'
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
import { Toolbar } from '../parts'

interface IProps {
  filter: string,
}

export function DeckOfCards({ filter }: IProps) {
  const router = useRouter()
  const arhiv = useArhiv()

  const [documents, isReady] = useObservable(() => arhiv.documents.getDocuments$())

  if (!isReady) {
    return (
      <ProgressLocker />
    )
  }

  const items = (documents || [])
    .filter(document => fuzzySearch(filter, document.id))
    .map(document => (
      <Box key={document.id}>
        <Box as="small" mr="small">
          {document.document._updatedAt}
        </Box>

        {document.document._type} {document.id}
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
          filter={filter}
          onChange={newFilter => router.replaceParam('filter', newFilter)}
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
