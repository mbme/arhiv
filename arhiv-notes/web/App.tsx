import * as React from 'react'
import { Column, ProgressLocker } from '@v/web-platform'
import { usePromise } from '@v/web-utils'
import { API } from './notes'
import { CatalogEntry } from './CatalogEntry'

export function App() {
  const [notes] = usePromise(() => API.list(), [])

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
    <Column
      width="500px"
      maxWidth="100%"
      alignX="stretch"
      mx="auto"
    >
      {items}
    </Column>
  )
}
