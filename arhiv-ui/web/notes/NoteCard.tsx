import * as React from 'react'
import {  RouterContext } from '@v/web-utils'
import { NoteDataDescription } from '../api'
import { CardLoader, Card } from '../parts'

interface IProps {
  id: string
}

export function NoteCard({ id }: IProps) {
  const router = RouterContext.use()

  const onEdit = () => router.replace(`/notes/${id}/edit` )
  const onClose = () => router.goBack()

  return (
    <CardLoader id={id}>
      {document => (
        <Card
          document={document}
          dataDescription={NoteDataDescription}
          onEdit={onEdit}
          onClose={onClose}
        />
      )}
    </CardLoader>
  )
}
