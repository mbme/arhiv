import * as React from 'react'
import {
  RouterContext,
} from '@v/web-utils'
import { CardLoader } from '../../parts'
import { API } from '../../api'
import { CardEditor } from './CardEditor'

interface IProps {
  documentType: string
}

export function NewCardEditorView({ documentType }: IProps) {
  const router = RouterContext.use()

  return (
    <CardLoader
      createDocument={() => API.create({ documentType, args: {} })}
    >
      {document => (
        <CardEditor
          document={document}
          onSave={() => router.replace({ path: `/documents/${document.id}` })}
          onCancel={() => router.goBack()}
        />
      )}
    </CardLoader>
  )
}
