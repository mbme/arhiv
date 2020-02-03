import * as React from 'react'
import { Procedure } from '~/utils'
import {
  Button,
  Row,
  Input,
  Textarea,
  Spacer,
} from '~/web-platform'
import { Note } from './Note'
import { DeleteDocumentButton, AddAttachmentButton, Frame } from '../../Card'
import { DocumentNote } from '../types'

interface IProps {
  document: DocumentNote
  onDone: Procedure
  isNew: boolean
}

export function NoteCardEditor({ document, isNew, onDone }: IProps) {
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

  const onSave = async () => {
    document.patch({ name, data })
    await document.updateRefs(data)
    await document.save()
    onDone()
  }

  const onDelete = async () => {
    document.delete()
    await document.save()
    onDone()
  }

  const buttons = (
    <>
      {!isNew && <DeleteDocumentButton onConfirmed={onDelete} />}
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
              onClick={onDone}
            >
              Cancel
            </Button>

            <Spacer width="medium" />

            <Button
              variant="primary"
              onClick={onSave}
              disabled={!isValid}
            >
              {isNew ? 'Create' : 'Save'}
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
