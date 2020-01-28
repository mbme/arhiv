import * as React from 'react'
import {
  ProgressLocker,
} from '~/web-platform'
import { NotFound } from '../../../parts'
import { NoteCard } from './NoteCard'
import { DocumentCard } from './DocumentCard'
import { useObservable } from '~/web-utils'
import { ArhivContext } from '~/web-app/arhiv-context'

interface IProps {
  id: string
}

export function Card({ id }: IProps) {
  const arhiv = ArhivContext.use()
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
