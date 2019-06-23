import * as React from 'react'
import {
  formatTs,
  fuzzySearch,
} from '~/utils'
import { useRouter } from '~/web-router'
import { useIsodb } from '~/isodb-web-client'
import {
  theme,
  Button,
  FilterInput,
  CleanLink,
  Link,
  Box,
} from '~/web-platform'
import { stylish } from '~/stylish'
import { Toolbar } from '../parts'

const $item = stylish({
  marginBottom: theme.spacing.medium,
  display: 'flex',
  alignItems: 'baseline',
  cursor: 'pointer',
})

const $counter = stylish({
  marginLeft: theme.spacing.small,
  whiteSpace: 'nowrap',
})

interface IProps {
  filter: string
}

export function NotesListView({ filter }: IProps) {
  const router = useRouter()
  const client = useIsodb()

  const items = client.notes
    .getNotes()
    .filter(note => fuzzySearch(filter, note.name))
    .map(note => (
      <CleanLink
        key={note.id}
        to={{ path: '/note', params: { id: note.id } }}
        $style={$item}
      >
        <Box as="small" mr="small">
          {formatTs(note.updatedTs)}
        </Box>

        {note.name}
      </CleanLink>
    ))

  const left = (
    <FilterInput
      placeholder="Filter notes"
      filter={filter}
      onChange={newFilter => router.replaceParam('filter', newFilter)}
    />
  )

  const right = (
    <Link to={{ path: '/note-editor' }}>
      <Button primary>
        Add
      </Button>
    </Link>
  )

  return (
    <>
      <Toolbar left={left} right={right} />

      <small className={$counter.className}>
        {items.length} items
      </small>

      {items}
    </>
  )
}
