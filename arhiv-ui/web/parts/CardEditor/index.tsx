import * as React from 'react'
import { Obj, Procedure } from '@v/utils'
import {
  Box,
} from '@v/web-platform'
import { DeleteDocumentButton } from './DeleteDocumentButton'
import { DocumentDataDescription } from '../../data-description'
import { API, IDocument } from '../../api'
import { CardEditorForm } from './CardEditorForm'
import { CardData } from '../CardData'
import { Action, Frame } from '..'

interface IProps<P extends Obj> {
  document: IDocument<string, P>
  dataDescription: DocumentDataDescription<P>
  onCancel: Procedure
  onSave: Procedure
  onDelete?: Procedure
}

export function CardEditor<P extends Obj>(props: IProps<P>) {
  const {
    document,
    dataDescription,
    onCancel,
    onSave,
    onDelete,
  } = props

  const [preview, showPreview] = React.useState(false)
  const formRef = React.useRef<P>(null)

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

  const actions = preview ? (
    <Action
      type="action"
      onClick={() => showPreview(false)}
    >
      Back
    </Action>
  ) : (
    <>
      <Action
        type="action"
        onClick={saveDocument}
      >
        Save Document
      </Action>

      <Action
        type="action"
        onClick={onCancel}
      >
        Cancel
      </Action>

      <Action
        type="action"
        onClick={() => showPreview(true)}
      >
        Show Preview
      </Action>

      {onDelete && <DeleteDocumentButton onConfirmed={deleteDocument} />}
    </>
  )

  return (
    <Frame
      actions={actions}
      title="Card Editor"
    >
      <Box hidden={preview}>
        <CardEditorForm
          ref={formRef}
          data={document.data}
          dataDescription={dataDescription}
        />
      </Box>

      {preview && (
        <CardData
          data={formRef.current!}
          dataDescription={dataDescription}
        />
      )}
    </Frame>
  )
}
