import * as React from 'react'
import { Procedure } from '@v/utils'
import {
  Input,
  Spacer,
  Textarea,
  useTextareaController,
  useForm,
  Box,
} from '@v/web-platform'
import { Frame, Action } from '../../parts'
import { Note } from '../Note'
import { AddAttachmentButton } from './AddAttachmentButton'
import { DeleteDocumentButton } from './DeleteDocumentButton'

interface IProps {
  name: string
  data: string
  onSave(name: string, data: string): void
  onCancel: Procedure
  onDelete?: Procedure
}

export function CardEditor(props: IProps) {
  const {
    name: originalName,
    data: originalData,
    onSave,
    onCancel,
    onDelete,
  } = props

  const {
    Form,
    values: {
      name = '',
      data = '',
    },
  } = useForm({ name: originalName, data: originalData })

  const [preview, showPreview] = React.useState(false)

  const isValid = name && (name !== originalName || data !== originalData)

  const textAreaRef = React.useRef<HTMLTextAreaElement>(null)
  const textAreaController = useTextareaController(textAreaRef)

  const onAttachments = (links: string[]) => {
    if (links.length) {
      textAreaController.insert(links.join(' '))
    }
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
        onClick={() => onSave(name, data)}
        disabled={!isValid}
      >
        Save Note
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

      <AddAttachmentButton onAttachments={onAttachments} />

      {onDelete && <DeleteDocumentButton onConfirmed={onDelete} />}
    </>
  )

  return (
    <Frame
      actions={actions}
      title="Card Editor"
    >
      <Box hidden={preview}>
        <Form>
          <Input
            label="Name"
            name="name"
            placeholder="Name"
          />

          <Spacer height="medium" />

          <Textarea
            label="Data"
            name="data"
            placeholder="Data"
            ref={textAreaRef}
          />

          <Spacer height="medium" />
        </Form>
      </Box>

      {preview && (
        <Note name={name} data={data} />
      )}
    </Frame>
  )
}
