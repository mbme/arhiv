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

interface IState {
  filter: string
  focusedId?: string
}

// FIXME use global state
const STATE: IState = {
  filter: '',
  focusedId: undefined,
}

export function Catalog() {
  const {
    Form,
    values: {
      filter = '',
    },
  } = useForm({ filter: STATE.filter })

  const debouncedFilter = useDebounced(filter, 300)

  const [notes, err] = usePromise(() => API.list(debouncedFilter), [debouncedFilter])

  // save filter in a temp variable
  React.useEffect(() => {
    STATE.filter = debouncedFilter
  }, [debouncedFilter])

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
        autoFocus={note.id === STATE.focusedId}
        onFocus={() => { STATE.focusedId = note.id }}
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
          pb="small"
          width="100%"
        >
          <Form>
            <Input
              label=""
              name="filter"
              placeholder="Filter documents"
              onFocus={() => { STATE.focusedId = undefined }}
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
