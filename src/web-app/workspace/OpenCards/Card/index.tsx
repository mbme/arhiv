import * as React from 'react'
import {
  ProgressLocker,
  useObservable,
} from '~/web-platform'
import { DocumentNote } from '~/arhiv/replica'
import { useArhiv } from '~/arhiv/useArhiv'
import { NotFound } from '../../../parts'
import { NoteCard } from './NoteCard'
import { DocumentCard } from './DocumentCard'

interface IProps {
  id: string
}

export function Card({ id }: IProps) {
  const arhiv = useArhiv()
  const [document, error] = useObservable(() => arhiv.documents.getDocument$(id), [id])

  if (error) {
    return NotFound
  }

  if (!document) {
    return (
      <ProgressLocker />
    )
  }

  if (document.deleted) {
    return (
      <DocumentCard document={document} />
    )
  }

  if (document instanceof DocumentNote) {
    return (
      <NoteCard document={document} />
    )
  }

  return (
    <DocumentCard document={document} />
  )
}
