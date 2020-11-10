import * as React from 'react'
import {
  RouterContext,
} from '@v/web-utils'
import { createNote, NoteDataDescription } from '../api'
import { CardEditor, CardLoader } from '../parts'

interface IProps {
  id?: string
}

export function NoteCardEditor({ id }: IProps) {
  const router = RouterContext.use()

  const onCancel = () => {
    if (id) {
      router.replace({ path: `/notes/${id}` })
    } else {
      router.goBack()
    }
  }

  const onDelete = () => router.push({ path: '/notes' })

  return (
    <CardLoader
      id={id}
      createDocument={createNote}
    >
      {document => (
        <CardEditor
          document={document}
          dataDescription={NoteDataDescription}
          onCancel={onCancel}
          onSave={() => router.replace({ path: `/notes/${document.id}` })}
          onDelete={id ? onDelete : undefined}
        />
      )}
    </CardLoader>
  )
}
