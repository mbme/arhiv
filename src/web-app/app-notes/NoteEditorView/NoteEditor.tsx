import * as React from 'react'
import { useRouter } from '~/web-router'
import { NoteDocument } from '~/arhiv'
import {
  Icon,
  Button,
  Input,
  Textarea,
  Box,
  Spacer,
} from '~/web-platform'
import {
  Toolbar,
  AddAttachmentsButton,
} from '../../parts'
import { Note } from '../Note'
import { DeleteNoteButton } from './DeleteNoteButton'

interface IProps {
  note: NoteDocument,
}

export function NoteEditor({ note }: IProps) {
  const router = useRouter()

  const [isPreview, setPreview] = React.useState(false)
  const [name, setName] = React.useState(note.record.name)
  const [data, setData] = React.useState(note.record.data)

  const textAreaRef = React.useRef<Textarea>(null)

  const deleteNote = () => {
    note.delete()
    router.push('/notes')
  }
  const onAttachments = (links: string[]) => {
    textAreaRef.current!.insert(links.join(' '))
    textAreaRef.current!.focus()
  }
  const onPreview = () => setPreview(!isPreview)
  const isValid = name && name !== note.record.name || data !== note.record.data
  const onCancel = () => router.push(
    note.isNew()
      ? { path: '/notes' }
      : { path: '/note', params: { id: note.id } },
  )
  const onSave = () => {
    note.patch({ name, data }, data)
    router.push({ path: '/note', params: { id: note.id } })
  }

  return (
    <>
      <Toolbar>
        {note.isNew() || <DeleteNoteButton onConfirmed={deleteNote} />}
        <AddAttachmentsButton onAttachments={onAttachments} />

        <Icon
          title="Preview"
          type={isPreview ? 'eye-off' : 'eye'}
          onClick={onPreview}
        />

        <Spacer />

        <Button onClick={onCancel}>
          Cancel
        </Button>

        <Button primary onClick={onSave} disabled={!isValid}>
          Save
        </Button>
      </Toolbar>

      <Box
        hidden={isPreview}
        mb="medium"
      >
        <Input
          name="name"
          value={name}
          onChange={setName}
          autoFocus
        />
      </Box>

      <div hidden={isPreview}>
        <Textarea
          name="data"
          value={data}
          onChange={setData}
          ref={textAreaRef}
        />
      </div>

      {isPreview && <Note name={name} data={data} />}
    </>
  )
}
