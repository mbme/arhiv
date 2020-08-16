import * as React from 'react'
import { Procedure } from '@v/utils'
import { Input, Spacer, Textarea } from '@v/web-platform'
import { Frame, Action } from '../parts'
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

  const [preview, showPreview] = React.useState(false)
  const [name, setName] = React.useState(originalName)
  const [data, setData] = React.useState(originalData)

  const isValid = name && (name !== originalName || data !== originalData)

  const textAreaRef = React.useRef<Textarea>(null)

  const onAttachments = (links: string[]) => {
    textAreaRef.current!.insert(links.join(' '))
    textAreaRef.current!.focus()
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
    >
      {preview ? (
        <Note name={name} data={data} />
      ) : (
        <>
          <Input
            name="name"
            value={name}
            onChange={setName}
            autoFocus
          />

          <Spacer height="medium" />

          <Textarea
            name="data"
            value={data}
            onChange={setData}
            ref={textAreaRef}
          />

          <Spacer height="medium" />
        </>
      )}
    </Frame>
  )
}
