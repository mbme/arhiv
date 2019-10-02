import * as React from 'react'
import {
  formatTs,
  fuzzySearch,
} from '~/utils'
import { useObservable } from '~/utils/react'
import { useRouter } from '~/web-router'
import {
  stylish,
  Button,
  FilterInput,
  CleanLink,
  Link,
  Box,
  Spacer,
} from '~/web-platform'
import { useArhiv } from '~/arhiv'
import { Toolbar } from '../parts'

const $item = stylish({
  mb: 'medium',
  display: 'flex',
  alignItems: 'baseline',
  cursor: 'pointer',
})

const $counter = stylish({
  ml: 'small',
  whiteSpace: 'nowrap',
})

interface IProps {
  filter: string
}

export function NotesListView({ filter }: IProps) {
  const router = useRouter()
  const arhiv = useArhiv()

  const notes = useObservable(() => arhiv.notes.getDocuments$())

  const items = (notes || [])
    .filter(note => fuzzySearch(filter, note.record.name))
    .map(note => (
      <CleanLink
        key={note.id}
        to={{ path: '/note', params: { id: note.id } }}
        $style={$item}
      >
        <Box as="small" mr="small">
          {formatTs(note.record._updatedTs)}
        </Box>

        {note.record.name}
      </CleanLink>
    ))

  return (
    <>
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

      <small className={$counter.className}>
        {items.length} items
      </small>

      {items}
    </>
  )
}
