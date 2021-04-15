import * as React from 'react'
import { Procedure } from '@v/utils'
import {
  Box,
} from '@v/web-platform'
import {
  copyTextToClipboard,
} from '@v/web-utils'
import { API, createRef, IDocument } from '../../../api'
import { CardData, ConfirmationButton, useActions } from '../../../parts'
import { CardEditorForm } from './CardEditorForm'

interface IProps {
  document: IDocument
  onCancel: Procedure
  onSave: Procedure
  onArchive?: Procedure
}

export function CardEditor(props: IProps) {
  const {
    document,
    onCancel,
    onSave,
    onArchive,
  } = props

  const [preview, showPreview] = React.useState(false)
  const formRef = React.useRef(null)

  const saveDocument = async () => {
    await API.put({
      document: {
        ...document,
        data: formRef.current!,
      },
    })

    onSave()
  }

  const archiveDocument = async () => {
    await API.put({
      document: {
        ...document,
        archived: true,
      },
    })

    onArchive!()
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
          const attachmentIds = await API.pick_attachments()

          copyTextToClipboard(attachmentIds.map(createRef).join(' '))
        },
        children: 'Attach File',
      },
      {
        onClick: () => showPreview(true),
        children: 'Show Preview',
      },
    ]
  }, [preview, saveDocument, onCancel])

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
        />
      )}

      {onArchive && !preview && (
        <ConfirmationButton
          name="Archive Document"
          confirmation="Archive"
          prompt={<>Are you sure you want to <b>archive this document?</b></>}
          onConfirmed={archiveDocument}
        />
      )}
    </>
  )
}
