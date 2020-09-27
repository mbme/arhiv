import * as React from 'react'
import { Procedure } from '@v/utils'
import {
  Input,
  Spacer,
  Textarea,
  useTextareaController,
  useForm,
  StyleArg,
} from '@v/web-platform'
import { Frame, Action } from '../parts'
import { Note } from '../Note'
import { AddAttachmentButton } from './AddAttachmentButton'
import { DeleteDocumentButton } from './DeleteDocumentButton'


const $container: StyleArg = {
  pt: 'large',
}

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
    textAreaController.insert(links.join(' '))
    textAreaController.focus()
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
      $style={$container}
    >
      <Form>
        {preview ? (
          <Note name={name} data={data} />
        ) : (
          <>
            <Input
              name="name"
              placeholder="Name"
            />

            <Spacer height="medium" />

            <Textarea
              name="data"
              placeholder="Data"
              ref={textAreaRef}
            />

            <Spacer height="medium" />
          </>
        )}
      </Form>
    </Frame>
  )
}
