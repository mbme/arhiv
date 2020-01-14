import * as React from 'react'
import { Procedure } from '~/utils'
import {
  Button,
  Row,
  Input,
  Textarea,
  Spacer,
} from '~/web-platform'
import { DocumentNote } from '~/arhiv/replica'
import { Frame } from './Frame'
import { Note } from './Note'
import { AddAttachmentsButton } from './AddAttachmentButton'
import { DeleteDocumentButton } from './DeleteDocumentButton'

interface IProps {
  document: DocumentNote
  onDone: Procedure
}

export function NoteCardEditor({ document, onDone }: IProps) {
  const [name, setName] = React.useState(document.name)
  const [data, setData] = React.useState(document.data)

  const isValid = name && (
    name !== document.name || data !== document.data
  )

  const textAreaRef = React.useRef<Textarea>(null)

  const onAttachments = (links: string[]) => {
    textAreaRef.current!.insert(links.join(' '))
    textAreaRef.current!.focus()
  }

  const onSave = async () => {
    await document.patch({ name, data })
    await document.save()
    onDone()
  }

  const onDelete = async () => {
    document.delete()
    await document.save()
    onDone()
  }

  const buttons = (
    <DeleteDocumentButton onConfirmed={onDelete} />
  )

  const tabs = {
    editor: () => (
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

        <Row
          alignX="space-between"
          mb="small"
        >
          <AddAttachmentsButton onAttachments={onAttachments} />

          <Row>
            <Button
              variant="link"
              onClick={onDone}
            >
              Cancel
            </Button>

            <Spacer widht="medium" />

            <Button
              variant="primary"
              onClick={onSave}
              disabled={!isValid}
            >
              Save
            </Button>
          </Row>
        </Row>
      </>
    ),
    preview: () => <Note name={name} data={data} />,
  }

  return (
    <Frame
      tabs={tabs}
      buttons={buttons}
    />
  )
}
