import * as React from 'react'
import { RouterContext } from '@v/web-utils'
import { Catalog } from '../parts'
import { Note, NoteDataDescription } from './note'
import { Matcher } from '../api'

export function NoteCatalog() {
  const router = RouterContext.use()

  const getMatchers = (filter: string): Matcher[] => [
    { Type: 'note' },
    { Data: { selector: '$.name', pattern: filter } },
  ]

  const onAdd = () => router.push('/notes/new')
  const onActivate = (document: Note) => router.push(`/notes/${document.id}`)

  return (
    <Catalog
      title="Notes Catalog"
      dataDescription={NoteDataDescription}
      getMatchers={getMatchers}
      onAdd={onAdd}
      onActivate={onActivate}
    />
  )
}
