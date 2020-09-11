import * as React from 'react'
import {
  ProgressLocker,
  StyleArg,
  useForm,
  Input,
  Row,
} from '@v/web-platform'
import { usePromise, useDebounced } from '@v/web-utils'
import { API } from '../types'
import { CatalogEntry } from './CatalogEntry'
import { ErrorBlock, Frame, Action } from '../parts'

const $header: StyleArg = {
  position: 'sticky',
  top: 0,
}

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
    >
      <Form>
        <Row
          as="nav"
          alignX="center"
          p="fine"
          width="100%"
          zIndex={1}
          $style={$header}
          bgColor="var(--color-bg0)"
        >
          <Input
            name="filter"
            placeholder="Filter documents"
          />
        </Row>
      </Form>

      {items}
    </Frame>
  )
}
