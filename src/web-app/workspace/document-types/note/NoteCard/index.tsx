import * as React from 'react'
import { DocumentNote } from '../types'
import { NoteCardViewer } from './NoteCardViewer'
import { NoteCardEditor } from './NoteCardEditor'
import { useObservable } from '~/web-utils'

interface IProps {
  document: DocumentNote
}

export function NoteCard({ document }: IProps) {
  const [isNew] = useObservable(() => document.isNew$(), [document])
  const [editMode, setEditMode] = React.useState(false)

  if (isNew === undefined) {
    return null
  }

  if (isNew || editMode) {
    return (
      <NoteCardEditor
        document={document}
        onDone={() => setEditMode(false)}
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
