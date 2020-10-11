import * as React from 'react'
import {
  ProgressLocker,
  useForm,
  Input,
  Column,
  Box,
} from '@v/web-platform'
import { usePromise, useDebounced } from '@v/web-utils'
import { API } from '../types'
import { CatalogEntry } from './CatalogEntry'
import { ErrorBlock, Frame, Action } from '../parts'

export function Catalog() {
  const {
    Form,
    values: {
      filter = '',
    },
  } = useForm()

  const debouncedFilter = useDebounced(filter, 300)

  const [notes, err] = usePromise(() => API.list(debouncedFilter), [debouncedFilter])

  if (err) {
    return (
      <ErrorBlock error={err} />
    )
  }

  if (!notes) {
    return (
      <ProgressLocker />
    )
  }

  const items = notes
    .map(note => (
      <CatalogEntry
        key={note.id}
        note={note}
      />
    ))

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
          p="fine"
          width="100%"
          bgColor="var(--color-bg0)"
        >
          <Form>
            <Input
              label="Filter"
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
          {items}
        </Box>
      </Column>
    </Frame>
  )
}
