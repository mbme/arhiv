import * as React from 'react'
import {
  style,
} from 'typestyle'
import {
  formatTs,
  fuzzySearch,
} from '~/utils'
import { useRouter } from '~/web-router'
import { useIsodb } from '~/isodb-web-client'
import {
  theme,
  margin,
  Button,
  FilterInput,
  CleanLink,
  Link,
} from '~/web-components'
import { Toolbar } from '../parts'

const itemStyles = style({
  marginBottom: theme.spacing.medium,
  display: 'flex',
  alignItems: 'baseline',
  cursor: 'pointer',
})

const counterStyles = style({
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
        className={itemStyles}
      >
        <small className={margin({ right: 'small' })}>
          {formatTs(note.updatedTs)}
        </small>
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

      <small className={counterStyles}>
        {items.length} items
      </small>

      {items}
    </>
  )
}
