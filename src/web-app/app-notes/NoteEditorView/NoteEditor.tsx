import * as React from 'react'
import { useRouter } from '~/web-router'
import { Note as ArhivNote } from '~/arhiv'
import {
  Icon,
  Button,
  Input,
  Textarea,
  AttachFileButton,
  Box,
} from '~/web-platform'
import { createLink } from '~/markup-parser/utils'
import { Toolbar } from '../../parts'
import {
  Note as NoteRenderer,
} from '../Note'
import { DeleteNoteButton } from './DeleteNoteButton'

interface IProps {
  note: ArhivNote,
}

export function NoteEditor({ note }: IProps) {
  const router = useRouter()

  const [isPreview, setPreview] = React.useState(false)
  const [name, setName] = React.useState(note.name)
  const [data, setData] = React.useState(note.data)

  const textAreaRef = React.useRef<Textarea>(null)

  const deleteNote = () => {
    note.deleted = true
    note.save()
    router.pushTo('/notes')
  }
  const attachFiles = (files: File[]) => {
    const links = files.map(file => createLink(note.createAttachment(file), file.name))
    textAreaRef.current!.insert(links.join(' '))
    textAreaRef.current!.focus()
  }
  const onPreview = () => setPreview(!isPreview)
  const left = (
    <>
      {note.isNew() || <DeleteNoteButton onConfirmed={deleteNote} />}
      <AttachFileButton onSelected={attachFiles} />

      <Icon
        title="Preview"
        type={isPreview ? 'eye-off' : 'eye'}
        onClick={onPreview}
      />
    </>
  )

  const isValid = name && name !== note.name || data !== note.data
  const onCancel = () => router.push(note.isNew() ? { path: '/notes' } : { path: '/note', params: { id: note.id } })
  const onSave = () => {
    note.name = name
    note.data = data
    note.save()
    router.push({ path: '/note', params: { id: note.id } })
  }

  const right = (
    <>
      <Button onClick={onCancel}>
        Cancel
      </Button>

      <Button primary onClick={onSave} disabled={!isValid}>
        Save
      </Button>
    </>
  )

  return (
    <>
      <Toolbar left={left} right={right} />

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

      {isPreview && <NoteRenderer name={name} data={data} />}
    </>
  )
}
