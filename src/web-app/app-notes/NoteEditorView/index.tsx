import * as React from 'react'
import {
  useRouter,
} from '~/web-router'
import { useIsodb } from '~/isodb-web-client'
import {
  Icon,
  Button,
} from '~/web-components'
import {
  Toolbar,
  NotFound,
  Markup,
} from '../parts'

interface IProps {
  id?: string
}

export function NoteEditorView({ id }: IProps) {
  const router = useRouter()
  const client = useIsodb()

  let note = id ? client.notes.getNote(id) : null
  React.useEffect(() => { // create note if needed
    if (!id) {
      note = client.notes.createNote()
    }
  }, [id])

  if (!note) {
    return NotFound
  }
  // TODO lock

  const onCancel = () => router.push(id ? { path: '/note', params: { id } } : { path: '/notes' })
  const onSave = () => router.push({ path: '/note', params: { id: note.id } })

  const left = (
    <Icon
      title="Preview"
      type={false ? 'eye-off' : 'eye'} // FIXME
    />
  )

  const right = (
    <Button onClick={onCancel}>
      Cancel
    </Button>
  )

  return (
    <>
      <Toolbar left={left} right={right} />

      <h1>
        {note.name}
      </h1>

      <Markup value={note.data} />
    </>
  )
}
