import * as React from 'react'
import { Procedure } from '@v/utils'
import {
  Box,
} from '@v/web-platform'
import { DeleteDocumentButton } from './DeleteDocumentButton'
import { API, IDocument } from '../../../api'
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
  const formRef = React.useRef(null)

  const saveDocument = async () => {
    await API.put({
      document: {
        ...document,
        data: formRef.current!,
      },
      newAttachments: [], // FIXME new attachments
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
          data={document.data}
        />
      </Box>

      {preview && (
        <CardData
          data={formRef.current!}
        />
      )}

      {onDelete && <DeleteDocumentButton onConfirmed={deleteDocument} />}
    </>
  )
}
