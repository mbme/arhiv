import * as React from 'react'
import { Link } from '~/web-router'
import { useIsodb } from '~/isodb-web-client'
import { Icon } from '~/web-components'
import { Toolbar, NotFound } from '../parts'

interface IProps {
  id: string
}

export function NoteView({ id }: IProps) {
  const client = useIsodb()
  const note = client.getNote(id)
  if (!note) {
    return NotFound
  }

  const right = (
    <Link to={{ path: '/note-editor', params: { id: note.id } }} clean>
      <Icon type="edit-2" />
    </Link>
  )

  return (
    <>
      <Toolbar right={right} />
    </>
  )
}
