import * as React from 'react'
import {
  RouterContext,
} from '@v/web-platform'
import { CardLoader, FrameTitle } from '../../parts'
import { CardEditor } from './CardEditor'

interface IProps {
  id: string
}

export function CardEditorView({ id }: IProps) {
  const router = RouterContext.use()

  return (
    <CardLoader
      id={id}
    >
      {(document) => {
        const gotoCatalog = () => router.push({ path: `/catalog/${document.documentType}` })
        const closeEditor = () => router.replace({ path: `/documents/${id}` })

        return (
          <>
            <FrameTitle>
              {document.documentType} Editor
            </FrameTitle>

            <CardEditor
              document={document}
              onCancel={closeEditor}
              onSave={closeEditor}
              onArchive={gotoCatalog}
              onDelete={gotoCatalog}
            />
          </>
        )
      }}
    </CardLoader>
  )
}
