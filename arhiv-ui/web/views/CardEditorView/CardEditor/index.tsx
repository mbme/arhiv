import * as React from 'react'
import { Procedure } from '@v/utils'
import {
  Box,
} from '@v/web-platform'
import { DeleteDocumentButton } from './DeleteDocumentButton'
import { API, IAttachmentSource, IDocument } from '../../../api'
import { CardData, useActions } from '../../../parts'
import { CardEditorForm } from './CardEditorForm'

interface IProps {
  document: IDocument
  onCancel: Procedure
  onSave: Procedure
  onDelete?: Procedure
}

export function CardEditor(props: IProps) {
  const {
    document,
    onCancel,
    onSave,
    onDelete,
  } = props

  const [preview, showPreview] = React.useState(false)
  const [newAttachments, setNewAttachments] = React.useState<IAttachmentSource[]>([])
  const formRef = React.useRef(null)

  const saveDocument = async () => {
    await API.put({
      document: {
        ...document,
        data: formRef.current!,
      },
      newAttachments,
    })

    onSave()
  }

  const deleteDocument = async () => {
    await API.put({
      document: {
        ...document,
        archived: true,
      },
      newAttachments: [],
    })

    onDelete!()
  }

  useActions(() => {
    if (preview) {
      return [
        {
          onClick: () => showPreview(false),
          children: 'Back',
        },
      ]
    }

    return [
      {
        onClick: saveDocument,
        children: 'Save Document',
      },
      {
        onClick: onCancel,
        children: 'Cancel',
      },
      {
        async onClick() {
          const attachments = await API.pick_attachments()
          setNewAttachments(prevState => [...prevState, ...attachments])
        },
        children: 'Attach File',
      },
      {
        onClick: () => showPreview(true),
        children: 'Show Preview',
      },
    ]
  }, [preview])

  return (
    <>
      <Box hidden={preview}>
        <CardEditorForm
          ref={formRef}
          documentType={document.documentType}
          data={document.data}
        />
      </Box>

      {preview && (
        <CardData
          documentType={document.documentType}
          data={formRef.current!}
          newAttachments={newAttachments}
        />
      )}

      {onDelete && !preview && <DeleteDocumentButton onConfirmed={deleteDocument} />}
    </>
  )
}
