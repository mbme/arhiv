import * as React from 'react'
import { useObservable } from '~/web-utils'
import { DocumentNote } from '../types'
import { NoteCardEditor } from './NoteCardEditor'
import { NoteCardViewer } from './NoteCardViewer'
import { useWorkspaceStore } from '~/web-app/workspace/store'

interface IProps {
  document: DocumentNote
}

export function NoteCard({ document }: IProps) {
  const [isNew] = useObservable(() => document.isNew$(), [document])
  const [editMode, setEditMode] = React.useState(false)
  const store = useWorkspaceStore()

  if (isNew === undefined) {
    return null
  }

  if (isNew || editMode) {
    const onSave = async (name: string, data: string) => {
      document.patch({ name, data })
      await document.updateRefs(data)
      await document.save()
      setEditMode(false)
    }

    const onDelete = isNew ? undefined : async () => {
      document.delete()
      await document.save()
      setEditMode(false)
    }

    const onCancel = () => {
      if (isNew) {
        store.closeDocument(document.id)
      } else {
        setEditMode(false)
      }
    }

    return (
      <NoteCardEditor
        document={document}
        onSave={onSave}
        onDelete={onDelete}
        onCancel={onCancel}
      />
    )
  }

  return (
    <NoteCardViewer
      document={document}
      onEdit={() => setEditMode(true)}
    />
  )
}
