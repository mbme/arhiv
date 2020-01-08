import * as React from 'react'
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
import { formatDate } from '~/utils'

interface IProps {
  filter: string,
}

export function DeckOfCards({ filter }: IProps) {
  const router = useRouter()
  const arhiv = useArhiv()

  const [documents] = useObservable(() => arhiv.documents.getDocuments$())

  if (!documents) {
    return (
      <ProgressLocker />
    )
  }

  const items = documents
    .filter(document => document.matches(filter))
    .map(document => (
      <Box key={document.id}>
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
