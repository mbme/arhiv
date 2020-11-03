import * as React from 'react'
import {
  ProgressLocker,
} from '@v/web-platform'
import {
  usePromise,
  RouterContext,
} from '@v/web-utils'
import { API, createNote, getNote } from '../api'
import { CardEditor } from './CardEditor'
import { NotFoundBlock, ErrorBlock } from '../parts'

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

  const onSave = async (name: string, data: string) => {
    await API.put({
      ...note,
      data: {
        name,
        data,
      },
    })

    router.replace({ path: `/${note.id}` })
  }

  const onDelete = async () => {
    await API.put({
      ...note,
      archived: true,
    })

    router.push({ path: '/' })
  }

  const onCancel = () => {
    if (isNew) {
      router.goBack()
    } else {
      router.replace({ path: `/${note.id}` })
    }
  }

  return (
    <CardEditor
      name={note.data.name}
      data={note.data.data}
      onSave={onSave}
      onDelete={id ? onDelete : undefined}
      onCancel={onCancel}
    />
  )
}
