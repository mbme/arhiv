import * as React from 'react'
import { Procedure } from '~/utils'
import { Button, Input, Row, Spacer, Textarea } from '~/web-platform'
import { AddAttachmentButton, DeleteDocumentButton, Frame } from '../../Card'
import { DocumentNote } from '../types'
import { Note } from './Note'

interface IProps {
  document: DocumentNote
  onSave(name: string, data: string): void
  onCancel: Procedure
  onDelete?: Procedure
}

export function NoteCardEditor({ document, onSave, onCancel, onDelete }: IProps) {
  const [name, setName] = React.useState(document.props.name)
  const [data, setData] = React.useState(document.props.data)

  const isValid = name && (
    name !== document.props.name || data !== document.props.data
  )

  const textAreaRef = React.useRef<Textarea>(null)

  const onAttachments = (links: string[]) => {
    textAreaRef.current!.insert(links.join(' '))
    textAreaRef.current!.focus()
  }

  const buttons = (
    <>
      {onDelete && <DeleteDocumentButton onConfirmed={onDelete} />}
    </>
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
          <AddAttachmentButton onAttachments={onAttachments} />

          <Row>
            <Button
              variant="link"
              onClick={onCancel}
            >
              Cancel
            </Button>

            <Spacer width="medium" />

            <Button
              variant="primary"
              onClick={() => onSave(name, data)}
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
