import * as React from 'react'
import {
  ProgressLocker,
  useForm,
  Input,
  Column,
  Box,
  Button,
} from '@v/web-platform'
import { useDebounced } from '@v/web-utils'
import { CatalogEntry } from './CatalogEntry'
import { ErrorBlock, Frame, Action } from '../parts'
import { useList } from './useList'

export function Catalog() {
  const {
    Form,
    values: {
      filter = '',
    },
  } = useForm()

  const debouncedFilter = useDebounced(filter, 300)
  const {
    items,
    hasMore,
    error,
    loadMore,
  } = useList(debouncedFilter)

  if (error) {
    return (
      <ErrorBlock error={error} />
    )
  }

  const content = items ? (
    items
      .map(note => (
        <CatalogEntry
          key={note.id}
          note={note}
        />
      ))
  ) : (
    <ProgressLocker />
  )

  const actions = (
    <Action
      type="location"
      to={{ path: '/new' }}
    >
      Add Note
    </Action>
  )

  return (
    <Frame
      actions={actions}
      title="Catalog"
    >
      <Column
        height="100%"
        overflow="hidden"
        flex="0 0 auto"
      >
        <Box
          as="nav"
          pb="small"
          width="100%"
        >
          <Form>
            <Input
              label=""
              name="filter"
              placeholder="Filter documents"
            />
          </Form>
        </Box>

        <Box
          flex="1 1 auto"
          overflowY="auto"
          width="100%"
        >
          {content}

          {hasMore && (
            <Button
              variant="link"
              onClick={loadMore}
            >
              Load more
            </Button>
          )}
        </Box>
      </Column>
    </Frame>
  )
}
