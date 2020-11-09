import * as React from 'react'
import { Obj } from '@v/utils'
import {
  ProgressLocker,
} from '@v/web-platform'
import {
  usePromise,
  RouterContext,
} from '@v/web-utils'
import { API, createNote, getNote, NoteDataDescription } from '../../api'
import { CardEditor } from './CardEditor'
import { NotFoundBlock, ErrorBlock } from '../../parts'

interface IProps {
  id?: string
}

export function CardEditorContainer({ id }: IProps) {
  const router = RouterContext.use()
  const isNew = !id

  const [note, err] = usePromise(() => {
    if (id) {
      return getNote(id)
    }

    return createNote()
  }, [id])

  if (err) {
    return (
      <ErrorBlock error={err} />
    )
  }

  if (note === undefined) {
    return (
      <ProgressLocker />
    )
  }

  if (note === null) {
    return (
      <NotFoundBlock>
        Can't find note with id "{id}"
      </NotFoundBlock>
    )
  }

  const onSave = async (data: Obj) => {
    await API.put({
      ...note,
      data,
    })

    router.replace({ path: `/notes/${note.id}` })
  }

  const onDelete = async () => {
    await API.put({
      ...note,
      archived: true,
    })

    router.push({ path: '/notes' })
  }

  const onCancel = () => {
    if (isNew) {
      router.goBack()
    } else {
      router.replace({ path: `/notes/${note.id}` })
    }
  }

  return (
    <CardEditor
      data={note.data}
      dataDescription={NoteDataDescription}
      onSave={onSave}
      onDelete={id ? onDelete : undefined}
      onCancel={onCancel}
    />
  )
}
