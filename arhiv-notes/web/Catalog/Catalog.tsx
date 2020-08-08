import * as React from 'react'
import {
  ProgressLocker,
} from '@v/web-platform'
import { usePromise } from '@v/web-utils'
import { API } from '../notes'
import { CatalogEntry } from './CatalogEntry'
import { ErrorBlock } from '../parts'
import { Header } from './Header'

export function Catalog() {
  const [filter, setFilter] = React.useState('')
  const [notes, err] = usePromise(() => API.list(filter), [filter])

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

  return (
    <>
      <Header
        filter={filter}
        onChange={setFilter}
      />

      {items}
    </>
  )
}
