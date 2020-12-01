import * as React from 'react'
import {
  RouterContext,
} from '@v/web-utils'
import { CardEditor, CardLoader } from '../parts'
import { API } from '../api'

interface IProps {
  id?: string
}

export function DocumentCardEditorView({ id }: IProps) {
  const router = RouterContext.use()

  const onCancel = () => {
    if (id) {
      router.replace({ path: `/documents/${id}` })
    } else {
      router.goBack()
    }
  }

  const onDelete = () => router.push({ path: '/documents' })

  return (
    <CardLoader
      id={id}
      createDocument={() => API.create({ documentType: 'note', args: {} })}
    >
      {document => (
        <CardEditor
          document={document}
          onCancel={onCancel}
          onSave={() => router.replace({ path: `/documents/${document.id}` })}
          onDelete={id ? onDelete : undefined}
        />
      )}
    </CardLoader>
  )
}
