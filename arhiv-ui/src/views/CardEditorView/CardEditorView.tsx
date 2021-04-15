import * as React from 'react'
import {
  RouterContext,
} from '@v/web-utils'
import { CardLoader, FrameTitle } from '../../parts'
import { CardEditor } from './CardEditor'

interface IProps {
  id: string
}

export function CardEditorView({ id }: IProps) {
  const router = RouterContext.use()

  const onCancel = () => router.replace({ path: `/documents/${id}` })
  const onArchive = () => router.push({ path: '/documents' })

  return (
    <CardLoader
      id={id}
    >
      {document => (
        <>
          <FrameTitle>
            {document.documentType} Editor
          </FrameTitle>

          <CardEditor
            document={document}
            onCancel={onCancel}
            onSave={() => router.replace({ path: `/documents/${document.id}` })}
            onArchive={onArchive}
          />
        </>
      )}
    </CardLoader>
  )
}
