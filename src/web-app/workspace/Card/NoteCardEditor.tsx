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
import { AddAttachmentsButton } from '~/web-app/parts'

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
    // FIXME modal, move to frame
    document.delete()
    await document.save()
    onDone()
  }

  const buttons = (
    <Row>
      <AddAttachmentsButton onAttachments={onAttachments} />

      <Button
        variant="link"
        onClick={onDelete}
      >
        Delete
      </Button>
      <Button
        variant="link"
        onClick={onDone}
      >
        Cancel
      </Button>
      <Button
        variant="primary"
        onClick={onSave}
        disabled={!isValid}
      >
        Save
      </Button>
    </Row>
  )

  return (
    <Frame
      tabs={['editor', 'preview']}
      buttons={buttons}
    >
      {(activeTabId) => {
        if (activeTabId === 'editor') {
          return (
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
            </>
          )
        }

        if (activeTabId === 'preview') {
          return (
            <Note name={name} data={data} />
          )
        }

        return null
      }}
    </Frame>
  )
}
